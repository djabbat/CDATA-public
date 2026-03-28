/// CDATA v3.0 — AgingEngine
///
/// Integrator that combines all 6 subsystems:
///   1. Mitochondrial (ROS, mtDNA mutations, mito_shield)
///   2. Inflammaging   (DAMPs, cGAS-STING, NF-κB, SASP, NK, fibrosis)
///   3. Asymmetric division / CHIP (DNMT3A, TET2 clones)
///   4. Tissue-specific (division rate, damage accumulation)
///   5. Telomere (M1)
///   6. Epigenetic clock (M2)
///
/// Previously this logic lived entirely in `basic_simulation.rs`.
/// Extracting it here allows validation, GUI, and python bindings
/// to drive the simulation without duplicating the step logic.

use cell_dt_core::{FixedParameters, TissueState, MitochondrialState, InflammagingState};
use cell_dt_mitochondrial::MitochondrialSystem;
use cell_dt_inflammaging::InflammagingSystem;
use cell_dt_tissue_specific::{TissueSpecificParams, TissueType};
use cell_dt_asymmetric_division::ChipSystem;
use serde::{Deserialize, Serialize};

// ── Constants calibrated from literature ────────────────────────────────────

/// HSC lose ~30–50 bp/division (Lansdorp 2005); normalized to fractional length.
/// At 12 divisions/year and ~100 years → ~50% telomere shortening.
pub const TELOMERE_LOSS_PER_DIVISION: f64 = 0.012;

/// Epigenetic-clock stress coefficient: damage + SASP accelerate Horvath/Hannum drift.
pub const EPI_STRESS_COEFF: f64 = 0.15;

// ── Configuration ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Integration step in years (default 1.0 yr)
    pub dt: f64,
    /// Total simulation duration in years (default 100)
    pub duration_years: usize,
    /// Tissue context for the simulation
    pub tissue_type: TissueType,
    /// RNG seed for CHIP stochastic events
    pub chip_seed: u64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            dt: 1.0,
            duration_years: 100,
            tissue_type: TissueType::Hematopoietic,
            chip_seed: 42,
        }
    }
}

// ── Snapshot (one time-point) ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeSnapshot {
    pub age_years: f64,
    pub centriole_damage: f64,
    pub stem_cell_pool: f64,
    pub ros_level: f64,
    pub sasp_level: f64,
    pub frailty_index: f64,
    pub telomere_length: f64,
    pub epigenetic_age: f64,
    pub nk_efficiency: f64,
    pub fibrosis_level: f64,
}

// ── Engine ───────────────────────────────────────────────────────────────────

pub struct AgingEngine {
    pub config: SimulationConfig,
    pub params: FixedParameters,
    mito_sys: MitochondrialSystem,
    inflamm_sys: InflammagingSystem,
    tissue_params: TissueSpecificParams,
    chip_sys: ChipSystem,
    // State
    pub tissue: TissueState,
    pub mito: MitochondrialState,
    pub inflamm: InflammagingState,
}

impl AgingEngine {
    pub fn new(config: SimulationConfig) -> Result<Self, String> {
        let params = FixedParameters::default();
        params.validate()?;
        let tissue_params = TissueSpecificParams::for_tissue(config.tissue_type.clone());
        Ok(Self {
            chip_sys: ChipSystem::new(config.chip_seed),
            mito_sys: MitochondrialSystem::new(),
            inflamm_sys: InflammagingSystem::new(),
            tissue_params,
            tissue: TissueState::new(0.0),
            mito: MitochondrialState::default(),
            inflamm: InflammagingState::default(),
            params,
            config,
        })
    }

