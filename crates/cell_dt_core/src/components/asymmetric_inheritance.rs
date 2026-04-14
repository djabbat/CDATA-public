use serde::{Deserialize, Serialize};

/// Asymmetric centriole inheritance state for a stem cell.
///
/// ## Experimental basis (CDATA v4.6)
///
/// **C2 — Directed maternal centrosome inheritance in mammals:**
/// - Human neural progenitor cells: ~80% of self-renewing daughters inherit the older centrosome
///   (Royall et al. 2023, eLife 12:e83157). Ninein-dependent.
/// - Murine CD8+ T cells: >90% of first asymmetric divisions direct the mother centrosome
///   to the effector-fated daughter (Barandun & Oxenius 2025, Cell Reports, PMID 39764850).
///   Ninein deletion abolishes directed inheritance.
/// - **Ninein** is the conserved molecular mediator in both systems.
///
/// ## Model parameters
/// - `inheritance_probability`: P(maternal centrosome → stem daughter).
///   Default 0.94 consistent with empirical range [0.80–0.90+] across cell types.
/// - `ninein_activity`: Fractional Ninein activity (0 = absent → random; 1 = full → directed).
///   Tracks the molecular mediator identified by Royall 2023 + Barandun 2025.
///   Default 1.0 (healthy young cell). Declines with D(t) via PCM deterioration.
///   NOTE: ninein_activity is a *computed observable*, not a FixedParameter — does not
///   count toward the 32-parameter budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsymmetricInheritance {
    pub inheritance_probability: f64,
    pub inherited_maternal_last: bool,
    pub total_divisions: u64,
    pub maternal_inheritance_count: u64,
    /// Ninein activity as molecular mediator of directed centrosome segregation.
    /// Range [0.0, 1.0]. 0.0 = random segregation; 1.0 = fully directed.
    /// Empirical support: Royall 2023 (NPCs), Barandun & Oxenius 2025 (CD8+ T cells).
    pub ninein_activity: f64,
}

impl Default for AsymmetricInheritance {
    fn default() -> Self {
        Self {
            inheritance_probability: 0.94,
            inherited_maternal_last: true,
            total_divisions: 0,
            maternal_inheritance_count: 0,
            ninein_activity: 1.0,
        }
    }
}

impl AsymmetricInheritance {
    pub fn asymmetry_fraction(&self) -> f64 {
        if self.total_divisions == 0 { return 0.0; }
        self.maternal_inheritance_count as f64 / self.total_divisions as f64
    }

    /// Effective inheritance probability accounting for Ninein activity.
    ///
    /// When Ninein is absent (ninein_activity=0.0), p_eff = 0.5 (random segregation).
    /// When Ninein is fully active (ninein_activity=1.0), p_eff = inheritance_probability.
    /// Linear interpolation: p_eff = 0.5 + (p - 0.5) * ninein_activity
    ///
    /// Empirical basis:
    /// - Ninein KO → complete randomization of centrosome inheritance in human NPCs
    ///   (Royall et al. 2023, eLife 12:e83157)
    /// - Ninein deletion abolishes directed inheritance in murine CD8+ T cells
    ///   (Barandun & Oxenius 2025, Cell Reports, PMID 39764850)
    pub fn effective_probability(&self) -> f64 {
        let p = self.inheritance_probability;
        (0.5 + (p - 0.5) * self.ninein_activity).clamp(0.5, 0.98)
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

    // ── ninein_activity and effective_probability ──────────────────────────────

    #[test]
    fn test_default_ninein_activity_is_one() {
        let a = AsymmetricInheritance::default();
        assert!((a.ninein_activity - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_effective_probability_full_ninein_equals_base() {
        let a = AsymmetricInheritance::default(); // ninein=1.0, p=0.94
        let eff = a.effective_probability();
        assert!((eff - 0.94).abs() < 1e-9,
            "Full Ninein activity → p_eff should equal inheritance_probability, got {}", eff);
    }

    #[test]
    fn test_effective_probability_zero_ninein_is_random() {
        let mut a = AsymmetricInheritance::default();
        a.ninein_activity = 0.0;
        let eff = a.effective_probability();
        assert!((eff - 0.5).abs() < 1e-9,
            "Zero Ninein activity → p_eff should be 0.5 (random), got {}", eff);
    }

    #[test]
    fn test_effective_probability_half_ninein() {
        let mut a = AsymmetricInheritance::default(); // p=0.94
        a.ninein_activity = 0.5;
        let eff = a.effective_probability();
        // p_eff = 0.5 + (0.94 - 0.5) * 0.5 = 0.5 + 0.22 = 0.72
        assert!((eff - 0.72).abs() < 1e-9,
            "Half Ninein → p_eff=0.72, got {}", eff);
    }

    #[test]
    fn test_effective_probability_clamp_upper() {
        let mut a = AsymmetricInheritance::default();
        a.inheritance_probability = 1.0;
        a.ninein_activity = 1.0;
        let eff = a.effective_probability();
        assert!(eff <= 0.98, "Effective probability must not exceed 0.98");
    }

    #[test]
    fn test_effective_probability_clamp_lower() {
        let mut a = AsymmetricInheritance::default();
        a.inheritance_probability = 0.0;
        a.ninein_activity = 0.0;
        let eff = a.effective_probability();
        assert!(eff >= 0.5, "Effective probability must not go below 0.5");
    }

    #[test]
    fn test_effective_probability_decreases_with_ninein_loss() {
        let a_full = AsymmetricInheritance::default();
        let mut a_half = AsymmetricInheritance::default();
        a_half.ninein_activity = 0.5;
        assert!(a_full.effective_probability() > a_half.effective_probability(),
            "Lower Ninein activity must reduce effective probability");
    }
}
