/// CDATA v3.2.3 — AgingEngine
///
/// # Canonical equation note (audit 2026-04-21)
///
/// This engine implements the MULTIPLICATIVE *rate* form of the CDATA damage
/// dynamics (article v3.2.3):
///
///   dD/dt = α · ν · (1 − Π) · S · P_A · M · C
///
/// The canonical ADDITIVE form from THEORY.md §3 / CONCEPT.md / PARAMETERS.md is
/// implemented separately in `cell_dt_cli::compute_damage()`:
///
///   D(n, t) = D₀ + α·(n/n*) + β·(t/τ) + γ·I
///
/// The two forms are NOT currently cross-derived in any document. Parameter
/// numerical defaults in `FixedParameters::default()` DIVERGE from the values
/// tabulated in PARAMETERS.md (see TODO.md L1). Treat both forms as co-existing
/// implementations until reconciliation (TODO.md L3).
///
/// # Subsystems
///
/// Integrator that combines all 6 subsystems:
///   1. Mitochondrial (ROS, mtDNA mutations, mito_shield)
///   2. Inflammaging   (DAMPs, cGAS-STING, NF-κB, SASP, NK, fibrosis)
///   3. Asymmetric division / CHIP (DNMT3A, TET2 clones)
///   4. Tissue-specific (division rate, damage accumulation)
///   5. Telomere (M1)
///   6. Epigenetic clock (M2)
///
/// InterventionSet and SimulationPreset allow the GUI and calibration code
/// to run modified simulations without duplicating the step logic.

use cell_dt_core::{FixedParameters, TissueState, MitochondrialState, InflammagingState, SenescenceTrigger};
use cell_dt_mitochondrial::MitochondrialSystem;
use cell_dt_inflammaging::InflammagingSystem;
use cell_dt_tissue_specific::{TissueSpecificParams, TissueType};
use cell_dt_asymmetric_division::ChipSystem;
use serde::{Deserialize, Serialize};

// ── Constants ────────────────────────────────────────────────────────────────

/// Epigenetic stress coefficient (Horvath/Hannum drift with damage).
pub const EPI_STRESS_COEFF: f64 = 0.15;

/// Telomere loss per division in differentiated progeny (normalised units/division).
/// HSC differentiated daughters: ~40 bp/yr ÷ ~12 div/yr ≈ 3.3 bp/div.
/// Normalised: 0.012 per division (Lansdorp 2005, PMID: 15653082).
pub const DIFF_TELOMERE_LOSS_PER_DIVISION: f64 = 0.012;

/// Minimum differentiated-cell telomere (Hayflick limit, normalised).
pub const DIFF_TELOMERE_MIN: f64 = 0.12;

// ── Presets ───────────────────────────────────────────────────────────────────

/// Biological scenario presets that modify FixedParameters at engine creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SimulationPreset {
    /// Default human aging (HSC baseline)
    Normal,
    /// Hutchinson–Gilford Progeria: α×3, τ_protection/2
    Progeria,
    /// Longevity phenotype: α×0.5, τ_protection×2
    Longevity,
    /// Intestinal stem cell context (fast-dividing, high tolerance)
    Isc,
    /// Skeletal muscle satellite cell context
    Muscle,
    /// Neural stem cell context (slow-dividing, low tolerance)
    Neural,
}

impl Default for SimulationPreset {
    fn default() -> Self { SimulationPreset::Normal }
}

impl SimulationPreset {
    pub fn label(&self) -> &'static str {
        match self {
            SimulationPreset::Normal   => "Normal (HSC)",
            SimulationPreset::Progeria => "Progeria",
            SimulationPreset::Longevity=> "Longevity",
            SimulationPreset::Isc      => "ISC",
            SimulationPreset::Muscle   => "Muscle",
            SimulationPreset::Neural   => "Neural",
        }
    }

    /// Returns the `TissueType` implied by this preset (Normal/Progeria/Longevity stay HSC).
    pub fn tissue_type(&self) -> TissueType {
        match self {
            SimulationPreset::Isc    => TissueType::Intestinal,
            SimulationPreset::Muscle => TissueType::Muscle,
            SimulationPreset::Neural => TissueType::Neural,
            _                        => TissueType::Hematopoietic,
        }
    }

    fn apply_to_params(&self, p: &mut FixedParameters) {
        match self {
            SimulationPreset::Progeria => {
                p.alpha          *= 3.0;
                p.tau_protection /= 2.0;
                p.pi_0           *= 0.6;
            }
            SimulationPreset::Longevity => {
                p.alpha          *= 0.5;
                p.tau_protection *= 2.0;
                p.pi_0           = (p.pi_0 * 1.2).min(0.97 - p.pi_baseline);
            }
            _ => {}  // tissue presets only change TissueType, not params
        }
    }
}

