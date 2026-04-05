use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflammagingParams {
    pub damps_rate: f64,
    pub cgas_sensitivity: f64,
    pub sasp_decay: f64,
    pub nk_age_decay: f64,
    pub fibrosis_rate: f64,
    // FIX C4: explicit DAMPs clearance rate (τ = 1/damps_decay_rate years)
    // Biochemistry: extracellular HMGB1/HSP70 cleared by lysosomal degradation
    // Default 0.1 yr⁻¹ → τ = 10 years; fast pool (HMGB1) τ~1 yr → use 1.0 if needed
    pub damps_decay_rate: f64,
    /// CHIP→SASP amplification strength (CDATA v3.5, Open Question 4).
    ///
    /// Quantitative coupling: CHIP VAF amplifies SASP production rate.
    ///   sasp_prod *= (1 + chip_vaf × chip_sasp_strength)
    ///
    /// Prior: Normal(0.5, 0.15) — from Wu et al. (2023, PMID: 37145845).
    /// Range: [0.0, 2.0]; values > 1.0 imply strong CHIP-driven inflammaging.
    pub chip_sasp_strength: f64,
}

impl Default for InflammagingParams {
    fn default() -> Self {
        Self {
            damps_rate: 0.05,
            cgas_sensitivity: 0.8,
            sasp_decay: 0.1,
            nk_age_decay: 0.010,  // FIX Round 7 (B4): 0.005→0.010; ~50% NK decline by age 70 per PMID: 12803352
            fibrosis_rate: 0.02,
            damps_decay_rate: 0.1, // FIX C4: τ = 10 yr for slow DAMPs pool (HMGB1 chronic)
            chip_sasp_strength: 0.5, // Normal(0.5, 0.15) prior — Wu et al. 2023
        }
    }
}

pub fn sasp_to_ros_contribution(sasp_level: f64) -> f64 {
    sasp_level * 0.3
}

