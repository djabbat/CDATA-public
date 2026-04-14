use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use cell_dt_core::FixedParameters;

#[derive(Debug, Default)]
pub struct AsymmetryStatistics {
    pub total_divisions: u64,
    pub maternal_inheritances: u64,
}

impl AsymmetryStatistics {
    pub fn record_division(&mut self, inherited_maternal: bool) {
        self.total_divisions += 1;
        if inherited_maternal {
            self.maternal_inheritances += 1;
        }
    }

    pub fn asymmetry_fraction(&self) -> f64 {
        if self.total_divisions == 0 { return 0.0; }
        self.maternal_inheritances as f64 / self.total_divisions as f64
    }
}

pub struct AsymmetricDivisionSystem {
    rng: ChaCha8Rng,
    pub stats: AsymmetryStatistics,
}

impl AsymmetricDivisionSystem {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            stats: AsymmetryStatistics::default(),
        }
    }

    pub fn calculate_probability(params: &FixedParameters, centriole_damage: f64, spindle_fidelity: f64) -> f64 {
        params.inheritance_probability(centriole_damage, spindle_fidelity)
    }

    pub fn roll_division(&mut self, params: &FixedParameters, centriole_damage: f64, spindle_fidelity: f64) -> bool {
        let prob = Self::calculate_probability(params, centriole_damage, spindle_fidelity);
        let inherited = self.rng.gen::<f64>() < prob;
        self.stats.record_division(inherited);
        inherited
    }

    pub fn damage_multiplier(inherited_maternal: bool) -> f64 {
        if inherited_maternal { 1.2 } else { 0.3 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probability_bounds() {
        let params = FixedParameters::default();
        let p_low_damage = AsymmetricDivisionSystem::calculate_probability(&params, 2.0, 1.0);
        let p_high_damage = AsymmetricDivisionSystem::calculate_probability(&params, 8.0, 0.5);
        assert!(p_low_damage >= 0.60 && p_low_damage <= 0.98, "p_low_damage={}", p_low_damage);
        assert!(p_high_damage >= 0.60 && p_high_damage <= 0.98, "p_high_damage={}", p_high_damage);
        assert!(p_low_damage > p_high_damage);
    }

    #[test]
    fn test_stochastic_distribution() {
        let params = FixedParameters::default();
        let mut sys = AsymmetricDivisionSystem::new(42);
        for _ in 0..1000 {
            sys.roll_division(&params, 5.0, 0.9);
        }
        let fraction = sys.stats.asymmetry_fraction();
        assert!(fraction > 0.5 && fraction < 0.99, "fraction={}", fraction);
    }

    // ── AsymmetryStatistics ───────────────────────────────────────────────────

    #[test]
    fn test_stats_default_zero() {
        let s = AsymmetryStatistics::default();
        assert_eq!(s.total_divisions, 0);
        assert_eq!(s.maternal_inheritances, 0);
    }

    #[test]
    fn test_asymmetry_fraction_zero_when_no_divisions() {
        let s = AsymmetryStatistics::default();
        assert_eq!(s.asymmetry_fraction(), 0.0);
    }

    #[test]
    fn test_record_division_increments_total() {
        let mut s = AsymmetryStatistics::default();
        s.record_division(true);
        s.record_division(false);
        assert_eq!(s.total_divisions, 2);
    }

    #[test]
    fn test_record_maternal_increments_count() {
        let mut s = AsymmetryStatistics::default();
        s.record_division(true);
        s.record_division(true);
        s.record_division(false);
        assert_eq!(s.maternal_inheritances, 2);
    }

    #[test]
    fn test_asymmetry_fraction_calculation() {
        let mut s = AsymmetryStatistics::default();
        for _ in 0..3 { s.record_division(true); }
        for _ in 0..1 { s.record_division(false); }
        assert!((s.asymmetry_fraction() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_asymmetry_fraction_all_maternal() {
        let mut s = AsymmetryStatistics::default();
        for _ in 0..100 { s.record_division(true); }
        assert!((s.asymmetry_fraction() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_asymmetry_fraction_none_maternal() {
        let mut s = AsymmetryStatistics::default();
        for _ in 0..100 { s.record_division(false); }
        assert!((s.asymmetry_fraction()).abs() < 1e-9);
    }

    // ── calculate_probability ─────────────────────────────────────────────────

    #[test]
    fn test_prob_decreases_monotonically_with_damage() {
        let params = FixedParameters::default();
        let damages = [0.0, 2.0, 4.0, 6.0, 8.0, 10.0];
        let fidelity = 0.9;
        for w in damages.windows(2) {
            let p1 = AsymmetricDivisionSystem::calculate_probability(&params, w[0], fidelity);
            let p2 = AsymmetricDivisionSystem::calculate_probability(&params, w[1], fidelity);
            assert!(p1 >= p2, "prob at damage={} ({}) must be >= prob at damage={} ({})", w[0], p1, w[1], p2);
        }
    }

    #[test]
    fn test_prob_decreases_with_fidelity_loss() {
        let params = FixedParameters::default();
        let damage = 5.0;
        let p_good = AsymmetricDivisionSystem::calculate_probability(&params, damage, 1.0);
        let p_bad  = AsymmetricDivisionSystem::calculate_probability(&params, damage, 0.5);
        assert!(p_good >= p_bad, "Better spindle fidelity → higher probability");
    }

    #[test]
    fn test_prob_always_in_clamp_range() {
        let params = FixedParameters::default();
        for damage in [0.0, 2.5, 5.0, 7.5, 10.0, 15.0] {
            for fidelity in [0.0, 0.5, 1.0] {
                let p = AsymmetricDivisionSystem::calculate_probability(&params, damage, fidelity);
                assert!(p >= 0.60 && p <= 0.98,
                    "prob={} out of clamp range at damage={} fid={}", p, damage, fidelity);
            }
        }
    }

    // ── roll_division ─────────────────────────────────────────────────────────

    #[test]
    fn test_roll_division_returns_bool() {
        let params = FixedParameters::default();
        let mut sys = AsymmetricDivisionSystem::new(42);
        let result = sys.roll_division(&params, 3.0, 0.9);
        // Just a bool — test that stats update
        assert_eq!(sys.stats.total_divisions, 1);
        let _ = result;
    }

    #[test]
    fn test_roll_division_tracks_stats() {
        let params = FixedParameters::default();
        let mut sys = AsymmetricDivisionSystem::new(99);
        for _ in 0..500 {
            sys.roll_division(&params, 4.0, 0.9);
        }
        assert_eq!(sys.stats.total_divisions, 500);
        assert!(sys.stats.maternal_inheritances <= 500);
    }

    #[test]
    fn test_roll_division_fraction_in_expected_range() {
        let params = FixedParameters::default();
        let mut sys = AsymmetricDivisionSystem::new(12345);
        for _ in 0..2000 {
            sys.roll_division(&params, 0.0, 1.0);
        }
        let f = sys.stats.asymmetry_fraction();
        // p0_inheritance=0.94, damage=0, fidelity=1 → p=0.94 → fraction should be near 0.94
        assert!(f > 0.88 && f < 0.98,
            "Fraction should be near 0.94, got {}", f);
    }

    // ── damage_multiplier ─────────────────────────────────────────────────────

    #[test]
    fn test_damage_multiplier_maternal_12() {
        assert!((AsymmetricDivisionSystem::damage_multiplier(true) - 1.2).abs() < 1e-9);
    }

    #[test]
    fn test_damage_multiplier_paternal_03() {
        assert!((AsymmetricDivisionSystem::damage_multiplier(false) - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_damage_multiplier_maternal_greater_than_paternal() {
        let m = AsymmetricDivisionSystem::damage_multiplier(true);
        let p = AsymmetricDivisionSystem::damage_multiplier(false);
        assert!(m > p, "Maternal inheritance has higher damage multiplier");
    }

    #[test]
    fn test_damage_multiplier_positive() {
        assert!(AsymmetricDivisionSystem::damage_multiplier(true) > 0.0);
        assert!(AsymmetricDivisionSystem::damage_multiplier(false) > 0.0);
    }

    // ── Seeded reproducibility ────────────────────────────────────────────────

    #[test]
    fn test_same_seed_same_results() {
        let params = FixedParameters::default();
        let mut sys1 = AsymmetricDivisionSystem::new(77777);
        let mut sys2 = AsymmetricDivisionSystem::new(77777);
        for _ in 0..100 {
            sys1.roll_division(&params, 5.0, 0.9);
            sys2.roll_division(&params, 5.0, 0.9);
        }
        assert_eq!(sys1.stats.maternal_inheritances, sys2.stats.maternal_inheritances,
            "Same seed must produce same results");
    }

    #[test]
    fn test_different_seeds_different_results() {
        let params = FixedParameters::default();
        let mut sys1 = AsymmetricDivisionSystem::new(1);
        let mut sys2 = AsymmetricDivisionSystem::new(2);
        for _ in 0..1000 {
            sys1.roll_division(&params, 5.0, 0.9);
            sys2.roll_division(&params, 5.0, 0.9);
        }
        // With very different seeds, exact counts should differ
        // (this could technically fail with probability < 1e-300 but practically never)
        assert_ne!(sys1.stats.maternal_inheritances, sys2.stats.maternal_inheritances,
            "Different seeds should produce different results");
    }

    #[test]
    fn test_record_division_false_does_not_increment_maternal() {
        let mut s = AsymmetryStatistics::default();
        s.record_division(false);
        s.record_division(false);
        s.record_division(false);
        assert_eq!(s.maternal_inheritances, 0);
        assert_eq!(s.total_divisions, 3);
    }

    #[test]
    fn test_asymmetry_fraction_increases_with_more_maternal() {
        let mut s1 = AsymmetryStatistics::default();
        let mut s2 = AsymmetryStatistics::default();
        for _ in 0..10 { s1.record_division(true); s1.record_division(false); }
        for _ in 0..10 { s2.record_division(true); }
        assert!(s2.asymmetry_fraction() > s1.asymmetry_fraction());
    }

    #[test]
    fn test_probability_at_extreme_low_fidelity() {
        let params = FixedParameters::default();
        // fidelity=0 → max fidelity_loss effect
        let p = AsymmetricDivisionSystem::calculate_probability(&params, 5.0, 0.0);
        assert!(p >= 0.60 && p <= 0.98);
    }

    #[test]
    fn test_damage_multiplier_ratio() {
        let maternal  = AsymmetricDivisionSystem::damage_multiplier(true);
        let paternal  = AsymmetricDivisionSystem::damage_multiplier(false);
        // 1.2 / 0.3 = 4.0
        assert!((maternal / paternal - 4.0).abs() < 1e-9,
            "Maternal/paternal damage ratio should be 4.0");
    }

    #[test]
    fn test_stats_large_number_of_divisions() {
        let mut s = AsymmetryStatistics::default();
        for _ in 0..10000 { s.record_division(true); }
        assert_eq!(s.total_divisions, 10000);
        assert_eq!(s.maternal_inheritances, 10000);
        assert!((s.asymmetry_fraction() - 1.0).abs() < 1e-9);
    }
}
