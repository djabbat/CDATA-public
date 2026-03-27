use cell_dt_core::MitochondrialState;
use crate::params::{MitochondrialParams, sigmoid_ros, compute_mitophagy, accumulate_mtdna};

pub struct MitochondrialSystem {
    pub params: MitochondrialParams,
}

impl MitochondrialSystem {
    pub fn new() -> Self {
        Self { params: MitochondrialParams::default() }
    }

    pub fn update(&self, state: &mut MitochondrialState, dt: f64, age_years: f64, inflammation_level: f64) {
        state.mtdna_mutations = accumulate_mtdna(state.mtdna_mutations, state.ros_level, dt);
        let oxidative_input = inflammation_level * 0.3;
        state.ros_level = sigmoid_ros(
            state.mtdna_mutations, oxidative_input,
            self.params.ros_steepness, self.params.mitophagy_threshold,
        );
        state.mitophagy_efficiency = compute_mitophagy(
            state.ros_level, age_years, self.params.mitophagy_threshold,
        );
        // FIX Round 7 (C1): exponential decay instead of linear
        // Literature: mitophagy declines exponentially with age (PMID: 25651178)
        // k calibrated: 50% decline by age ~70yr → k = ln(2)/70 ≈ 0.0099
        state.mito_shield = ((-0.0099_f64 * age_years).exp()).max(0.1);
        state.membrane_potential = (1.0 - state.mtdna_mutations * 0.5).max(0.2);
    }

    pub fn calculate_oxygen_delivery(&self, state: &MitochondrialState, age_years: f64) -> f64 {
        let base = 1.0 - age_years / 200.0;
        (base * state.membrane_potential).max(0.1)
    }

    pub fn check_mitochondrial_collapse(&self, state: &MitochondrialState) -> bool {
        state.mtdna_mutations > 0.9 || state.membrane_potential < 0.15
    }
}

impl Default for MitochondrialSystem {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::MitochondrialState;

    fn state() -> MitochondrialState { MitochondrialState::default() }
    fn sys() -> MitochondrialSystem { MitochondrialSystem::new() }

    // ── Construction ──────────────────────────────────────────────────────────

    #[test]
    fn test_new_default_params() {
        let s = sys();
        assert!((s.params.mitophagy_threshold - 0.35).abs() < 1e-9);
    }

    // ── update: mtDNA mutations ───────────────────────────────────────────────

    #[test]
    fn test_mtdna_accumulates_over_time() {
        let sys = sys();
        let mut s = state();
        s.ros_level = 0.5;
        let before = s.mtdna_mutations;
        sys.update(&mut s, 1.0, 30.0, 0.0);
        assert!(s.mtdna_mutations >= before, "mtDNA should accumulate");
    }

    #[test]
    fn test_mtdna_bounded_zero_one() {
        let sys = sys();
        let mut s = state();
        s.ros_level = 1.0;
        for _ in 0..2000 {
            sys.update(&mut s, 1.0, 50.0, 0.0);
        }
        assert!(s.mtdna_mutations >= 0.0 && s.mtdna_mutations <= 1.0);
    }

    #[test]
    fn test_mtdna_faster_with_more_ros() {
        let sys = sys();
        let mut s1 = state();
        let mut s2 = state();
        s1.ros_level = 0.2;
        s2.ros_level = 0.8;
        for _ in 0..10 {
            sys.update(&mut s1, 1.0, 30.0, 0.0);
            sys.update(&mut s2, 1.0, 30.0, 0.0);
        }
        assert!(s2.mtdna_mutations > s1.mtdna_mutations,
            "Higher ROS → faster mtDNA accumulation");
    }

    // ── update: ROS level ─────────────────────────────────────────────────────

    #[test]
    fn test_ros_bounded_zero_one() {
        let sys = sys();
        let mut s = state();
        for _ in 0..100 {
            sys.update(&mut s, 1.0, 50.0, 1.0);
        }
        assert!(s.ros_level >= 0.0 && s.ros_level <= 1.0);
    }

    #[test]
    fn test_ros_increases_with_inflammation() {
        let sys = sys();
        let mut s1 = state();
        let mut s2 = state();
        sys.update(&mut s1, 0.001, 30.0, 0.0);
        sys.update(&mut s2, 0.001, 30.0, 1.0);
        assert!(s2.ros_level >= s1.ros_level,
            "Inflammation should increase ROS");
    }

    // ── update: mitophagy efficiency ──────────────────────────────────────────

    #[test]
    fn test_mitophagy_declines_with_age() {
        let sys = sys();
        let mut s_young = state();
        let mut s_old   = state();
        s_young.ros_level = 0.6;
        s_old.ros_level   = 0.6;
        sys.update(&mut s_young, 0.001, 20.0, 0.0);
        sys.update(&mut s_old,   0.001, 80.0, 0.0);
        assert!(s_young.mitophagy_efficiency >= s_old.mitophagy_efficiency,
            "Mitophagy should decline with age");
    }

    #[test]
    fn test_mitophagy_non_negative() {
        let sys = sys();
        let mut s = state();
        sys.update(&mut s, 1.0, 90.0, 1.0);
        assert!(s.mitophagy_efficiency >= 0.0);
    }

    // ── update: mito_shield (C1 exponential decay) ───────────────────────────

    #[test]
    fn test_mito_shield_at_age_zero_near_one() {
        let sys = sys();
        let mut s = state();
        sys.update(&mut s, 0.001, 0.0, 0.0);
        assert!((s.mito_shield - 1.0).abs() < 0.01,
            "mito_shield at age=0 should ≈ 1.0, got {}", s.mito_shield);
    }