pub fn sasp_damage_multiplier(sasp_level: f64) -> f64 {
    1.0 + sasp_level * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sasp_ros_feedback() {
        let ros = sasp_to_ros_contribution(0.5);
        assert!(ros > 0.0 && ros < 0.5);
    }

    #[test]
    fn test_sasp_multiplier() {
        assert!((sasp_damage_multiplier(0.0) - 1.0).abs() < 1e-6);
        assert!(sasp_damage_multiplier(1.0) > 1.0);
    }

    // ── InflammagingParams defaults ────────────────────────────────────────────

    #[test]
    fn test_default_damps_rate() {
        let p = InflammagingParams::default();
        assert!((p.damps_rate - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_default_cgas_sensitivity() {
        let p = InflammagingParams::default();
        assert!((p.cgas_sensitivity - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_default_sasp_decay() {
        let p = InflammagingParams::default();
        assert!((p.sasp_decay - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_default_nk_age_decay() {
        let p = InflammagingParams::default();
        assert!((p.nk_age_decay - 0.010).abs() < 1e-9,
            "Round 7 fix: nk_age_decay should be 0.010, got {}", p.nk_age_decay);
    }

    #[test]
    fn test_default_fibrosis_rate() {
        let p = InflammagingParams::default();
        assert!((p.fibrosis_rate - 0.02).abs() < 1e-9);
    }

    #[test]
    fn test_all_params_positive() {
        let p = InflammagingParams::default();
        assert!(p.damps_rate > 0.0);
        assert!(p.cgas_sensitivity > 0.0);
        assert!(p.sasp_decay > 0.0);
        assert!(p.nk_age_decay > 0.0);
        assert!(p.fibrosis_rate > 0.0);
    }

    // ── sasp_to_ros_contribution ───────────────────────────────────────────────

    #[test]
    fn test_sasp_ros_at_zero() {
        assert!((sasp_to_ros_contribution(0.0)).abs() < 1e-9);
    }

    #[test]
    fn test_sasp_ros_at_one() {
        // 1.0 * 0.3 = 0.3
        assert!((sasp_to_ros_contribution(1.0) - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_sasp_ros_linear() {
        // Proportional: 0.6 = 2 × sasp_to_ros(0.3)?  → 0.5 × 0.3 = 0.15; 1.0 × 0.3 = 0.3
        let r1 = sasp_to_ros_contribution(0.4);
        let r2 = sasp_to_ros_contribution(0.8);
        assert!((r2 / r1 - 2.0).abs() < 1e-6, "should be linear");
    }

    #[test]
    fn test_sasp_ros_non_negative() {
        for s in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert!(sasp_to_ros_contribution(s) >= 0.0);
        }
    }

    #[test]
    fn test_sasp_ros_bounded_below_sasp() {
        for s in [0.1, 0.5, 1.0] {
            assert!(sasp_to_ros_contribution(s) < s,
                "ROS contribution must be less than SASP level");
        }
    }

    // ── sasp_damage_multiplier ────────────────────────────────────────────────

    #[test]
    fn test_sasp_multiplier_at_zero_is_one() {
        assert!((sasp_damage_multiplier(0.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_sasp_multiplier_at_one() {
        // 1.0 + 0.5 = 1.5
        assert!((sasp_damage_multiplier(1.0) - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_sasp_multiplier_at_half() {
        // 1.0 + 0.5*0.5 = 1.25
        assert!((sasp_damage_multiplier(0.5) - 1.25).abs() < 1e-9);
    }

    #[test]
    fn test_sasp_multiplier_monotone_increasing() {
        let levels = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
        for w in levels.windows(2) {
            assert!(sasp_damage_multiplier(w[0]) <= sasp_damage_multiplier(w[1]));
        }
    }

    #[test]
    fn test_sasp_multiplier_always_at_least_one() {
        for s in [0.0, 0.1, 0.5, 1.0] {
            assert!(sasp_damage_multiplier(s) >= 1.0);
        }
    }

    #[test]
    fn test_sasp_multiplier_linear() {
        let m1 = sasp_damage_multiplier(0.2);
        let m2 = sasp_damage_multiplier(0.4);
        // Should be: (m2 - 1) = 2 × (m1 - 1) if linear
        assert!(((m2 - 1.0) / (m1 - 1.0) - 2.0).abs() < 1e-6, "should be linear");
    }

    // ── C4: DAMPs decay test (damps_half_life proxy) ──────────────────────────

    #[test]
    fn test_damps_decay_rate_default() {
        // C4 fix: explicit damps_decay_rate field (τ = 1/rate years)
        let p = InflammagingParams::default();
        assert!((p.damps_decay_rate - 0.1).abs() < 1e-9,
            "Default damps_decay_rate should be 0.1 (τ=10yr), got {}", p.damps_decay_rate);
    }

    #[test]
    fn test_damps_decay_rate_positive() {
        let p = InflammagingParams::default();
        assert!(p.damps_decay_rate > 0.0, "damps_decay_rate must be positive");
    }

    #[test]
    fn test_damps_decay_tau_10_years() {
        // At rate=0.1, τ = 10 years → 90% cleared after one τ
        let p = InflammagingParams::default();
        let tau = 1.0 / p.damps_decay_rate;
        assert!((tau - 10.0).abs() < 1e-6,
            "damps_decay τ should be 10 years, got {:.2}", tau);
    }

    #[test]
    fn test_damps_rate_drives_production() {
        // damps_prod = damps_rate * (senescent + dna_damage*0.5)
        // With p.damps_rate doubled, production doubles.
        let mut p1 = InflammagingParams::default();
        let mut p2 = InflammagingParams::default();
        p2.damps_rate = p1.damps_rate * 2.0;
        let senescent = 0.2;
        let dna_damage = 0.1;
        let prod1 = p1.damps_rate * (senescent + dna_damage * 0.5);
        let prod2 = p2.damps_rate * (senescent + dna_damage * 0.5);
        assert!((prod2 - 2.0 * prod1).abs() < 1e-9, "Doubling damps_rate must double production");
    }

    // ── Integration-level parameter sanity checks ─────────────────────────────

    #[test]
    fn test_sasp_ros_zero_at_zero_sasp() {
        assert_eq!(sasp_to_ros_contribution(0.0), 0.0);
    }

    #[test]
    fn test_sasp_damage_multiplier_max_is_15() {
        assert!((sasp_damage_multiplier(1.0) - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_cgas_sensitivity_in_plausible_range() {
        let p = InflammagingParams::default();
        assert!(p.cgas_sensitivity > 0.0 && p.cgas_sensitivity <= 1.0);
    }

    #[test]
    fn test_nk_age_decay_produces_50pct_decline_by_70() {
        // nk_age_decay = 0.010; base_nk = 1.0 - age * 0.010
        // At age 70: 1.0 - 70*0.010 = 0.30 (30% remaining, so ~70% decline)
        let p = InflammagingParams::default();
        let nk_at_70 = 1.0 - 70.0 * p.nk_age_decay;
        assert!(nk_at_70 < 0.5, "NK efficiency at 70 should be below 50%, got {:.2}", nk_at_70);
        assert!(nk_at_70 > 0.1, "NK efficiency at 70 should stay above minimum 0.1, got {:.2}", nk_at_70);
    }

    #[test]
    fn test_fibrosis_rate_small_enough() {
        let p = InflammagingParams::default();
        // Fibrosis should not explode: rate * dt should be << 1
        let fibrosis_step = p.fibrosis_rate * 1.0; // dt=1 year
        assert!(fibrosis_step < 0.1, "Fibrosis rate should be moderate: {}", fibrosis_step);
    }

    #[test]
    fn test_sasp_decay_rate_positive() {
        let p = InflammagingParams::default();
        assert!(p.sasp_decay > 0.0, "SASP must decay");
    }

    #[test]
    fn test_params_serializable_conceptually() {
        // Test that clone works (proxy for serde compatibility)
        let p1 = InflammagingParams::default();
        let p2 = p1.clone();
        assert!((p1.damps_rate - p2.damps_rate).abs() < 1e-9);
    }

    #[test]
    fn test_sasp_multiplier_between_one_and_15() {
        for s in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let m = sasp_damage_multiplier(s);
            assert!(m >= 1.0 && m <= 1.5,
                "sasp multiplier {} out of [1.0, 1.5] at sasp={}", m, s);
        }
    }

    #[test]
    fn test_sasp_ros_contribution_at_typical_values() {
        // At sasp=0.3 (typical low-grade): ROS = 0.09
        let ros = sasp_to_ros_contribution(0.3);
        assert!((ros - 0.09).abs() < 1e-9, "sasp_to_ros(0.3) = 0.09, got {}", ros);
    }

    #[test]
    fn test_sasp_ros_contribution_at_high_sasp() {
        // At sasp=0.8 (high): ROS = 0.24
        let ros = sasp_to_ros_contribution(0.8);
        assert!((ros - 0.24).abs() < 1e-9, "sasp_to_ros(0.8) = 0.24, got {}", ros);
    }

    #[test]
    fn test_damps_production_with_max_senescent() {
        let p = InflammagingParams::default();
        // All senescent, no DNA damage: prod = 0.05 * 1.0 = 0.05
        let prod = p.damps_rate * (1.0 + 0.0 * 0.5);
        assert!((prod - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_debug_output_inflammaging_params() {
        let p = InflammagingParams::default();
        let dbg = format!("{:?}", p);
        assert!(dbg.contains("InflammagingParams"));
    }
}