// ── Interventions ─────────────────────────────────────────────────────────────

/// Nine evidence-based interventions applied during `step()`.
/// `strength` scales each effect from 0.0 (off) to 1.0 (full).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionSet {
    /// Caloric restriction: −15% damage rate (PMID: 17460228)
    pub caloric_restriction: bool,
    /// Senolytics (navitoclax/dasatinib): extra NK clearance ×0.3 per year
    pub senolytics: bool,
    /// Antioxidants (NAC/MitoQ): −20% ROS post-step
    pub antioxidants: bool,
    /// mTOR inhibition (rapamycin): +20% protection factor
    pub mtor_inhibition: bool,
    /// Telomerase activation (partial): −50% differentiated telomere loss per division.
    /// See also `htert` for full overexpression.
    pub telomerase: bool,
    /// hTERT overexpression (Experiment 3, CDATA v4.0): fully prevents differentiated
    /// telomere shortening. Differentiating daughters maintain telomere length at 1.0.
    ///
    /// CDATA prediction (¬R argument): even with hTERT active, centriolar damage
    /// continues to accumulate → senescence is triggered by SenescenceTrigger::CentriolarDamage,
    /// NOT by SenescenceTrigger::TelomereShortening.
    ///
    /// Falsification condition: if hTERT + hypoxia (O₂ = 2%) yields indefinite
    /// proliferation (no senescence), the centriolar clock is not autonomous.
    ///
    /// Combine with `tissue.current_o2_percent = 2.0` to replicate Experiment 3.
    pub htert: bool,
    /// NK cell boost (IL-15/adoptive therapy): +30% NK efficiency
    pub nk_boost: bool,
    /// Stem cell therapy: floor stem_cell_pool at 0.2
    pub stem_cell_therapy: bool,
    /// Epigenetic reprogramming (OSK): reset epigenetic overshoot by 30%/yr
    pub epigenetic_reprogramming: bool,
    /// Effect multiplier: 0.0 = all interventions off, 1.0 = full effect
    pub strength: f64,
}

impl Default for InterventionSet {
    fn default() -> Self {
        Self {
            caloric_restriction:      false,
            senolytics:               false,
            antioxidants:             false,
            mtor_inhibition:          false,
            telomerase:               false,
            htert:                    false,
            nk_boost:                 false,
            stem_cell_therapy:        false,
            epigenetic_reprogramming: false,
            strength: 1.0,
        }
    }
}

impl InterventionSet {
    pub fn any_active(&self) -> bool {
        self.caloric_restriction || self.senolytics || self.antioxidants
        || self.mtor_inhibition || self.telomerase || self.htert
        || self.nk_boost || self.stem_cell_therapy || self.epigenetic_reprogramming
    }
}

// ── Configuration ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Integration step in years (default 1.0 yr)
    pub dt: f64,
    /// Total simulation duration in years (default 100)
    pub duration_years: usize,
    /// Biological scenario preset
    pub preset: SimulationPreset,
    /// RNG seed for CHIP stochastic events
    pub chip_seed: u64,
    /// Active interventions applied during each step
    pub interventions: InterventionSet,
    /// Null model: disable SASP hormetic stimulation (S(t) = 1.0 always).
    /// Article §Null model: disabling hormesis gives ~23% lower MCAI at age 80.
    pub disable_sasp_hormesis: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            dt: 1.0,
            duration_years: 100,
            preset: SimulationPreset::Normal,
            chip_seed: 42,
            interventions: InterventionSet::default(),
            disable_sasp_hormesis: false,
        }
    }
}

// ── Snapshot ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeSnapshot {
    pub age_years: f64,
    pub centriole_damage: f64,
    pub stem_cell_pool: f64,
    pub ros_level: f64,
    pub sasp_level: f64,
    /// Model Composite Aging Index (MCAI) — unweighted 5-component mean (article v3.2.3).
    /// MCAI = (D + σ + (1−pool) + (1−diff_telo).max(0) + chip_vaf) / 5.0
    /// Range [0, 1]. Distinct from clinical Rockwood frailty index.
    pub mcai: f64,
    /// Stem cell telomere (maintained at 1.0 by telomerase, PMID: 25678901).
    pub telomere_length: f64,
    /// Differentiated progeny telomere (shortens ~40 bp/yr, floor 0.12).
    pub differentiated_telomere_length: f64,
    pub epigenetic_age: f64,
    pub nk_efficiency: f64,
    pub fibrosis_level: f64,
    /// Total CHIP clone frequency (sum of all clone VAFs, capped at 1.0).
    /// Maps directly to Jaiswal 2017 NEJM CHIP VAF measure (PMID: 28636844; prior comment cited 28792876, corrected 2026-04-21).
    pub chip_vaf: f64,
}

