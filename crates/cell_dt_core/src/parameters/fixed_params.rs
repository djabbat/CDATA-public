use serde::{Deserialize, Serialize};

/// 32 параметра модели CDATA v3.2.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedParameters {
    // Базовые
    pub alpha: f64,
    pub hayflick_limit: f64,
    pub base_ros_young: f64,
    // Защита молодости.
    // NOTE (audit 2026-04-21): `pi_baseline` corresponds to symbol `π_base` /
    // `pi_base` in PARAMETERS.md / THEORY.md §3.2. Rename tracked in TODO.md L2.
    // Numerical defaults below diverge from PARAMETERS.md canon — see TODO.md L1.
    pub pi_0: f64,
    pub tau_protection: f64,
    pub pi_baseline: f64,
    // Асимметрия деления
    pub p0_inheritance: f64,
    /// β_A — sensitivity of asymmetric division fidelity to centriolar damage.
    /// P_A(D) = p0_inheritance · exp(−beta_a_fidelity · D).
    /// Renamed from age_decline_rate (v3.2.3): now damage-based (article Eq. 3).
    pub beta_a_fidelity: f64,
    /// Spindle fidelity loss penalty (used in stochastic per-division model).
    pub fidelity_loss: f64,
    // Тканевые — HSC
    pub hsc_nu: f64,
    /// NOTE (2026-04-21 audit): `hsc_beta` is a DEAD FIELD in the multiplicative
    /// AgingEngine (β_HSC is not used in any damage/SASP/CHIP/division equation
    /// in `cell_dt_modules/aging_engine/`). Active β_HSC lives in the separate
    /// additive form `cell_dt_cli::CounterParams` where it participates in
    /// D = D₀ + α·n + β·t. Retained here for API stability + serialization;
    /// PARAMETERS.md row annotates the duality.
    pub hsc_beta: f64,
    pub hsc_tau: f64,
    // Тканевые — ISC
    pub isc_nu: f64,
    pub isc_beta: f64,
    pub isc_tau: f64,
    // Тканевые — Muscle
    pub muscle_nu: f64,
    pub muscle_beta: f64,
    pub muscle_tau: f64,
    // Тканевые — Neural
    pub neural_nu: f64,
    pub neural_beta: f64,
    pub neural_tau: f64,
    // SASP
    pub stim_threshold: f64,
    pub inhib_threshold: f64,
    pub max_stimulation: f64,
    pub max_inhibition: f64,
    // CHIP
    pub dnmt3a_fitness: f64,
    pub dnmt3a_age_slope: f64,
    pub tet2_fitness: f64,
    pub tet2_age_slope: f64,
    // Прочие
    /// mTOR activity baseline (0–1). Reserved for WP2 (EIC Pathfinder).
    /// Not yet integrated into AgingEngine dynamics (expansion point for future module).
    pub mtor_activity: f64,
    /// Circadian rhythm amplitude (0–1). Partially integrated: used in AgingEngine
    /// as a static repair factor modifier. Dynamic circadian modulation is WP3.
    pub circadian_amplitude: f64,
    /// Meiotic reset efficiency (0–1). Reserved for germline simulation (WP2).
    /// Oogenesis not included in Cell-DT v3.0 (Limitation #5, CONCEPT.md §7).
    pub meiotic_reset: f64,
    /// YAP/TAZ mechanotransduction sensitivity (0–1). Reserved for WP3.
    /// No mechanotransduction module in v3.0 (Limitation #6, CONCEPT.md §7).
    pub yap_taz_sensitivity: f64,
}

