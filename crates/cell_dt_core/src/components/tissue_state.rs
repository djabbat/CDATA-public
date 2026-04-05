use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TissueState {
    pub age_years: f64,
    pub stem_cell_pool: f64,
    pub centriole_damage: f64,
    pub division_count: u64,
    pub mcai: f64,
    pub epigenetic_age: f64,
    /// Stem cell telomere — maintained at 1.0 by constitutive telomerase (PMID: 25678901).
    pub telomere_length: f64,
    /// Differentiated progeny telomere (normalised: 1.0 = young adult, 0.12 = Hayflick).
    /// Differentiating daughters lack telomerase → shorten ~30–50 bp/yr (Lansdorp 2005).
    pub differentiated_telomere_length: f64,
    /// Ambient O₂ concentration in the stem cell niche (% O₂).
    /// Default: 21.0 (normoxia / standard culture).
    /// HSC niche in vivo: ~1–3%. Set to 2.0 to simulate physiological hypoxia.
    /// Used by MitochondrialSystem to compute mito_shield_for_o2() (CDATA v3.4).
    pub current_o2_percent: f64,
}

impl Default for TissueState {
    fn default() -> Self {
        Self {
            age_years: 0.0,
            stem_cell_pool: 1.0,
            centriole_damage: 0.0,
            division_count: 0,
            mcai: 0.0,
            epigenetic_age: 0.0,
            telomere_length: 1.0,
            differentiated_telomere_length: 1.0,
            current_o2_percent: 21.0,
        }
    }
}

impl TissueState {
    pub fn new(age_years: f64) -> Self {
        Self { age_years, epigenetic_age: age_years, ..Default::default() }
    }

    /// Create tissue starting at given age and oxygen tension.
    pub fn new_with_o2(age_years: f64, o2_percent: f64) -> Self {
        Self {
            age_years,
            epigenetic_age: age_years,
            current_o2_percent: o2_percent,
            ..Default::default()
        }
    }