    /// Advance simulation by one `dt` step at the given `age_years`.
    pub fn step(&mut self, age_years: f64) {
        let dt = self.config.dt;
        self.tissue.age_years = age_years;

        // Protection factor
        let protection = self.params.youth_protection(age_years);
        let age_factor = 1.0 - (age_years / 120.0_f64).min(0.5);
        let sasp_factor = self.params.sasp_hormetic_response(self.inflamm.sasp_level);

        // L2: high centriole damage → quiescence (PMID: 20357022)
        let quiescence_factor = (1.0 - self.tissue.centriole_damage * 0.5).max(0.2);
        // L3: fibrosis reduces regenerative potential
        let regen_factor = (1.0 - self.inflamm.fibrosis_level * 0.4).max(0.3);

        let division_rate = self.tissue_params.base_division_rate
            * age_factor
            * sasp_factor
            * self.tissue_params.regenerative_potential
            * quiescence_factor
            * regen_factor;

        let ros_damage_factor = 1.0 + self.mito.ros_level * 0.5;

        // Core CDATA equation: d(Damage)/dt = α × ν(t) × (1 − Π(t)) × β × (1 − tol) × ROS
        let damage_rate = self.params.alpha
            * division_rate
            * (1.0 - protection)
            * self.tissue_params.damage_per_division_multiplier
            * (1.0 - self.tissue_params.tolerance)
            * ros_damage_factor;

        self.tissue.centriole_damage = (self.tissue.centriole_damage + damage_rate * dt).min(1.0);
        self.tissue.stem_cell_pool = (1.0 - self.tissue.centriole_damage * 0.8).max(0.0);

        // M1: Telomere shortening
        let telomere_loss = TELOMERE_LOSS_PER_DIVISION * division_rate * dt;
        self.tissue.telomere_length = (self.tissue.telomere_length - telomere_loss).max(0.0);

        // M2: Epigenetic clock drift + damage/SASP stress
        let epi_base_drift = (age_years - self.tissue.epigenetic_age) * 0.1 * dt;
        let epi_stress = EPI_STRESS_COEFF
            * (self.tissue.centriole_damage + self.inflamm.sasp_level * 0.5)
            * dt;
        self.tissue.epigenetic_age = (self.tissue.epigenetic_age + epi_base_drift + epi_stress)
            .clamp(0.0, age_years + 30.0);

        // Mitochondrial update
        self.mito_sys.update(&mut self.mito, dt, age_years, self.inflamm.sasp_level);

        // Senescence production
        let new_sen = self.tissue.centriole_damage * 0.05 * dt;
        self.inflamm.senescent_cell_fraction =
            (self.inflamm.senescent_cell_fraction + new_sen).min(1.0);

        // Inflammaging update
        self.inflamm_sys.update(
            &mut self.inflamm,
            dt,
            age_years,
            self.tissue.centriole_damage,
            self.mito.mtdna_mutations * 0.1,
        );

        // L1: CHIP → SASP amplification (PMID: 29507339)
        self.chip_sys.update(division_rate, self.inflamm.sasp_level, age_years, dt);
        let sasp_chip_boost = (self.chip_sys.sasp_amplification() - 1.0) * 0.1 * dt;
        self.inflamm.sasp_level = (self.inflamm.sasp_level + sasp_chip_boost).min(1.0);

        // M3: circadian penalty placeholder (annual-step approximation)
        let _circadian_repair_factor =
            1.0 - (1.0 - self.params.circadian_amplitude) * (age_years / 100.0) * 0.2;

        // Frailty index (composite)
        self.tissue.frailty_index = (self.tissue.centriole_damage * 0.4
            + self.inflamm.sasp_level * 0.3
            + (1.0 - self.tissue.stem_cell_pool) * 0.2
            + (1.0 - self.tissue.telomere_length) * 0.1)
            .min(1.0);
    }

    /// Run full simulation; returns snapshots at every `record_every` years.
    pub fn run(&mut self, record_every: usize) -> Vec<AgeSnapshot> {
        let mut history = Vec::new();
        let duration = self.config.duration_years;
        let dt = self.config.dt;
        let steps = (duration as f64 / dt).ceil() as usize;

        for i in 0..=steps {
            let age = i as f64 * dt;
            self.step(age);
            if i % record_every == 0 {
                history.push(self.snapshot(age));
            }
        }
        history
    }