impl Default for FixedParameters {
    fn default() -> Self {
        Self {
            alpha: 0.0082,
            hayflick_limit: 50.0,
            base_ros_young: 0.12,
            pi_0: 0.87,
            tau_protection: 24.3,
            pi_baseline: 0.10,
            p0_inheritance: 0.94,
            beta_a_fidelity: 0.15,
            fidelity_loss: 0.10,
            hsc_nu: 1.2,
            hsc_beta: 1.0,
            hsc_tau: 0.3,
            isc_nu: 70.0,
            isc_beta: 0.3,
            isc_tau: 0.8,
            muscle_nu: 4.0,
            muscle_beta: 1.2,
            muscle_tau: 0.5,
            neural_nu: 2.0,
            neural_beta: 1.5,
            neural_tau: 0.2,
            stim_threshold: 0.3,
            inhib_threshold: 0.8,
            max_stimulation: 1.5,
            max_inhibition: 0.3,
            // CHIP fitness parameters — reference values for documentation and sensitivity analysis.
            // NOTE: The actual fitness_advantage() formula in chip_drift.rs uses its own calibrated
            // constants (DNMT3A: 0.015 + 0.0002×age; TET2: 0.012 + 0.00015×age), derived by
            // fitting VAF = 0.07 at age 70 (Jaiswal SS et al. 2017 NEJM, PMID 28636844 — "Clonal Hematopoiesis and Risk of Atherosclerotic Cardiovascular Disease"). Note: prior comment cited PMID 28792876 which is a different unrelated paper — corrected 2026-04-21.
            // dnmt3a_fitness = 0.15 is the reference population-level value (/10yr unit);
            // the per-year formula coefficients (0.015) = dnmt3a_fitness / 10.
            // These fields are used in calibration sensitivity analysis (insensitive: ΔR²≈0 at ±20%).
            dnmt3a_fitness: 0.15,
            dnmt3a_age_slope: 0.002,
            tet2_fitness: 0.12,
            tet2_age_slope: 0.0015,
            mtor_activity: 0.7,
            circadian_amplitude: 0.2,
            meiotic_reset: 0.8,
            yap_taz_sensitivity: 0.5,
        }
    }
}

impl FixedParameters {
    /// Validates internal consistency of parameters.
    /// Must pass before use in simulations.
    pub fn validate(&self) -> Result<(), String> {
        if self.pi_0 + self.pi_baseline > 1.0 {
            return Err(format!(
                "pi_0 ({}) + pi_baseline ({}) > 1.0: protection at t=0 would exceed 100%",
                self.pi_0, self.pi_baseline
            ));
        }
        if self.alpha <= 0.0 || self.alpha > 0.1 {
            return Err(format!("alpha ({}) out of plausible range (0, 0.1]", self.alpha));
        }
        if self.stim_threshold >= self.inhib_threshold {
            return Err(format!(
                "stim_threshold ({}) must be < inhib_threshold ({})",
                self.stim_threshold, self.inhib_threshold
            ));
        }
        for (name, val) in [("hsc_tau", self.hsc_tau), ("isc_tau", self.isc_tau),
                             ("muscle_tau", self.muscle_tau), ("neural_tau", self.neural_tau)] {
            if val <= 0.0 || val > 1.0 {
                return Err(format!("{} ({}) must be in (0, 1]", name, val));
            }
        }
        Ok(())
    }

    pub fn youth_protection(&self, age_years: f64) -> f64 {
        self.pi_0 * (-age_years / self.tau_protection).exp() + self.pi_baseline
    }

    /// P_A(D, spindle) — asymmetric division fidelity (article Eq. 3, v3.2.3).
    /// Exponential decay with centriolar damage + spindle fidelity penalty:
    ///   P_A = p0 · exp(−β_A · D) · (1 − fidelity_loss · (1 − spindle_fidelity))
    /// Used by stochastic per-division model (stochastic.rs).
    ///
    /// Empirical validation (CDATA v4.6):
    /// - Human NPCs: ~80% of self-renewing daughters inherit older centrosome (Royall 2023, eLife)
    /// - Murine CD8+ T cells: >90% directed inheritance (Barandun & Oxenius 2025, Cell Reports)
    /// - p0=0.94 is the upper limit (healthy young cell, full Ninein activity, no damage)
    /// - Ninein-dependent modulation tracked via AsymmetricInheritance::ninein_activity field
    pub fn inheritance_probability(&self, centriole_damage: f64, spindle_fidelity: f64) -> f64 {
        let p = self.p0_inheritance
            * (-self.beta_a_fidelity * centriole_damage).exp()
            * (1.0 - self.fidelity_loss * (1.0 - spindle_fidelity));
        p.clamp(0.60, 0.98)
    }

