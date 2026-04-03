use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsymmetricInheritance {
    pub inheritance_probability: f64,
    pub inherited_maternal_last: bool,
    pub total_divisions: u64,
    pub maternal_inheritance_count: u64,
}

impl Default for AsymmetricInheritance {
    fn default() -> Self {
        Self {
            inheritance_probability: 0.94,
            inherited_maternal_last: true,
            total_divisions: 0,
            maternal_inheritance_count: 0,
        }
    }
}

impl AsymmetricInheritance {
    pub fn asymmetry_fraction(&self) -> f64 {
        if self.total_divisions == 0 { return 0.0; }
        self.maternal_inheritance_count as f64 / self.total_divisions as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_probability() {
        let a = AsymmetricInheritance::default();
        assert!((a.inheritance_probability - 0.94).abs() < 1e-9);
    }

    #[test]
    fn test_default_inherited_maternal_true() {
        let a = AsymmetricInheritance::default();
        assert!(a.inherited_maternal_last);
    }

    #[test]
    fn test_default_counts_zero() {
        let a = AsymmetricInheritance::default();
        assert_eq!(a.total_divisions, 0);
        assert_eq!(a.maternal_inheritance_count, 0);
    }

    #[test]
    fn test_asymmetry_fraction_zero_divisions() {
        let a = AsymmetricInheritance::default();
        assert_eq!(a.asymmetry_fraction(), 0.0);
    }

    #[test]
    fn test_asymmetry_fraction_all_maternal() {
        let mut a = AsymmetricInheritance::default();
        a.total_divisions = 10;
        a.maternal_inheritance_count = 10;
        assert!((a.asymmetry_fraction() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_asymmetry_fraction_none_maternal() {
        let mut a = AsymmetricInheritance::default();
        a.total_divisions = 10;
        a.maternal_inheritance_count = 0;
        assert_eq!(a.asymmetry_fraction(), 0.0);
    }

    #[test]
    fn test_asymmetry_fraction_partial() {
        let mut a = AsymmetricInheritance::default();
        a.total_divisions = 4;
        a.maternal_inheritance_count = 3;
        assert!((a.asymmetry_fraction() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_asymmetry_fraction_in_range() {
        let mut a = AsymmetricInheritance::default();
        a.total_divisions = 100;
        a.maternal_inheritance_count = 73;
        let frac = a.asymmetry_fraction();
        assert!(frac >= 0.0 && frac <= 1.0);
    }

    #[test]
    fn test_clone_independent() {
        let a1 = AsymmetricInheritance::default();
        let mut a2 = a1.clone();
        a2.total_divisions = 5;
        assert_eq!(a1.total_divisions, 0);
    }

    #[test]
    fn test_debug_output() {
        let a = AsymmetricInheritance::default();
        let dbg = format!("{:?}", a);
        assert!(dbg.contains("AsymmetricInheritance"));
    }
}