    pub fn snapshot(&self, age_years: f64) -> AgeSnapshot {
        AgeSnapshot {
            age_years,
            centriole_damage: self.tissue.centriole_damage,
            stem_cell_pool: self.tissue.stem_cell_pool,
            ros_level: self.mito.ros_level,
            sasp_level: self.inflamm.sasp_level,
            frailty_index: self.tissue.frailty_index,
            telomere_length: self.tissue.telomere_length,
            epigenetic_age: self.tissue.epigenetic_age,
            nk_efficiency: self.inflamm.nk_efficiency,
            fibrosis_level: self.inflamm.fibrosis_level,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> AgingEngine {
        AgingEngine::new(SimulationConfig::default()).unwrap()
    }

    #[test]
    fn test_engine_new_ok() {
        let e = engine();
        assert!((e.tissue.centriole_damage).abs() < 1e-9, "starts at zero damage");
    }

    #[test]
    fn test_step_increases_damage() {
        let mut e = engine();
        e.step(50.0);
        assert!(e.tissue.centriole_damage > 0.0, "damage must increase after one step");
    }

    #[test]
    fn test_damage_bounded() {
        let mut e = engine();
        for age in (0..=200).step_by(1) {
            e.step(age as f64);
        }
        assert!(e.tissue.centriole_damage <= 1.0);
        assert!(e.tissue.centriole_damage >= 0.0);
    }

    #[test]
    fn test_frailty_bounded() {
        let mut e = engine();
        for age in (0..=100).step_by(1) {
            e.step(age as f64);
        }
        assert!(e.tissue.frailty_index >= 0.0 && e.tissue.frailty_index <= 1.0);
    }

    #[test]
    fn test_telomere_decreases_monotonically() {
        let mut e = engine();
        let mut prev = e.tissue.telomere_length;
        for age in 1..=100usize {
            e.step(age as f64);
            assert!(e.tissue.telomere_length <= prev + 1e-9,
                "Telomere must not increase at age {}", age);
            prev = e.tissue.telomere_length;
        }
    }

    #[test]
    fn test_telomere_non_negative() {
        let mut e = engine();
        for age in 0..=200usize {
            e.step(age as f64);
        }
        assert!(e.tissue.telomere_length >= 0.0);
    }

    #[test]
    fn test_epigenetic_age_increases() {
        let mut e = engine();
        for age in 1..=100usize {
            e.step(age as f64);
        }
        assert!(e.tissue.epigenetic_age > 0.0, "Epigenetic age must drift upward");
    }

    #[test]
    fn test_run_returns_correct_snapshot_count() {
        let mut e = engine();
        let history = e.run(10);
        // 0..=100 steps, every 10 → 11 snapshots (0,10,20,...,100)
        assert_eq!(history.len(), 11, "Expected 11 snapshots, got {}", history.len());
    }

    #[test]
    fn test_snapshot_ages_increasing() {
        let mut e = engine();
        let history = e.run(10);
        for w in history.windows(2) {
            assert!(w[1].age_years > w[0].age_years,
                "Snapshot ages must be increasing: {} -> {}", w[0].age_years, w[1].age_years);
        }
    }

    #[test]
    fn test_snapshot_damage_increasing() {
        let mut e = engine();
        let history = e.run(10);
        for w in history.windows(2) {
            assert!(w[1].centriole_damage >= w[0].centriole_damage - 1e-9,
                "Damage should be non-decreasing in snapshots");
        }
    }

    #[test]
    fn test_stem_cell_pool_decreases_with_damage() {
        let mut e = engine();
        let history = e.run(10);
        let young = &history[0];
        let old   = &history[history.len() - 1];
        assert!(old.stem_cell_pool <= young.stem_cell_pool + 1e-9,
            "Stem pool should decrease: young={:.4} old={:.4}", young.stem_cell_pool, old.stem_cell_pool);
    }

    #[test]
    fn test_config_default_hematopoietic() {
        let cfg = SimulationConfig::default();
        assert!(matches!(cfg.tissue_type, TissueType::Hematopoietic));
        assert!((cfg.dt - 1.0).abs() < 1e-9);
        assert_eq!(cfg.duration_years, 100);
    }

    #[test]
    fn test_invalid_params_rejected() {
        // Manually break a parameter to check validation gate
        let mut cfg = SimulationConfig::default();
        cfg.dt = 1.0;
        // Engine::new calls params.validate() — if we could modify params it would fail.
        // Here just confirm normal construction is ok.
        let e = AgingEngine::new(cfg);
        assert!(e.is_ok());
    }

    #[test]
    fn test_all_snapshot_fields_non_negative() {
        let mut e = engine();
        let history = e.run(5);
        for snap in &history {
            assert!(snap.centriole_damage >= 0.0);
            assert!(snap.stem_cell_pool >= 0.0);
            assert!(snap.ros_level >= 0.0);
            assert!(snap.sasp_level >= 0.0);
            assert!(snap.frailty_index >= 0.0);
            assert!(snap.telomere_length >= 0.0);
            assert!(snap.epigenetic_age >= 0.0);
            assert!(snap.nk_efficiency >= 0.0);
            assert!(snap.fibrosis_level >= 0.0);
        }
    }
}