    pub fn is_viable(&self) -> bool {
        self.stem_cell_pool > 0.05 && self.mcai < 0.95
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Default state ──────────────────────────────────────────────────────────

    #[test]
    fn test_default_age_zero() {
        let s = TissueState::default();
        assert_eq!(s.age_years, 0.0);
    }

    #[test]
    fn test_default_stem_cell_pool_full() {
        let s = TissueState::default();
        assert!((s.stem_cell_pool - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_default_centriole_damage_zero() {
        let s = TissueState::default();
        assert_eq!(s.centriole_damage, 0.0);
    }

    #[test]
    fn test_default_division_count_zero() {
        let s = TissueState::default();
        assert_eq!(s.division_count, 0);
    }

    #[test]
    fn test_default_mcai_zero() {
        let s = TissueState::default();
        assert_eq!(s.mcai, 0.0);
    }

    #[test]
    fn test_default_epigenetic_age_zero() {
        let s = TissueState::default();
        assert_eq!(s.epigenetic_age, 0.0);
    }

    #[test]
    fn test_default_telomere_full() {
        let s = TissueState::default();
        assert!((s.telomere_length - 1.0).abs() < 1e-9);
    }

    // ── new() ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_new_sets_age() {
        let s = TissueState::new(45.0);
        assert!((s.age_years - 45.0).abs() < 1e-9);
    }

    #[test]
    fn test_new_sets_epigenetic_age_equal_to_age() {
        let s = TissueState::new(30.0);
        assert!((s.epigenetic_age - 30.0).abs() < 1e-9);
    }

    #[test]
    fn test_new_stem_pool_still_full() {
        let s = TissueState::new(80.0);
        assert!((s.stem_cell_pool - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_new_mcai_zero_regardless_of_age() {
        let s = TissueState::new(90.0);
        assert_eq!(s.mcai, 0.0);
    }

    #[test]
    fn test_new_division_count_zero() {
        let s = TissueState::new(50.0);
        assert_eq!(s.division_count, 0);
    }

    #[test]
    fn test_new_zero_age() {
        let s = TissueState::new(0.0);
        assert_eq!(s.age_years, 0.0);
        assert_eq!(s.epigenetic_age, 0.0);
    }

    // ── is_viable ─────────────────────────────────────────────────────────────

    #[test]
    fn test_viable_default_state() {
        let s = TissueState::default();
        assert!(s.is_viable(), "Default state should be viable");
    }

    #[test]
    fn test_not_viable_when_pool_depleted() {
        let mut s = TissueState::default();
        s.stem_cell_pool = 0.04;
        assert!(!s.is_viable(), "Pool <= 0.05 should be non-viable");
    }

    #[test]
    fn test_not_viable_pool_exactly_zero() {
        let mut s = TissueState::default();
        s.stem_cell_pool = 0.0;
        assert!(!s.is_viable());
    }

    #[test]
    fn test_viable_pool_boundary_just_above() {
        let mut s = TissueState::default();
        s.stem_cell_pool = 0.051;
        assert!(s.is_viable(), "Pool just above 0.05 should be viable");
    }

    #[test]
    fn test_viable_pool_boundary_just_below() {
        let mut s = TissueState::default();
        s.stem_cell_pool = 0.049;
        assert!(!s.is_viable(), "Pool just below 0.05 should be non-viable");
    }

    #[test]
    fn test_not_viable_when_mcai_high() {
        let mut s = TissueState::default();
        s.mcai = 0.96;
        assert!(!s.is_viable(), "mcai >= 0.95 should be non-viable");
    }

    #[test]
    fn test_viable_mcai_just_below_threshold() {
        let mut s = TissueState::default();
        s.mcai = 0.94;
        assert!(s.is_viable(), "mcai < 0.95 with good pool must be viable");
    }

    #[test]
    fn test_not_viable_mcai_exactly_at_threshold() {
        let mut s = TissueState::default();
        s.mcai = 0.95;
        assert!(!s.is_viable(), "mcai = 0.95 should be non-viable (strict less than)");
    }

    #[test]
    fn test_not_viable_both_conditions_bad() {
        let mut s = TissueState::default();
        s.stem_cell_pool = 0.01;
        s.mcai = 0.99;
        assert!(!s.is_viable());
    }

    #[test]
    fn test_new_at_any_age_viable_initially() {
        for age in [0.0, 20.0, 40.0, 60.0, 80.0, 100.0] {
            let s = TissueState::new(age);
            assert!(s.is_viable(), "New tissue at age {} should be viable", age);
        }
    }

    // ── Biological constraints ─────────────────────────────────────────────────

    #[test]
    fn test_mcai_in_range_zero_to_one() {
        let s = TissueState::default();
        assert!(s.mcai >= 0.0 && s.mcai <= 1.0);
    }

    #[test]
    fn test_stem_cell_pool_non_negative_default() {
        let s = TissueState::default();
        assert!(s.stem_cell_pool >= 0.0);
    }

    #[test]
    fn test_telomere_length_non_negative() {
        let s = TissueState::default();
        assert!(s.telomere_length >= 0.0);
    }

    #[test]
    fn test_default_differentiated_telomere_full() {
        let s = TissueState::default();
        assert!((s.differentiated_telomere_length - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_differentiated_telomere_non_negative() {
        let s = TissueState::default();
        assert!(s.differentiated_telomere_length >= 0.0);
    }

    #[test]
    fn test_centriole_damage_non_negative_default() {
        let s = TissueState::default();
        assert!(s.centriole_damage >= 0.0);
    }

    // ── Clone / Debug ─────────────────────────────────────────────────────────

    #[test]
    fn test_clone_is_independent() {
        let s1 = TissueState::new(30.0);
        let mut s2 = s1.clone();
        s2.age_years = 50.0;
        assert!((s1.age_years - 30.0).abs() < 1e-9, "Clone must not affect original");
    }

    #[test]
    fn test_debug_output_exists() {
        let s = TissueState::default();
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("TissueState"));
    }

    // ── current_o2_percent ────────────────────────────────────────────────────

    #[test]
    fn test_default_o2_is_normoxia() {
        let s = TissueState::default();
        assert!((s.current_o2_percent - 21.0).abs() < 1e-9,
            "Default O₂ must be 21.0% (normoxia / standard culture)");
    }

    #[test]
    fn test_new_o2_inherits_normoxia() {
        let s = TissueState::new(40.0);
        assert!((s.current_o2_percent - 21.0).abs() < 1e-9);
    }

    #[test]
    fn test_new_with_o2_sets_hypoxia() {
        let s = TissueState::new_with_o2(0.0, 2.0);
        assert!((s.current_o2_percent - 2.0).abs() < 1e-9,
            "new_with_o2(2.0) must produce HSC-niche hypoxia");
    }

    #[test]
    fn test_new_with_o2_preserves_age() {
        let s = TissueState::new_with_o2(35.0, 3.0);
        assert!((s.age_years - 35.0).abs() < 1e-9);
    }

    #[test]
    fn test_o2_can_be_set_directly() {
        let mut s = TissueState::default();
        s.current_o2_percent = 1.0;
        assert!((s.current_o2_percent - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_o2_zero_is_valid_extreme() {
        let s = TissueState::new_with_o2(0.0, 0.0);
        assert!(s.current_o2_percent.is_finite());
    }
}