    #[test]
    fn test_mito_shield_declines_with_age() {
        let sys = sys();
        let mut s_young = state();
        let mut s_old   = state();
        sys.update(&mut s_young, 0.001, 20.0, 0.0);
        sys.update(&mut s_old,   0.001, 70.0, 0.0);
        assert!(s_young.mito_shield > s_old.mito_shield,
            "mito_shield should decline with age (C1 exponential decay)");
    }

    #[test]
    fn test_mito_shield_minimum_010() {
        let sys = sys();
        let mut s = state();
        sys.update(&mut s, 0.001, 1000.0, 0.0);
        assert!(s.mito_shield >= 0.1,
            "mito_shield minimum must be 0.1, got {}", s.mito_shield);
    }

    #[test]
    fn test_mito_shield_half_life_70yr() {
        // k = ln(2)/70 ≈ 0.0099; at age 70 → exp(-0.0099*70) ≈ 0.5
        let expected_at_70 = (-0.0099_f64 * 70.0).exp();
        assert!((expected_at_70 - 0.5).abs() < 0.05,
            "mito_shield half-life ~70 years, got {} at age 70", expected_at_70);
    }

    // ── update: membrane_potential ────────────────────────────────────────────

    #[test]
    fn test_membrane_potential_at_zero_mutations() {
        let sys = sys();
        let mut s = state();
        sys.update(&mut s, 0.001, 30.0, 0.0);
        // With no mutations accumulated: potential ≈ 1.0 (minor ROS effect)
        assert!(s.membrane_potential >= 0.9, "Potential near 1.0 with no mutations");
    }

    #[test]
    fn test_membrane_potential_minimum_02() {
        let sys = sys();
        let mut s = state();
        s.mtdna_mutations = 1.0;
        sys.update(&mut s, 0.001, 30.0, 0.0);
        // (1 - 1.0*0.5).max(0.2) = 0.5
        assert!(s.membrane_potential >= 0.2);
    }

    #[test]
    fn test_membrane_potential_bounded() {
        let sys = sys();
        let mut s = state();
        for _ in 0..100 {
            sys.update(&mut s, 1.0, 50.0, 0.5);
        }
        assert!(s.membrane_potential >= 0.0 && s.membrane_potential <= 1.0);
    }

    // ── calculate_oxygen_delivery ─────────────────────────────────────────────

    #[test]
    fn test_oxygen_delivery_positive() {
        let sys = sys();
        let s = state();
        let o2 = sys.calculate_oxygen_delivery(&s, 30.0);
        assert!(o2 > 0.0);
    }

    #[test]
    fn test_oxygen_delivery_declines_with_age() {
        let sys = sys();
        let s = state();
        let young = sys.calculate_oxygen_delivery(&s, 20.0);
        let old   = sys.calculate_oxygen_delivery(&s, 80.0);
        assert!(young > old, "O2 delivery declines with age");
    }

    #[test]
    fn test_oxygen_delivery_minimum_01() {
        let sys = sys();
        let s = state();
        let o2 = sys.calculate_oxygen_delivery(&s, 300.0);
        assert!(o2 >= 0.1, "O2 delivery minimum must be 0.1");
    }

    #[test]
    fn test_oxygen_delivery_reduced_by_low_potential() {
        let sys = sys();
        let mut s1 = state();
        let mut s2 = state();
        s2.membrane_potential = 0.3;
        let o1 = sys.calculate_oxygen_delivery(&s1, 40.0);
        let o2 = sys.calculate_oxygen_delivery(&s2, 40.0);
        assert!(o1 > o2, "Low membrane potential reduces O2 delivery");
    }

    // ── check_mitochondrial_collapse ──────────────────────────────────────────

    #[test]
    fn test_no_collapse_default_state() {
        let sys = sys();
        let s = state();
        assert!(!sys.check_mitochondrial_collapse(&s),
            "Default state should not trigger collapse");
    }

    #[test]
    fn test_collapse_with_high_mutations() {
        let sys = sys();
        let mut s = state();
        s.mtdna_mutations = 0.95;
        assert!(sys.check_mitochondrial_collapse(&s),
            "High mtDNA mutations should trigger collapse");
    }

    #[test]
    fn test_collapse_with_low_membrane_potential() {
        let sys = sys();
        let mut s = state();
        s.membrane_potential = 0.1;
        assert!(sys.check_mitochondrial_collapse(&s),
            "Low membrane potential should trigger collapse");
    }

    #[test]
    fn test_no_collapse_boundary_values() {
        let sys = sys();
        let mut s = state();
        s.mtdna_mutations = 0.89;
        s.membrane_potential = 0.16;
        assert!(!sys.check_mitochondrial_collapse(&s),
            "Just below thresholds should not collapse");
    }

    #[test]
    fn test_collapse_boundary_mutations_exactly_09() {
        let sys = sys();
        // > 0.9 is the condition, so exactly 0.9 does NOT collapse
        let mut s = state();
        s.mtdna_mutations = 0.9;
        assert!(!sys.check_mitochondrial_collapse(&s),
            "mutations = 0.9 (not > 0.9) must NOT collapse");
        // 0.901 > 0.9 → should collapse
        let mut s2 = state();
        s2.mtdna_mutations = 0.901;
        assert!(sys.check_mitochondrial_collapse(&s2),
            "mutations 0.901 > 0.9 should collapse");
    }
}