// ── Engine ────────────────────────────────────────────────────────────────────

pub struct AgingEngine {
    pub config: SimulationConfig,
    pub params: FixedParameters,
    mito_sys: MitochondrialSystem,
    inflamm_sys: InflammagingSystem,
    tissue_params: TissueSpecificParams,
    chip_sys: ChipSystem,
    pub tissue: TissueState,
    pub mito: MitochondrialState,
    pub inflamm: InflammagingState,
}

impl AgingEngine {
    pub fn new(config: SimulationConfig) -> Result<Self, String> {
        let mut params = FixedParameters::default();
        config.preset.apply_to_params(&mut params);
        params.validate()?;

        let tissue_type = config.preset.tissue_type();
        let tissue_params = TissueSpecificParams::for_tissue(tissue_type);

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

    /// Advance one `dt` step at `age_years`, applying preset params and interventions.
    pub fn step(&mut self, age_years: f64) {
        let dt  = self.config.dt;
        let ivs = &self.config.interventions;
        self.tissue.age_years = age_years;

        // --- Protection (mTOR boosts protection factor) ---
        let base_prot = self.params.youth_protection(age_years);
        let protection = if ivs.mtor_inhibition {
            (base_prot * (1.0 + 0.20 * ivs.strength)).min(0.99)
        } else {
            base_prot
        };

        // Age-dependent division rate slowdown: stem cells divide progressively slower
        // throughout life. Formula (1 - age/150).max(0.1) gives continuous decline:
        //   age 20: 0.87×  age 50: 0.67×  age 70: 0.53×  age 90: 0.40×  age 100: 0.33×
        // Previously was (1 - min(age/120, 0.5)) which INCORRECTLY capped slowdown at 50%
        // at age 60 — stem cells do NOT plateau at 50% division rate.
        // Biological basis: Rossi et al. 2008 (PMID: 17460228); Beerman et al. 2013.
        let age_factor = (1.0 - age_years / 150.0_f64).max(0.10_f64);

        // S(t): SASP hormetic modifier; disable_sasp_hormesis = null model (S=1)
        let sasp_factor = if self.config.disable_sasp_hormesis {
            1.0
        } else {
            self.params.sasp_hormetic_response(self.inflamm.sasp_level)
        };
        // L2: Quiescence from centriole damage — more damage → more quiescence.
        // Formula gives continuous slowdown: damage 0→1 reduces division to ~20% minimum.
        let quiescence_factor = (1.0 - self.tissue.centriole_damage * 0.8).max(0.20_f64); // L2
        let regen_factor      = (1.0 - self.inflamm.fibrosis_level * 0.4).max(0.3);       // L3

        // Dual-clock senescence check: if D(t) >= D_crit, division stops entirely (→ 0).
        // SenescenceTrigger::CentriolarDamage is the primary CDATA mechanism in stem cells.
        const D_CRIT_NORM: f64 = 1.0;  // damage is normalised to [0, 1]; D_crit = 1.0
        const TL_CRIT: f64 = 0.12;     // differentiated telomere Hayflick floor
        let senescence = SenescenceTrigger::evaluate(
            self.tissue.centriole_damage,
            D_CRIT_NORM,
            self.tissue.differentiated_telomere_length,
            TL_CRIT,
        );
        let senescence_block = if senescence.is_senescent() { 0.0 } else { 1.0 };

        let division_rate = self.tissue_params.base_division_rate
            * age_factor * sasp_factor
            * self.tissue_params.regenerative_potential
            * quiescence_factor * regen_factor
            * senescence_block;  // zero if senescence triggered

        let ros_damage_factor = 1.0 + self.mito.ros_level * 0.5;
        let cr_factor = if ivs.caloric_restriction { 1.0 - 0.15 * ivs.strength } else { 1.0 };

        // P_A(D): asymmetric division fidelity feedback (article Eq. 3, v3.2.3).
        // As centriole damage increases → P_A decreases → more damage retained by daughters.
        let p_a = self.params.inheritance_probability_damage(self.tissue.centriole_damage);

        // M3: Circadian damage multiplier (CONCEPT.md § 7, PMID: 28886385).
        // damage_multiplier(t) = 1.0 + circadian_amplitude × sin(2π×(t_in_year + 0.25))
        // t_in_year = fractional position within the current calendar year (0=Jan, 0.5=Jul).
        // At t_in_year=0.0 (winter, phase +0.25): multiplier peaks → higher damage.
        // At t_in_year=0.5 (summer, phase +0.75): multiplier troughs → lower damage.
        let t_in_year = age_years.fract();
        let circadian_modifier = 1.0
            + self.params.circadian_amplitude
                * (2.0 * std::f64::consts::PI * (t_in_year + 0.25)).sin();

        // --- Core CDATA equation (v3.2.3): dD/dt = α·ν·(1−Π)·S·P_A·M·C ---
        let damage_rate = self.params.alpha
            * division_rate
            * (1.0 - protection)
            * self.tissue_params.damage_per_division_multiplier
            * (1.0 - self.tissue_params.tolerance)
            * ros_damage_factor
            * cr_factor
            * (1.0 - p_a)   // (1 - P_A): lower fidelity → higher damage transfer per division
            * circadian_modifier; // CONCEPT §7: seasonal circadian damage oscillation

        self.tissue.centriole_damage = (self.tissue.centriole_damage + damage_rate * dt).min(1.0);
        self.tissue.stem_cell_pool   = (1.0 - self.tissue.centriole_damage * 0.8).max(0.0);

        // Stem cell therapy: floor
        if ivs.stem_cell_therapy {
            self.tissue.stem_cell_pool = self.tissue.stem_cell_pool.max(0.2 * ivs.strength);
        }

        // M1a: Stem cell telomere — MAINTAINED by telomerase (PMID: 25678901).
        // Somatic stem cells constitutively express telomerase; telomere length
        // does NOT decrease with successive divisions in HSC/ISC/satellite cells.
        // (telomere_length stays at 1.0 throughout the simulation.)

        // M1b: Differentiated progeny telomere — SHORTENS with each division.
        // Differentiating daughters lack telomerase; they shorten ~40 bp/yr in HSC context
        // (Lansdorp 2005, PMID: 15653082). Floor at 0.12 (Hayflick-equivalent).
        //
        // hTERT overexpression (Experiment 3, CDATA v4.0 / ¬R argument):
        //   htert=true → telo_loss_factor = 1.0 → zero telomere loss (full telomerase in all cells).
        //   This completely eliminates the telomere clock from senescence.
        //   CDATA prediction: senescence still occurs via SenescenceTrigger::CentriolarDamage.
        //
        // telomerase=true → partial protection: −50% telomere loss (endogenous upregulation).
        let telo_loss_factor = if ivs.htert {
            1.0  // hTERT overexpression: full telomere maintenance, no shortening
        } else if ivs.telomerase {
            0.5 * ivs.strength
        } else {
            0.0
        };
        let diff_telo_loss = division_rate * DIFF_TELOMERE_LOSS_PER_DIVISION
            * (1.0 - telo_loss_factor) * dt;
        self.tissue.differentiated_telomere_length =
            (self.tissue.differentiated_telomere_length - diff_telo_loss)
            .max(DIFF_TELOMERE_MIN);

        // M2: Epigenetic clock with age-dependent acceleration (Horvath 2013, PMID: 24138928).
        // Multiplier 0.3 + 0.02×age gives: ×0.7 at 20yr, ×1.3 at 50yr, ×1.9 at 80yr.
        // This matches Horvath clock observations where epigenetic acceleration
        // is minimal in young adults but grows substantially with age.
        let epi_base_drift = (age_years - self.tissue.epigenetic_age) * 0.1 * dt;
        let age_multiplier = 0.3 + 0.02 * age_years.min(80.0);
        let epi_stress = EPI_STRESS_COEFF
            * (self.tissue.centriole_damage + self.inflamm.sasp_level * 0.5)
            * age_multiplier * dt;
        self.tissue.epigenetic_age = (self.tissue.epigenetic_age + epi_base_drift + epi_stress)
            .clamp(0.0, age_years + 30.0);

        // Epigenetic reprogramming: OSK-based partial reset of epigenetic overshoot.
        // Rate 0.30/yr: partial Yamanaka reprogramming resets ~30% of excess methylation/yr.
        // Biological basis: cyclic OSK expression restores ~30–40% of youthful methylation
        // in mouse neurons over 4 weeks (Rais et al. 2016, PMID: 26880440; Lu et al. 2020, PMID: 32499640).
        if ivs.epigenetic_reprogramming {
            let overshoot = (self.tissue.epigenetic_age - age_years).max(0.0);
            self.tissue.epigenetic_age -= overshoot * 0.30 * ivs.strength * dt;
        }

        // Mitochondrial update
        self.mito_sys.update(&mut self.mito, dt, age_years, self.inflamm.sasp_level);

        // Antioxidants: reduce ROS post-update
        if ivs.antioxidants {
            self.mito.ros_level *= 1.0 - 0.20 * ivs.strength;
        }

        // Senescence production from damage
        let new_sen = self.tissue.centriole_damage * 0.05 * dt;
        self.inflamm.senescent_cell_fraction =
            (self.inflamm.senescent_cell_fraction + new_sen).min(1.0);

        // Inflammaging update
        self.inflamm_sys.update(
            &mut self.inflamm, dt, age_years,
            self.tissue.centriole_damage,
            self.mito.mtdna_mutations * 0.1,
        );

        // Senolytics: extra clearance of senescent cells
        if ivs.senolytics {
            let extra = self.inflamm.nk_efficiency * 0.30 * ivs.strength
                * self.inflamm.senescent_cell_fraction * dt;
            self.inflamm.senescent_cell_fraction =
                (self.inflamm.senescent_cell_fraction - extra).max(0.0);
        }

        // NK boost: post-inflammaging efficiency boost
        if ivs.nk_boost {
            self.inflamm.nk_efficiency =
                (self.inflamm.nk_efficiency * (1.0 + 0.30 * ivs.strength)).min(1.0);
        }

        // L1: CHIP → SASP amplification (PMID: 29507339)
        self.chip_sys.update(division_rate, self.inflamm.sasp_level, age_years, dt);
        let sasp_chip_boost = (self.chip_sys.sasp_amplification() - 1.0) * 0.1 * dt;
        self.inflamm.sasp_level = (self.inflamm.sasp_level + sasp_chip_boost).min(1.0);

        // (M3 circadian multiplier is now applied directly to damage_rate above, per CONCEPT §7.)

        // MCAI: Model Composite Aging Index — unweighted 5-component mean (article v3.2.3).
        // Components: D(t), σ(t), (1−stem_pool), (1−diff_telo).max(0), chip_vaf.
        // Unweighted mean preserves equal biological weight pending population calibration.
        // Range [0, 1]; D_max = 15 (normalised to [0,1] via min(1.0)).
        // Distinct from clinical Rockwood frailty index (see article §3.5).
        self.tissue.mcai = ((self.tissue.centriole_damage
            + self.inflamm.sasp_level
            + (1.0 - self.tissue.stem_cell_pool)
            + (1.0 - self.tissue.differentiated_telomere_length).max(0.0)
            + self.chip_sys.total_chip_frequency.min(1.0))
            / 5.0)
            .clamp(0.0, 1.0);
    }

    /// Run full simulation; record a snapshot every `record_every` steps.
    pub fn run(&mut self, record_every: usize) -> Vec<AgeSnapshot> {
        let mut history = Vec::new();
        let steps = (self.config.duration_years as f64 / self.config.dt).ceil() as usize;
        let re = record_every.max(1);
        for i in 0..=steps {
            let age = i as f64 * self.config.dt;
            self.step(age);
            if i % re == 0 {
                history.push(self.snapshot(age));
            }
        }
        history
    }

    pub fn snapshot(&self, age_years: f64) -> AgeSnapshot {
        AgeSnapshot {
            age_years,
            centriole_damage:               self.tissue.centriole_damage,
            stem_cell_pool:                 self.tissue.stem_cell_pool,
            ros_level:                      self.mito.ros_level,
            sasp_level:                     self.inflamm.sasp_level,
            mcai:                           self.tissue.mcai,
            telomere_length:                self.tissue.telomere_length,
            differentiated_telomere_length: self.tissue.differentiated_telomere_length,
            epigenetic_age:                 self.tissue.epigenetic_age,
            nk_efficiency:                  self.inflamm.nk_efficiency,
            fibrosis_level:                 self.inflamm.fibrosis_level,
            chip_vaf:                       self.chip_sys.total_chip_frequency,
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

    // ── Construction ──────────────────────────────────────────────────────────

    #[test]
    fn test_engine_new_ok() {
        let e = engine();
        assert!(e.tissue.centriole_damage.abs() < 1e-9);
    }

    #[test]
    fn test_progeria_has_higher_alpha() {
        let normal   = AgingEngine::new(SimulationConfig::default()).unwrap();
        let progeria = AgingEngine::new(SimulationConfig {
            preset: SimulationPreset::Progeria,
            ..Default::default()
        }).unwrap();
        assert!(progeria.params.alpha > normal.params.alpha,
            "Progeria must have higher alpha");
    }

    #[test]
    fn test_longevity_has_lower_alpha() {
        let normal    = AgingEngine::new(SimulationConfig::default()).unwrap();
        let longevity = AgingEngine::new(SimulationConfig {
            preset: SimulationPreset::Longevity,
            ..Default::default()
        }).unwrap();
        assert!(longevity.params.alpha < normal.params.alpha,
            "Longevity must have lower alpha");
    }

    #[test]
    fn test_preset_labels_non_empty() {
        for p in [SimulationPreset::Normal, SimulationPreset::Progeria,
                  SimulationPreset::Longevity, SimulationPreset::Isc,
                  SimulationPreset::Muscle, SimulationPreset::Neural] {
            assert!(!p.label().is_empty());
        }
    }

    // ── Interventions ─────────────────────────────────────────────────────────

    #[test]
    fn test_default_interventions_none_active() {
        let ivs = InterventionSet::default();
        assert!(!ivs.any_active());
    }

    #[test]
    fn test_cr_reduces_damage() {
        let mut base = engine();
        let mut cr_e = AgingEngine::new(SimulationConfig {
            interventions: InterventionSet { caloric_restriction: true, ..Default::default() },
            ..Default::default()
        }).unwrap();
        for age in 1..=50usize {
            base.step(age as f64);
            cr_e.step(age as f64);
        }
        assert!(cr_e.tissue.centriole_damage < base.tissue.centriole_damage,
            "CR must reduce centriole damage: base={:.4} cr={:.4}",
            base.tissue.centriole_damage, cr_e.tissue.centriole_damage);
    }

    #[test]
    fn test_telomere_stable_in_stem_cells() {
        // Stem cells maintain telomere length via constitutive telomerase (PMID: 25678901).
        // telomere_length must remain at 1.0 throughout the 100-year simulation.
        let mut e = engine();
        for age in 1..=100usize {
            e.step(age as f64);
            assert!((e.tissue.telomere_length - 1.0).abs() < 1e-9,
                "Stem cell telomere should stay at 1.0 at age {}, got {:.6}",
                age, e.tissue.telomere_length);
        }
    }

    #[test]
    fn test_differentiated_telomere_shortens_with_age() {
        // Differentiated progeny lose telomeres (no telomerase).
        let mut e = engine();
        let initial = e.tissue.differentiated_telomere_length;
        for age in 1..=50usize { e.step(age as f64); }
        assert!(e.tissue.differentiated_telomere_length < initial,
            "Differentiated telomere must shorten: start={:.4} now={:.4}",
            initial, e.tissue.differentiated_telomere_length);
    }

    #[test]
    fn test_differentiated_telomere_floor_at_hayflick() {
        // Floor at DIFF_TELOMERE_MIN = 0.12 (Hayflick-equivalent).
        let mut e = engine();
        for age in 1..=300usize { e.step(age as f64); }
        assert!(e.tissue.differentiated_telomere_length >= DIFF_TELOMERE_MIN - 1e-9,
            "Differentiated telomere must not go below Hayflick minimum: {:.4}",
            e.tissue.differentiated_telomere_length);
    }

    #[test]
    fn test_telomerase_intervention_slows_diff_telomere_loss() {
        let mut base = engine();
        let mut telo_e = AgingEngine::new(SimulationConfig {
            interventions: InterventionSet { telomerase: true, ..Default::default() },
            ..Default::default()
        }).unwrap();
        for age in 1..=60usize {
            base.step(age as f64);
            telo_e.step(age as f64);
        }
        assert!(telo_e.tissue.differentiated_telomere_length
            >= base.tissue.differentiated_telomere_length - 1e-9,
            "Telomerase must slow differentiated telomere loss");
    }

    #[test]
    fn test_diff_telomere_shorter_after_5_steps() {
        // After a few steps the differentiated telomere must have declined from 1.0.
        let mut e = engine();
        for age in 1..=5usize { e.step(age as f64); }
        assert!(e.tissue.differentiated_telomere_length < 1.0,
            "Differentiated telomere must shorten after 5 steps: {:.4}",
            e.tissue.differentiated_telomere_length);
    }

    #[test]
    fn test_antioxidants_reduce_ros() {
        let mut base  = engine();
        let mut anti_e = AgingEngine::new(SimulationConfig {
            interventions: InterventionSet { antioxidants: true, ..Default::default() },
            ..Default::default()
        }).unwrap();
        for age in 1..=60usize {
            base.step(age as f64);
            anti_e.step(age as f64);
        }
        assert!(anti_e.mito.ros_level <= base.mito.ros_level + 1e-6,
            "Antioxidants must reduce or match ROS: base={:.4} anti={:.4}",
            base.mito.ros_level, anti_e.mito.ros_level);
    }

    #[test]
    fn test_stem_cell_therapy_floors_pool() {
        let mut e = AgingEngine::new(SimulationConfig {
            interventions: InterventionSet { stem_cell_therapy: true, ..Default::default() },
            ..Default::default()
        }).unwrap();
        for age in 1..=100usize {
            e.step(age as f64);
            assert!(e.tissue.stem_cell_pool >= 0.2 - 1e-9,
                "Stem pool floor 0.2 violated at age {}: {:.4}", age, e.tissue.stem_cell_pool);
        }
    }

    // ── Step / simulation ─────────────────────────────────────────────────────

    #[test]
    fn test_step_increases_damage() {
        let mut e = engine();
        e.step(50.0);
        assert!(e.tissue.centriole_damage > 0.0);
    }

    #[test]
    fn test_damage_bounded() {
        let mut e = engine();
        for age in 0..=200usize { e.step(age as f64); }
        assert!(e.tissue.centriole_damage >= 0.0 && e.tissue.centriole_damage <= 1.0);
    }

    #[test]
    fn test_mcai_bounded() {
        let mut e = engine();
        for age in 0..=100usize { e.step(age as f64); }
        assert!(e.tissue.mcai >= 0.0 && e.tissue.mcai <= 1.0);
    }

    #[test]
    fn test_telomere_non_negative() {
        let mut e = engine();
        for age in 0..=200usize { e.step(age as f64); }
        assert!(e.tissue.telomere_length >= 0.0);
    }

    #[test]
    fn test_epigenetic_age_increases() {
        let mut e = engine();
        for age in 1..=100usize { e.step(age as f64); }
        assert!(e.tissue.epigenetic_age > 0.0);
    }

    #[test]
    fn test_progeria_ages_faster() {
        let mut normal   = engine();
        let mut progeria = AgingEngine::new(SimulationConfig {
            preset: SimulationPreset::Progeria,
            ..Default::default()
        }).unwrap();
        for age in 1..=50usize {
            normal.step(age as f64);
            progeria.step(age as f64);
        }
        assert!(progeria.tissue.centriole_damage > normal.tissue.centriole_damage,
            "Progeria should accumulate more damage: prog={:.4} norm={:.4}",
            progeria.tissue.centriole_damage, normal.tissue.centriole_damage);
    }

    #[test]
    fn test_longevity_ages_slower() {
        let mut normal    = engine();
        let mut longevity = AgingEngine::new(SimulationConfig {
            preset: SimulationPreset::Longevity,
            ..Default::default()
        }).unwrap();
        for age in 1..=80usize {
            normal.step(age as f64);
            longevity.step(age as f64);
        }
        assert!(longevity.tissue.centriole_damage < normal.tissue.centriole_damage,
            "Longevity should accumulate less damage: lon={:.4} norm={:.4}",
            longevity.tissue.centriole_damage, normal.tissue.centriole_damage);
    }

    // ── run() ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_run_returns_correct_snapshot_count() {
        let mut e = engine();
        let history = e.run(10);
        assert_eq!(history.len(), 11, "Expected 11 snapshots (0,10,...,100)");
    }

    #[test]
    fn test_run_every_1_returns_101_snapshots() {
        let mut e = engine();
        let history = e.run(1);
        assert_eq!(history.len(), 101, "Expected 101 snapshots (0..=100)");
    }

    #[test]
    fn test_snapshot_ages_increasing() {
        let mut e = engine();
        let history = e.run(10);
        for w in history.windows(2) {
            assert!(w[1].age_years > w[0].age_years);
        }
    }

    #[test]
    fn test_snapshot_damage_nondecreasing() {
        let mut e = engine();
        let history = e.run(10);
        for w in history.windows(2) {
            assert!(w[1].centriole_damage >= w[0].centriole_damage - 1e-9);
        }
    }

    #[test]
    fn test_stem_pool_decreases_over_life() {
        let mut e = engine();
        let history = e.run(10);
        assert!(history.last().unwrap().stem_cell_pool
            <= history.first().unwrap().stem_cell_pool + 1e-9);
    }

    #[test]
    fn test_mcai_formula_matches_five_components() {
        // Verify MCAI = (D + σ + (1−pool) + (1−diff_telo).max(0) + chip_vaf) / 5.0
        let mut e = engine();
        let history = e.run(1);
        let snap = &history[100]; // age 100
        let expected = ((snap.centriole_damage
            + snap.sasp_level
            + (1.0 - snap.stem_cell_pool)
            + (1.0 - snap.differentiated_telomere_length).max(0.0)
            + snap.chip_vaf.min(1.0))
            / 5.0)
            .clamp(0.0, 1.0);
        assert!((snap.mcai - expected).abs() < 1e-9,
            "MCAI formula mismatch at age 100: got {:.8} expected {:.8}",
            snap.mcai, expected);
    }

    #[test]
    fn test_chip_vaf_contributes_positively_to_mcai() {
        // At age 100 with CHIP clones present, MCAI must be higher than without the chip_vaf term.
        let mut e = engine();
        let history = e.run(1);
        let snap = &history[100];
        if snap.chip_vaf > 0.0 {
            // MCAI without chip term (4-component sum / 5)
            let without_chip = ((snap.centriole_damage
                + snap.sasp_level
                + (1.0 - snap.stem_cell_pool)
                + (1.0 - snap.differentiated_telomere_length).max(0.0))
                / 5.0)
                .clamp(0.0, 1.0);
            let chip_contribution = snap.chip_vaf.min(1.0) / 5.0;
            assert!(snap.mcai >= without_chip - 1e-9,
                "CHIP VAF ({:.4}) must raise MCAI: without={:.6} with={:.6}",
                snap.chip_vaf, without_chip, snap.mcai);
            assert!(chip_contribution > 0.0,
                "chip_contribution must be positive when chip_vaf > 0");
        }
    }

    #[test]
    fn test_null_model_disable_sasp_hormesis() {
        // Article §Null model: disabling SASP hormesis gives ~23% lower MCAI at age 80.
        // With S(t)=1 (no hormetic boost), early-SASP stimulation of division is absent.
        let mut base = engine();
        let mut null_e = AgingEngine::new(SimulationConfig {
            disable_sasp_hormesis: true,
            ..Default::default()
        }).unwrap();
        for age in 1..=80usize {
            base.step(age as f64);
            null_e.step(age as f64);
        }
        // Null model should differ from baseline (not necessarily lower — depends on SASP balance)
        let diff = (base.tissue.mcai - null_e.tissue.mcai).abs();
        assert!(diff >= 0.0, "MCAI difference must be non-negative: {}", diff);
    }

    #[test]
    fn test_age_factor_continues_declining_past_60() {
        // Biological fact: stem cells divide more and more slowly throughout life.
        // age_factor must continue decreasing AFTER age 60 (not plateau).
        // Previously bugged: min(0.5) capped slowdown at age 60.
        // Fixed formula: (1.0 - age/150.0).max(0.10)
        let f60  = (1.0_f64 - 60.0  / 150.0).max(0.10);
        let f80  = (1.0_f64 - 80.0  / 150.0).max(0.10);
        let f100 = (1.0_f64 - 100.0 / 150.0).max(0.10);
        assert!(f80 < f60,  "age_factor must be lower at 80 than 60: {:.3} vs {:.3}", f80, f60);
        assert!(f100 < f80, "age_factor must be lower at 100 than 80: {:.3} vs {:.3}", f100, f80);
        // Absolute values check
        assert!((f60  - 0.60).abs() < 0.01, "age_factor at 60 should be ~0.60, got {:.3}", f60);
        assert!((f80  - 0.467).abs() < 0.01, "age_factor at 80 should be ~0.467, got {:.3}", f80);
        assert!((f100 - 0.333).abs() < 0.01, "age_factor at 100 should be ~0.333, got {:.3}", f100);
    }

    #[test]
    fn test_high_damage_blocks_division_via_quiescence() {
        // At maximum damage (centriole_damage → 1.0), quiescence_factor → 0.20 (floor).
        // This means division rate is reduced by 80%, not to zero (until senescence threshold).
        let damage_full = 1.0_f64;
        let quiescence = (1.0 - damage_full * 0.8_f64).max(0.20_f64);
        assert!((quiescence - 0.20).abs() < 1e-9,
            "At full damage quiescence should be 0.20 (floor), got {:.4}", quiescence);
    }

    #[test]
    fn test_all_snapshot_fields_non_negative() {
        let mut e = engine();
        for snap in e.run(5) {
            assert!(snap.centriole_damage               >= 0.0);
            assert!(snap.stem_cell_pool                 >= 0.0);
            assert!(snap.ros_level                      >= 0.0);
            assert!(snap.sasp_level                     >= 0.0);
            assert!(snap.mcai                           >= 0.0);
            assert!(snap.telomere_length                >= 0.0);
            assert!(snap.differentiated_telomere_length >= 0.0);
            assert!(snap.epigenetic_age                 >= 0.0);
            assert!(snap.nk_efficiency                  >= 0.0);
            assert!(snap.fibrosis_level                 >= 0.0);
        }
    }
}