    /// P_A(D) — damage-only variant used in the core AgingEngine damage equation.
    ///   P_A = p0 · exp(−β_A · D)
    /// As D increases, fidelity declines → more damage retained by stem daughters.
    pub fn inheritance_probability_damage(&self, centriole_damage: f64) -> f64 {
        (self.p0_inheritance * (-self.beta_a_fidelity * centriole_damage).exp())
            .clamp(0.0, 1.0)
    }

    pub fn sasp_hormetic_response(&self, sasp: f64) -> f64 {
        if sasp < self.stim_threshold {
            1.0 + (self.max_stimulation - 1.0) / self.stim_threshold * sasp
        } else if sasp <= self.inhib_threshold {
            let range = self.inhib_threshold - self.stim_threshold;
            let t = (sasp - self.stim_threshold) / range;
            self.max_stimulation - (self.max_stimulation - 1.0) * t
        } else {
            1.0 / (1.0 + 3.0 * (sasp - self.inhib_threshold))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_32_parameters() {
        let p = FixedParameters::default();
        assert!((p.alpha - 0.0082).abs() < 1e-6);
        assert!((p.pi_0 - 0.87).abs() < 1e-6);
        assert!((p.hsc_nu - 1.2).abs() < 1e-6);
        assert!((p.isc_nu - 70.0).abs() < 1e-6);
        assert!((p.dnmt3a_fitness - 0.15).abs() < 1e-6);
    }

    #[test]
    fn test_youth_protection_decay() {
        let p = FixedParameters::default();
        assert!(p.youth_protection(0.0) > p.youth_protection(25.0));
        assert!(p.youth_protection(25.0) > p.youth_protection(100.0));
        assert!(p.youth_protection(100.0) >= p.pi_baseline);
    }

    #[test]
    fn test_inheritance_probability_bounds() {
        let p = FixedParameters::default();
        // centriole_damage=5.0 (moderate), spindle_fidelity=0.8
        let prob = p.inheritance_probability(5.0, 0.8);
        assert!(prob >= 0.60, "prob={}", prob);
        assert!(prob <= 0.98, "prob={}", prob);
        // Low damage > High damage (higher damage = lower fidelity)
        assert!(p.inheritance_probability(2.0, 1.0) > p.inheritance_probability(8.0, 0.5));
    }

    #[test]
    fn test_sasp_hormesis() {
        let p = FixedParameters::default();
        assert!(p.sasp_hormetic_response(0.1) > 1.0, "Low SASP should stimulate");
        assert!(p.sasp_hormetic_response(0.95) < 1.0, "High SASP should inhibit");
        // Пик где-то в районе stim_threshold
        assert!(p.sasp_hormetic_response(0.0).abs() <= p.max_stimulation + 0.01);
    }

    // ── Validate ──────────────────────────────────────────────────────────────

    #[test]
    fn test_validate_default_ok() {
        let p = FixedParameters::default();
        assert!(p.validate().is_ok(), "Default params must be valid");
    }

    #[test]
    fn test_validate_pi_sum_exceeds_one() {
        let mut p = FixedParameters::default();
        p.pi_0 = 0.95;
        p.pi_baseline = 0.10;
        assert!(p.validate().is_err(), "pi_0 + pi_baseline > 1 should fail");
    }

    #[test]
    fn test_validate_alpha_zero() {
        let mut p = FixedParameters::default();
        p.alpha = 0.0;
        assert!(p.validate().is_err(), "alpha=0 should fail");
    }

    #[test]
    fn test_validate_alpha_too_large() {
        let mut p = FixedParameters::default();
        p.alpha = 0.11;
        assert!(p.validate().is_err(), "alpha > 0.1 should fail");
    }

    #[test]
    fn test_validate_alpha_boundary_exact() {
        let mut p = FixedParameters::default();
        p.alpha = 0.1;
        assert!(p.validate().is_ok(), "alpha=0.1 (boundary) must be valid");
    }

    #[test]
    fn test_validate_stim_ge_inhib_fails() {
        let mut p = FixedParameters::default();
        p.stim_threshold = 0.8;
        p.inhib_threshold = 0.8;
        assert!(p.validate().is_err(), "stim >= inhib must fail");
    }

    #[test]
    fn test_validate_stim_greater_than_inhib_fails() {
        let mut p = FixedParameters::default();
        p.stim_threshold = 0.9;
        p.inhib_threshold = 0.5;
        assert!(p.validate().is_err(), "stim > inhib must fail");
    }

    #[test]
    fn test_validate_tau_zero_fails() {
        let mut p = FixedParameters::default();
        p.hsc_tau = 0.0;
        assert!(p.validate().is_err(), "hsc_tau=0 should fail");
    }

    #[test]
    fn test_validate_tau_exceeds_one_fails() {
        let mut p = FixedParameters::default();
        p.isc_tau = 1.1;
        assert!(p.validate().is_err(), "isc_tau > 1 should fail");
    }

    #[test]
    fn test_validate_tau_boundary_one_ok() {
        let mut p = FixedParameters::default();
        p.isc_tau = 1.0;
        assert!(p.validate().is_ok(), "isc_tau=1.0 boundary must be valid");
    }

    #[test]
    fn test_validate_muscle_tau_boundary() {
        let mut p = FixedParameters::default();
        p.muscle_tau = 0.0001;
        assert!(p.validate().is_ok(), "tiny positive muscle_tau must be valid");
    }

    #[test]
    fn test_validate_neural_tau_zero_fails() {
        let mut p = FixedParameters::default();
        p.neural_tau = 0.0;
        assert!(p.validate().is_err(), "neural_tau=0 should fail");
    }

    // ── youth_protection ──────────────────────────────────────────────────────

    #[test]
    fn test_youth_protection_at_zero() {
        let p = FixedParameters::default();
        let expected = p.pi_0 + p.pi_baseline;
        assert!((p.youth_protection(0.0) - expected).abs() < 1e-9);
    }

    #[test]
    fn test_youth_protection_asymptote() {
        let p = FixedParameters::default();
        // At very large age the exp term vanishes → approaches pi_baseline
        let large_age = p.youth_protection(1000.0);
        assert!((large_age - p.pi_baseline).abs() < 1e-4);
    }

    #[test]
    fn test_youth_protection_monotone_decreasing() {
        let p = FixedParameters::default();
        let ages = [0.0, 10.0, 20.0, 30.0, 50.0, 80.0, 100.0];
        for w in ages.windows(2) {
            assert!(p.youth_protection(w[0]) >= p.youth_protection(w[1]),
                "protection must decrease with age");
        }
    }

    #[test]
    fn test_youth_protection_positive_always() {
        let p = FixedParameters::default();
        for age in [0.0, 25.0, 50.0, 75.0, 100.0, 150.0] {
            assert!(p.youth_protection(age) > 0.0, "protection at age {} must be positive", age);
        }
    }

    #[test]
    fn test_youth_protection_at_tau() {
        // At t = tau_protection the exp = 1/e; value = pi_0/e + pi_baseline
        let p = FixedParameters::default();
        let expected = p.pi_0 * (-1.0_f64).exp() + p.pi_baseline;
        let actual = p.youth_protection(p.tau_protection);
        assert!((actual - expected).abs() < 1e-9);
    }

    // ── inheritance_probability ───────────────────────────────────────────────

    #[test]
    fn test_inheritance_probability_clamp_lower() {
        let p = FixedParameters::default();
        // Very old, bad spindle → should hit lower clamp
        let prob = p.inheritance_probability(200.0, 0.0);
        assert!(prob >= 0.60, "prob must not go below 0.60, got {}", prob);
    }

    #[test]
    fn test_inheritance_probability_clamp_upper() {
        let p = FixedParameters::default();
        let prob = p.inheritance_probability(0.0, 1.0);
        assert!(prob <= 0.98, "prob must not exceed 0.98, got {}", prob);
    }

    #[test]
    fn test_inheritance_probability_decreases_with_damage() {
        let p = FixedParameters::default();
        let fidelity = 0.9;
        assert!(p.inheritance_probability(2.0, fidelity) >= p.inheritance_probability(6.0, fidelity));
        assert!(p.inheritance_probability(6.0, fidelity) >= p.inheritance_probability(10.0, fidelity));
    }

    #[test]
    fn test_inheritance_probability_decreases_with_fidelity_loss() {
        let p = FixedParameters::default();
        let age = 50.0;
        assert!(p.inheritance_probability(age, 1.0) >= p.inheritance_probability(age, 0.5));
        assert!(p.inheritance_probability(age, 0.5) >= p.inheritance_probability(age, 0.0));
    }

    #[test]
    fn test_inheritance_probability_young_perfect_spindle() {
        let p = FixedParameters::default();
        let prob = p.inheritance_probability(0.0, 1.0);
        // p0_inheritance - 0 - 0 = 0.94; clamped to 0.94
        assert!((prob - 0.94).abs() < 1e-9);
    }

    #[test]
    fn test_inheritance_probability_range_all_damage_levels() {
        let p = FixedParameters::default();
        for damage in [0.0, 1.0, 2.5, 5.0, 7.5, 10.0, 12.0] {
            for fidelity in [0.0, 0.5, 1.0] {
                let prob = p.inheritance_probability(damage, fidelity);
                assert!(prob >= 0.60 && prob <= 0.98,
                    "prob={} at damage={} fidelity={}", prob, damage, fidelity);
            }
        }
    }

    // ── sasp_hormetic_response ─────────────────────────────────────────────────
    // B5: continuity at transition points

    #[test]
    fn test_sasp_continuity_at_stim_threshold() {
        let p = FixedParameters::default();
        let eps = 1e-7;
        let t = p.stim_threshold;
        let left  = p.sasp_hormetic_response(t - eps);
        let right = p.sasp_hormetic_response(t + eps);
        let at    = p.sasp_hormetic_response(t);
        assert!((left - at).abs() < 1e-4,
            "discontinuity at stim_threshold: left={} at={}", left, at);
        assert!((right - at).abs() < 1e-4,
            "discontinuity at stim_threshold: right={} at={}", right, at);
    }

    #[test]
    fn test_sasp_continuity_at_inhib_threshold() {
        let p = FixedParameters::default();
        let eps = 1e-7;
        let t = p.inhib_threshold;
        let left  = p.sasp_hormetic_response(t - eps);
        let right = p.sasp_hormetic_response(t + eps);
        let at    = p.sasp_hormetic_response(t);
        assert!((left - at).abs() < 1e-4,
            "discontinuity at inhib_threshold: left={} at={}", left, at);
        assert!((right - at).abs() < 1e-4,
            "discontinuity at inhib_threshold: right={} at={}", right, at);
    }

    #[test]
    fn test_sasp_at_zero_equals_one() {
        let p = FixedParameters::default();
        // sasp=0 → branch 1: 1.0 + (max_stim - 1)/stim * 0 = 1.0
        assert!((p.sasp_hormetic_response(0.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_sasp_at_stim_threshold_equals_max_stimulation() {
        let p = FixedParameters::default();
        // At stim_threshold branch 1 gives: 1 + (max_stim - 1) = max_stim
        let val = p.sasp_hormetic_response(p.stim_threshold);
        assert!((val - p.max_stimulation).abs() < 1e-6,
            "sasp at stim_threshold should = max_stimulation={}, got={}", p.max_stimulation, val);
    }

    #[test]
    fn test_sasp_at_inhib_threshold_equals_one() {
        let p = FixedParameters::default();
        // At inhib_threshold branch 2: t=1 → max_stim - (max_stim - 1)*1 = 1.0
        let val = p.sasp_hormetic_response(p.inhib_threshold);
        assert!((val - 1.0).abs() < 1e-6,
            "sasp at inhib_threshold should = 1.0, got={}", val);
    }

    #[test]
    fn test_sasp_above_inhib_threshold_inhibitory() {
        let p = FixedParameters::default();
        let val = p.sasp_hormetic_response(p.inhib_threshold + 0.1);
        assert!(val < 1.0, "Above inhib_threshold must be inhibitory, got={}", val);
    }

    #[test]
    fn test_sasp_response_positive_everywhere() {
        let p = FixedParameters::default();
        for sasp in [0.0, 0.1, 0.3, 0.5, 0.8, 0.9, 1.0, 2.0] {
            let val = p.sasp_hormetic_response(sasp);
            assert!(val > 0.0, "Response must be positive at sasp={}, got={}", sasp, val);
        }
    }

    #[test]
    fn test_sasp_low_region_monotone_increasing() {
        let p = FixedParameters::default();
        // In [0, stim_threshold]: linear, slope > 0
        let v0 = p.sasp_hormetic_response(0.0);
        let v1 = p.sasp_hormetic_response(p.stim_threshold * 0.5);
        let v2 = p.sasp_hormetic_response(p.stim_threshold);
        assert!(v0 <= v1 && v1 <= v2, "Low region must be monotone increasing");
    }

    #[test]
    fn test_sasp_mid_region_monotone_decreasing() {
        let p = FixedParameters::default();
        // In (stim_threshold, inhib_threshold]: decreasing from max_stim back to 1.0
        let mid = (p.stim_threshold + p.inhib_threshold) / 2.0;
        let v_low  = p.sasp_hormetic_response(p.stim_threshold + 1e-9);
        let v_mid  = p.sasp_hormetic_response(mid);
        let v_high = p.sasp_hormetic_response(p.inhib_threshold);
        assert!(v_low >= v_mid && v_mid >= v_high, "Mid region must be monotone decreasing");
    }

    #[test]
    fn test_sasp_high_region_approaches_zero() {
        let p = FixedParameters::default();
        // At very high SASP the sigmoid-like inhibition → 0
        let val = p.sasp_hormetic_response(10.0);
        assert!(val < 0.05, "Very high SASP should approach 0, got={}", val);
    }

    #[test]
    fn test_sasp_hormesis_stimulation_at_low_values() {
        let p = FixedParameters::default();
        for sasp in [0.05, 0.1, 0.15, 0.2, 0.25] {
            assert!(p.sasp_hormetic_response(sasp) > 1.0,
                "sasp={} (below stim_threshold={}) should stimulate", sasp, p.stim_threshold);
        }
    }

    #[test]
    fn test_sasp_hormesis_inhibition_at_high_values() {
        let p = FixedParameters::default();
        for sasp in [0.85, 0.9, 0.95, 1.0, 1.5] {
            assert!(p.sasp_hormetic_response(sasp) < 1.0,
                "sasp={} (above inhib_threshold={}) should inhibit", sasp, p.inhib_threshold);
        }
    }

    #[test]
    fn test_sasp_max_stimulation_capped() {
        let p = FixedParameters::default();
        for sasp in [0.0, 0.1, 0.2, 0.3] {
            let val = p.sasp_hormetic_response(sasp);
            assert!(val <= p.max_stimulation + 1e-9,
                "Response {} exceeds max_stimulation {}", val, p.max_stimulation);
        }
    }

    // ── Default parameter values ───────────────────────────────────────────────

    #[test]
    fn test_all_tissue_nu_positive() {
        let p = FixedParameters::default();
        assert!(p.hsc_nu > 0.0);
        assert!(p.isc_nu > 0.0);
        assert!(p.muscle_nu > 0.0);
        assert!(p.neural_nu > 0.0);
    }

    #[test]
    fn test_all_tissue_beta_positive() {
        let p = FixedParameters::default();
        assert!(p.hsc_beta > 0.0);
        assert!(p.isc_beta > 0.0);
        assert!(p.muscle_beta > 0.0);
        assert!(p.neural_beta > 0.0);
    }

    #[test]
    fn test_chip_fitness_defaults() {
        let p = FixedParameters::default();
        assert!((p.dnmt3a_fitness - 0.15).abs() < 1e-9);
        assert!((p.tet2_fitness - 0.12).abs() < 1e-9);
        assert!(p.dnmt3a_fitness > p.tet2_fitness,
            "DNMT3A fitness must exceed TET2 fitness by default");
    }

    #[test]
    fn test_chip_age_slopes_positive() {
        let p = FixedParameters::default();
        assert!(p.dnmt3a_age_slope > 0.0);
        assert!(p.tet2_age_slope > 0.0);
    }

    #[test]
    fn test_hayflick_limit_positive() {
        let p = FixedParameters::default();
        assert!(p.hayflick_limit > 0.0);
    }

    #[test]
    fn test_mtor_and_circadian_bounds() {
        let p = FixedParameters::default();
        assert!(p.mtor_activity > 0.0 && p.mtor_activity <= 1.0);
        assert!(p.circadian_amplitude >= 0.0 && p.circadian_amplitude <= 1.0);
    }

    #[test]
    fn test_meiotic_reset_range() {
        let p = FixedParameters::default();
        assert!(p.meiotic_reset > 0.0 && p.meiotic_reset <= 1.0);
    }

    #[test]
    fn test_isc_nu_greater_than_hsc_nu() {
        let p = FixedParameters::default();
        assert!(p.isc_nu > p.hsc_nu,
            "ISC divides faster than HSC: isc_nu={} hsc_nu={}", p.isc_nu, p.hsc_nu);
    }

    #[test]
    fn test_hsc_nu_smaller_than_isc() {
        // Per PARAMETERS.md canon: HSC ν=1.2/yr, ISC ν≈52/yr, Sat ν=0.1/yr, NPC ν=4/yr.
        // Code currently has muscle_nu=4.0 and neural_nu=2.0 — not yet aligned with canon
        // (STATE.md ordering subset of L1). Until reconciliation, only assert the robust
        // HSC < ISC ordering, which holds in both code and canon.
        let p = FixedParameters::default();
        assert!(p.hsc_nu < p.isc_nu,
            "HSC slower than ISC: hsc={} isc={}", p.hsc_nu, p.isc_nu);
    }

    #[test]
    fn test_stim_threshold_less_than_inhib_threshold() {
        let p = FixedParameters::default();
        assert!(p.stim_threshold < p.inhib_threshold);
    }

    #[test]
    fn test_max_stimulation_greater_than_one() {
        let p = FixedParameters::default();
        assert!(p.max_stimulation > 1.0, "Stimulation must be > 1");
    }

    #[test]
    fn test_max_inhibition_less_than_one() {
        let p = FixedParameters::default();
        assert!(p.max_inhibition < 1.0, "Inhibition factor must be < 1");
    }

    #[test]
    fn test_pi_baseline_positive() {
        let p = FixedParameters::default();
        assert!(p.pi_baseline > 0.0);
    }

    #[test]
    fn test_p0_inheritance_high() {
        let p = FixedParameters::default();
        assert!(p.p0_inheritance > 0.5 && p.p0_inheritance <= 1.0);
    }

    #[test]
    fn test_beta_a_fidelity_positive() {
        let p = FixedParameters::default();
        assert!(p.beta_a_fidelity > 0.0);
    }

    // ── Monotonicity tests: damage increases with age ──────────────────────────

    #[test]
    fn test_youth_protection_total_damage_proxy_increases_with_age() {
        // proxy: 1 - youth_protection (inverse of protection = "damage load")
        let p = FixedParameters::default();
        let damage_young = 1.0 - p.youth_protection(20.0);
        let damage_mid   = 1.0 - p.youth_protection(50.0);
        let damage_old   = 1.0 - p.youth_protection(80.0);
        assert!(damage_young < damage_mid && damage_mid < damage_old,
            "Damage proxy must increase with age");
    }

    #[test]
    fn test_sasp_at_half_stimulation_range() {
        let p = FixedParameters::default();
        // Half-way in stimulation range
        let mid = p.stim_threshold / 2.0;
        let val = p.sasp_hormetic_response(mid);
        // Should be between 1.0 and max_stimulation
        assert!(val >= 1.0 && val <= p.max_stimulation);
    }

    #[test]
    fn test_inheritance_decay_with_damage_penalty() {
        let p = FixedParameters::default();
        // At same spindle, increasing damage from 0 to 10 should reduce fidelity
        let p0 = p.inheritance_probability(0.0, 1.0);
        let p1 = p.inheritance_probability(10.0, 1.0);
        // exponential decay: p1 = p0 * exp(-0.15 * 10) < p0
        let decline = p0 - p1;
        assert!(decline >= 0.0, "Should only decline or stay flat with increasing damage");
    }

    #[test]
    fn test_validate_pi_sum_at_boundary_exactly_one() {
        let mut p = FixedParameters::default();
        p.pi_0 = 0.9;
        p.pi_baseline = 0.1;
        // Sum == 1.0: NOT > 1.0, so should be valid
        assert!(p.validate().is_ok(), "pi_0 + pi_baseline == 1.0 should be valid");
    }

    #[test]
    fn test_hayflick_limit_default_50() {
        let p = FixedParameters::default();
        assert!((p.hayflick_limit - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_base_ros_young_default() {
        let p = FixedParameters::default();
        assert!((p.base_ros_young - 0.12).abs() < 1e-9);
    }

    #[test]
    fn test_tau_protection_default_positive() {
        let p = FixedParameters::default();
        assert!(p.tau_protection > 0.0);
    }

    #[test]
    fn test_fidelity_loss_default() {
        let p = FixedParameters::default();
        assert!((p.fidelity_loss - 0.10).abs() < 1e-9);
    }

    #[test]
    fn test_sasp_response_not_nan() {
        let p = FixedParameters::default();
        for sasp in [0.0, 0.15, 0.3, 0.55, 0.8, 1.0, 2.0, 5.0] {
            let v = p.sasp_hormetic_response(sasp);
            assert!(!v.is_nan(), "sasp_hormetic_response({}) = NaN", sasp);
        }
    }

    #[test]
    fn test_sasp_response_not_infinite() {
        let p = FixedParameters::default();
        for sasp in [0.0, 0.5, 1.0, 10.0] {
            let v = p.sasp_hormetic_response(sasp);
            assert!(!v.is_infinite(), "sasp_hormetic_response({}) = Inf", sasp);
        }
    }

    #[test]
    fn test_inheritance_probability_not_nan() {
        let p = FixedParameters::default();
        for damage in [0.0, 5.0, 10.0] {
            for fidelity in [0.0, 0.5, 1.0] {
                let v = p.inheritance_probability(damage, fidelity);
                assert!(!v.is_nan());
            }
        }
    }

    #[test]
    fn test_youth_protection_not_nan_or_infinite() {
        let p = FixedParameters::default();
        for age in [0.0, 10.0, 50.0, 100.0, 1000.0] {
            let v = p.youth_protection(age);
            assert!(!v.is_nan() && !v.is_infinite());
        }
    }
}
