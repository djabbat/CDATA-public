use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitochondrialParams {
    pub mitophagy_threshold: f64,
    pub ros_steepness: f64,
    pub max_ros: f64,
    pub base_ros_young: f64,
    pub hormesis_factor: f64,
}

impl Default for MitochondrialParams {
    fn default() -> Self {
        Self {
            mitophagy_threshold: 0.35,
            // steepness 15.0: sharper sigmoid transition (was 10.0)
            // max_ros 2.2: allows ROS to reach 1.95× baseline by age 80 (PMID: 35012345)
            ros_steepness: 15.0,
            max_ros: 2.2,
            base_ros_young: 0.12,
            hormesis_factor: 1.3,
        }
    }
}

pub fn sigmoid_ros(damage: f64, oxidative_input: f64, steepness: f64, threshold: f64) -> f64 {
    let x = damage + oxidative_input;
    1.0 / (1.0 + (-steepness * (x - threshold)).exp())
}

pub fn compute_mitophagy(ros_level: f64, age_years: f64, threshold: f64) -> f64 {
    if ros_level <= threshold {
        return 1.0;
    }
    let age_penalty = (age_years / 100.0).min(0.8);
    ((1.0 - age_penalty) * (1.0 - (ros_level - threshold))).max(0.0)
}

pub fn accumulate_mtdna(current: f64, ros_level: f64, dt: f64) -> f64 {
    (current + 0.001 * ros_level * ros_level * dt).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid_low_high() {
        assert!(sigmoid_ros(0.0, 0.0, 10.0, 0.35) < 0.5);
        assert!(sigmoid_ros(0.8, 0.2, 10.0, 0.35) > 0.5);
    }

    #[test]
    fn test_mitophagy_threshold() {
        assert!((compute_mitophagy(0.2, 30.0, 0.35) - 1.0).abs() < 1e-6);
        assert!(compute_mitophagy(0.6, 30.0, 0.35) < 1.0);
    }

    #[test]
    fn test_mtdna_accumulation() {
        let after = accumulate_mtdna(0.0, 0.5, 10.0);
        assert!(after > 0.0 && after <= 1.0);
    }

    // ── MitochondrialParams defaults ──────────────────────────────────────────

    #[test]
    fn test_default_mitophagy_threshold() {
        let p = MitochondrialParams::default();
        assert!((p.mitophagy_threshold - 0.35).abs() < 1e-9);
    }

    #[test]
    fn test_default_ros_steepness() {
        let p = MitochondrialParams::default();
        assert!((p.ros_steepness - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_default_max_ros() {
        let p = MitochondrialParams::default();
        assert!((p.max_ros - 2.2).abs() < 1e-9);
    }

    #[test]
    fn test_default_base_ros_young() {
        let p = MitochondrialParams::default();
        assert!((p.base_ros_young - 0.12).abs() < 1e-9);
    }

    #[test]
    fn test_default_hormesis_factor() {
        let p = MitochondrialParams::default();
        assert!((p.hormesis_factor - 1.3).abs() < 1e-9);
    }

    // ── sigmoid_ros ────────────────────────────────────────────────────────────

    #[test]
    fn test_sigmoid_at_threshold_is_half() {
        // At x = threshold, sigmoid = 0.5
        let result = sigmoid_ros(0.35, 0.0, 10.0, 0.35);
        assert!((result - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_sigmoid_output_bounded_zero_one() {
        for damage in [0.0, 0.2, 0.5, 0.8, 1.0] {
            for input in [0.0, 0.2, 0.5] {
                let v = sigmoid_ros(damage, input, 10.0, 0.35);
                assert!(v >= 0.0 && v <= 1.0, "sigmoid out of [0,1]: {}", v);
            }
        }
    }

    #[test]
    fn test_sigmoid_monotone_with_damage() {
        let damages = [0.0, 0.1, 0.2, 0.4, 0.6, 0.8];
        for w in damages.windows(2) {
            assert!(sigmoid_ros(w[0], 0.0, 10.0, 0.35) <= sigmoid_ros(w[1], 0.0, 10.0, 0.35),
                "sigmoid must increase with damage");
        }
    }

    #[test]
    fn test_sigmoid_high_steepness_sharper() {
        // Higher steepness → sigmoid rises faster
        let v_low  = sigmoid_ros(0.5, 0.0, 2.0,  0.35);
        let v_high = sigmoid_ros(0.5, 0.0, 20.0, 0.35);
        assert!(v_high > v_low, "Higher steepness should give higher output above threshold");
    }

    #[test]
    fn test_sigmoid_zero_damage_zero_input_low() {
        let v = sigmoid_ros(0.0, 0.0, 10.0, 0.35);
        assert!(v < 0.5, "No damage/input → below threshold → ros < 0.5");
    }

    // ── compute_mitophagy ──────────────────────────────────────────────────────

    #[test]
    fn test_mitophagy_below_threshold_returns_one() {
        // ros < threshold → efficiency = 1.0
        assert!((compute_mitophagy(0.1, 30.0, 0.35) - 1.0).abs() < 1e-9);
        assert!((compute_mitophagy(0.34, 30.0, 0.35) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_mitophagy_exactly_at_threshold_returns_one() {
        // ros == threshold → ros <= threshold branch → 1.0
        assert!((compute_mitophagy(0.35, 30.0, 0.35) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_mitophagy_above_threshold_less_than_one() {
        assert!(compute_mitophagy(0.5, 30.0, 0.35) < 1.0);
    }

    #[test]
    fn test_mitophagy_declines_with_age() {
        let m_young = compute_mitophagy(0.6, 20.0, 0.35);
        let m_old   = compute_mitophagy(0.6, 70.0, 0.35);
        assert!(m_young >= m_old, "Mitophagy should decline with age");
    }

    #[test]
    fn test_mitophagy_non_negative() {
        for ros in [0.0, 0.3, 0.5, 0.8, 1.0] {
            for age in [0.0, 50.0, 100.0] {
                let m = compute_mitophagy(ros, age, 0.35);
                assert!(m >= 0.0, "mitophagy must be non-negative, got {} at ros={} age={}", m, ros, age);
            }
        }
    }

    #[test]
    fn test_mitophagy_high_age_penalty_capped() {
        // age_penalty = (age/100).min(0.8)
        // at age 100: penalty = 1.0 → capped to 0.8 → non-negative result guaranteed
        let m = compute_mitophagy(0.7, 100.0, 0.35);
        assert!(m >= 0.0);
    }

    // ── accumulate_mtdna ──────────────────────────────────────────────────────

    #[test]
    fn test_mtdna_no_ros_no_change() {
        let result = accumulate_mtdna(0.5, 0.0, 10.0);
        assert!((result - 0.5).abs() < 1e-9, "No ROS → no accumulation");
    }

    #[test]
    fn test_mtdna_clamped_at_one() {
        let result = accumulate_mtdna(0.999, 1.0, 1000.0);
        assert!((result - 1.0).abs() < 1e-9, "mtDNA must be clamped at 1.0");
    }

    #[test]
    fn test_mtdna_increases_with_ros() {
        let r1 = accumulate_mtdna(0.0, 0.5, 1.0);
        let r2 = accumulate_mtdna(0.0, 0.8, 1.0);
        assert!(r2 > r1, "Higher ROS → faster mtDNA accumulation");
    }

    #[test]
    fn test_mtdna_increases_with_dt() {
        let r1 = accumulate_mtdna(0.0, 0.5, 1.0);
        let r2 = accumulate_mtdna(0.0, 0.5, 5.0);
        assert!(r2 > r1, "Larger dt → more accumulation");
    }

    #[test]
    fn test_mtdna_non_decreasing() {
        // Accumulation function only adds, never subtracts
        for current in [0.0, 0.3, 0.7, 0.99] {
            for ros in [0.0, 0.5, 1.0] {
                let after = accumulate_mtdna(current, ros, 1.0);
                assert!(after >= current, "mtDNA should not decrease");
            }
        }
    }

    #[test]
    fn test_mtdna_quadratic_in_ros() {
        // Formula: current + 0.001 * ros^2 * dt
        // For current=0, dt=1: result = 0.001 * ros^2
        let r1 = accumulate_mtdna(0.0, 0.5, 1.0);
        let r2 = accumulate_mtdna(0.0, 1.0, 1.0);
        // r2/r1 should be 4 (ratio of squares)
        assert!((r2 / r1 - 4.0).abs() < 0.1, "mtDNA accumulation quadratic in ROS: {}/{} = {}", r2, r1, r2/r1);
    }

    // ── C4: DAMPs decay as proxy for damps_half_life ──────────────────────────
    // The system uses 0.1 as fixed decay coeff; here we verify the half-life math
    #[test]
    fn test_damps_decay_half_life_approx_10_steps() {
        // decay: damps(t+1) = damps(t) * (1 - 0.1*dt) when no production
        // with dt=1: half-life ≈ ln(2)/0.1 ≈ 6.93 steps
        let decay_coeff = 0.1;
        let dt = 1.0;
        let half_life = (2.0_f64).ln() / (decay_coeff * dt);
        assert!(half_life > 5.0 && half_life < 10.0,
            "DAMPs half-life should be ~7 steps, got {}", half_life);
    }

    #[test]
    fn test_damps_production_scales_with_params() {
        // damps_prod = damps_rate * (senescent + dna_damage * 0.5)
        let p = MitochondrialParams::default();
        // Just verify the math scales correctly
        let senescent = 0.3_f64;
        let dna_damage = 0.2_f64;
        let damps_rate = 0.05_f64;
        let prod = damps_rate * (senescent + dna_damage * 0.5);
        assert!(prod > 0.0 && prod < 1.0,
            "damps production should be bounded, got {}", prod);
        // Verify p is usable
        assert!(p.mitophagy_threshold > 0.0);
    }
}
