use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TissueType {
    Hematopoietic,
    Intestinal,
    Muscle,
    Neural,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TissueSpecificParams {
    pub tissue_type: TissueType,
    pub base_division_rate: f64,
    pub damage_per_division_multiplier: f64,
    pub centriole_repair_efficiency: f64,
    pub sasp_sensitivity: f64,
    pub regenerative_potential: f64,
    pub tolerance: f64,
}

impl TissueSpecificParams {
    pub fn for_tissue(tissue: TissueType) -> Self {
        match tissue {
            TissueType::Hematopoietic => Self {
                tissue_type: TissueType::Hematopoietic,
                base_division_rate: 12.0,
                damage_per_division_multiplier: 1.0,
                centriole_repair_efficiency: 0.7,
                sasp_sensitivity: 1.0,
                regenerative_potential: 0.8,
                tolerance: 0.3,
            },
            TissueType::Intestinal => Self {
                tissue_type: TissueType::Intestinal,
                base_division_rate: 70.0,
                damage_per_division_multiplier: 0.3,
                centriole_repair_efficiency: 0.9,
                sasp_sensitivity: 0.6,
                regenerative_potential: 0.95,
                tolerance: 0.8,
            },
            TissueType::Muscle => Self {
                tissue_type: TissueType::Muscle,
                base_division_rate: 4.0,
                damage_per_division_multiplier: 1.2,
                centriole_repair_efficiency: 0.6,
                sasp_sensitivity: 0.8,
                regenerative_potential: 0.5,
                tolerance: 0.5,
            },
            TissueType::Neural => Self {
                tissue_type: TissueType::Neural,
                base_division_rate: 2.0,
                damage_per_division_multiplier: 1.5,
                centriole_repair_efficiency: 0.4,
                sasp_sensitivity: 1.2,
                regenerative_potential: 0.2,
                tolerance: 0.2,
            },
        }
    }

    pub fn effective_division_rate(&self, age_factor: f64, sasp_factor: f64) -> f64 {
        self.base_division_rate * age_factor * sasp_factor * self.regenerative_potential
    }

    /// Damage multiplier at a given age.
    /// tolerance = "protective fraction" [0,1]: higher → less net damage per division.
    /// FIXED Round 6: was /tolerance (denominator → explosion); now ×(1-tolerance).
    pub fn damage_accumulation_multiplier(&self, age_years: f64) -> f64 {
        let age_effect = 1.0 + age_years / 100.0;
        self.damage_per_division_multiplier * age_effect * (1.0 - self.tolerance)
    }

    /// Relative effective aging rate: ν × β × (1 - tolerance).
    /// HSC: 12×1.0×0.7 = 8.4  >  ISC: 70×0.3×0.2 = 4.2  (intestinal paradox resolved)
    pub fn effective_aging_rate(&self) -> f64 {
        self.base_division_rate * self.damage_per_division_multiplier * (1.0 - self.tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsc_params() {
        let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        assert!((hsc.base_division_rate - 12.0).abs() < 1e-6);
        assert!((hsc.tolerance - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_isc_params() {
        let isc = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        assert!((isc.base_division_rate - 70.0).abs() < 1e-6);
        assert!((isc.tolerance - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_effective_aging_rates() {
        let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        let isc = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        // FIXED Round 6: formula = ν × β × (1 - tolerance)
        // HSC: 12×1.0×(1-0.3) = 8.4 > ISC: 70×0.3×(1-0.8) = 4.2
        // Intestinal paradox preserved: despite 6× more divisions, ISC ages slower
        assert!(
            hsc.effective_aging_rate() > isc.effective_aging_rate(),
            "HSC ({:.2}) must age faster than ISC ({:.2})",
            hsc.effective_aging_rate(), isc.effective_aging_rate()
        );
        // Verify concrete values
        assert!((hsc.effective_aging_rate() - 8.4).abs() < 0.01);
        assert!((isc.effective_aging_rate() - 4.2).abs() < 0.01);
    }

    #[test]
    fn test_all_tissues() {
        for tissue in [TissueType::Hematopoietic, TissueType::Intestinal,
                       TissueType::Muscle, TissueType::Neural] {
            let p = TissueSpecificParams::for_tissue(tissue);
            assert!(p.base_division_rate > 0.0);
            assert!(p.tolerance > 0.0 && p.tolerance <= 1.0);
        }
    }

    // ── Tissue ordering: effective_aging_rate ─────────────────────────────────

    #[test]
    fn test_hsc_effective_aging_rate_greater_than_isc() {
        let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        let isc = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        assert!(hsc.effective_aging_rate() > isc.effective_aging_rate(),
            "HSC ({:.3}) must age faster than ISC ({:.3}) — intestinal paradox",
            hsc.effective_aging_rate(), isc.effective_aging_rate());
    }

    #[test]
    fn test_hsc_effective_aging_rate_greater_than_muscle() {
        let hsc    = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        let muscle = TissueSpecificParams::for_tissue(TissueType::Muscle);
        assert!(hsc.effective_aging_rate() > muscle.effective_aging_rate(),
            "HSC ({:.3}) > Muscle ({:.3})",
            hsc.effective_aging_rate(), muscle.effective_aging_rate());
    }

    #[test]
    fn test_hsc_effective_aging_rate_greater_than_neural() {
        let hsc    = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        let neural = TissueSpecificParams::for_tissue(TissueType::Neural);
        assert!(hsc.effective_aging_rate() > neural.effective_aging_rate(),
            "HSC ({:.3}) > Neural ({:.3})",
            hsc.effective_aging_rate(), neural.effective_aging_rate());
    }

    #[test]
    fn test_isc_effective_aging_rate_greater_than_neural() {
        let isc    = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        let neural = TissueSpecificParams::for_tissue(TissueType::Neural);
        assert!(isc.effective_aging_rate() > neural.effective_aging_rate(),
            "ISC ({:.3}) > Neural ({:.3})",
            isc.effective_aging_rate(), neural.effective_aging_rate());
    }

    #[test]
    fn test_isc_effective_aging_rate_greater_than_muscle() {
        let isc    = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        let muscle = TissueSpecificParams::for_tissue(TissueType::Muscle);
        assert!(isc.effective_aging_rate() > muscle.effective_aging_rate(),
            "ISC ({:.3}) > Muscle ({:.3})",
            isc.effective_aging_rate(), muscle.effective_aging_rate());
    }

    #[test]
    fn test_effective_aging_rate_concrete_hsc() {
        // HSC: 12 × 1.0 × (1 - 0.3) = 8.4
        let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        assert!((hsc.effective_aging_rate() - 8.4).abs() < 0.001,
            "HSC effective aging rate should be 8.4, got {}", hsc.effective_aging_rate());
    }

    #[test]
    fn test_effective_aging_rate_concrete_isc() {
        // ISC: 70 × 0.3 × (1 - 0.8) = 4.2
        let isc = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        assert!((isc.effective_aging_rate() - 4.2).abs() < 0.001,
            "ISC effective aging rate should be 4.2, got {}", isc.effective_aging_rate());
    }

    #[test]
    fn test_effective_aging_rate_concrete_muscle() {
        // Muscle: 4.0 × 1.2 × (1 - 0.5) = 2.4
        let muscle = TissueSpecificParams::for_tissue(TissueType::Muscle);
        assert!((muscle.effective_aging_rate() - 2.4).abs() < 0.001,
            "Muscle effective aging rate should be 2.4, got {}", muscle.effective_aging_rate());
    }

    #[test]
    fn test_effective_aging_rate_concrete_neural() {
        // Neural: 2.0 × 1.5 × (1 - 0.2) = 2.4
        let neural = TissueSpecificParams::for_tissue(TissueType::Neural);
        assert!((neural.effective_aging_rate() - 2.4).abs() < 0.001,
            "Neural effective aging rate should be 2.4, got {}", neural.effective_aging_rate());
    }

    // ── effective_division_rate ───────────────────────────────────────────────

    #[test]
    fn test_effective_division_rate_positive() {
        let tissues = [TissueType::Hematopoietic, TissueType::Intestinal,
                       TissueType::Muscle, TissueType::Neural];
        for t in tissues {
            let p = TissueSpecificParams::for_tissue(t);
            let rate = p.effective_division_rate(1.0, 1.0);
            assert!(rate > 0.0, "effective_division_rate must be positive");
        }
    }

    #[test]
    fn test_effective_division_rate_scales_with_age_factor() {
        let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        let r1 = hsc.effective_division_rate(0.5, 1.0);
        let r2 = hsc.effective_division_rate(1.0, 1.0);
        assert!((r2 / r1 - 2.0).abs() < 1e-6, "rate should scale linearly with age_factor");
    }

    #[test]
    fn test_effective_division_rate_scales_with_sasp_factor() {
        let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        let r1 = hsc.effective_division_rate(1.0, 0.5);
        let r2 = hsc.effective_division_rate(1.0, 1.0);
        assert!((r2 / r1 - 2.0).abs() < 1e-6, "rate should scale linearly with sasp_factor");
    }

    #[test]
    fn test_effective_division_rate_zero_age_factor_zero() {
        let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        let r = hsc.effective_division_rate(0.0, 1.0);
        assert_eq!(r, 0.0);
    }

    // ── damage_accumulation_multiplier ────────────────────────────────────────

    #[test]
    fn test_damage_multiplier_positive() {
        let tissues = [TissueType::Hematopoietic, TissueType::Intestinal,
                       TissueType::Muscle, TissueType::Neural];
        for t in tissues {
            let p = TissueSpecificParams::for_tissue(t);
            assert!(p.damage_accumulation_multiplier(30.0) > 0.0);
        }
    }

    #[test]
    fn test_damage_multiplier_increases_with_age() {
        let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        let m_young = hsc.damage_accumulation_multiplier(20.0);
        let m_old   = hsc.damage_accumulation_multiplier(80.0);
        assert!(m_old > m_young, "Damage multiplier increases with age");
    }

    #[test]
    fn test_neural_high_damage_multiplier() {
        // Neural: β=1.5, age_effect at 0 = 1.0, (1-tolerance)=0.8 → 1.5 * 1.0 * 0.8 = 1.2
        let neural = TissueSpecificParams::for_tissue(TissueType::Neural);
        let m = neural.damage_accumulation_multiplier(0.0);
        assert!((m - 1.2).abs() < 0.001, "Neural damage multiplier at age 0 = 1.2, got {}", m);
    }

    #[test]
    fn test_isc_low_damage_multiplier() {
        // ISC: β=0.3, age_effect at 0 = 1.0, (1-tolerance)=0.2 → 0.3 * 1.0 * 0.2 = 0.06
        let isc = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        let m = isc.damage_accumulation_multiplier(0.0);
        assert!((m - 0.06).abs() < 0.001, "ISC damage multiplier at age 0 = 0.06, got {}", m);
    }

    // ── TissueSpecificParams field ranges ─────────────────────────────────────

    #[test]
    fn test_all_regenerative_potential_in_range() {
        let tissues = [TissueType::Hematopoietic, TissueType::Intestinal,
                       TissueType::Muscle, TissueType::Neural];
        for t in tissues {
            let p = TissueSpecificParams::for_tissue(t);
            assert!(p.regenerative_potential >= 0.0 && p.regenerative_potential <= 1.0,
                "regenerative_potential out of range for {:?}: {}", p.tissue_type, p.regenerative_potential);
        }
    }

    #[test]
    fn test_all_sasp_sensitivity_positive() {
        let tissues = [TissueType::Hematopoietic, TissueType::Intestinal,
                       TissueType::Muscle, TissueType::Neural];
        for t in tissues {
            let p = TissueSpecificParams::for_tissue(t);
            assert!(p.sasp_sensitivity > 0.0);
        }
    }

    #[test]
    fn test_all_centriole_repair_in_range() {
        let tissues = [TissueType::Hematopoietic, TissueType::Intestinal,
                       TissueType::Muscle, TissueType::Neural];
        for t in tissues {
            let p = TissueSpecificParams::for_tissue(t);
            assert!(p.centriole_repair_efficiency >= 0.0 && p.centriole_repair_efficiency <= 1.0);
        }
    }

    #[test]
    fn test_isc_highest_regenerative_potential() {
        let isc    = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        let neural = TissueSpecificParams::for_tissue(TissueType::Neural);
        assert!(isc.regenerative_potential > neural.regenerative_potential,
            "ISC should have higher regenerative potential than Neural");
    }

    #[test]
    fn test_neural_lowest_regenerative_potential() {
        let tissues = [TissueType::Hematopoietic, TissueType::Intestinal, TissueType::Muscle];
        let neural = TissueSpecificParams::for_tissue(TissueType::Neural);
        for t in tissues {
            let p = TissueSpecificParams::for_tissue(t);
            assert!(p.regenerative_potential > neural.regenerative_potential,
                "{:?} should have higher regenerative potential than Neural", p.tissue_type);
        }
    }

    #[test]
    fn test_neural_highest_sasp_sensitivity() {
        let neural = TissueSpecificParams::for_tissue(TissueType::Neural);
        let isc    = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        assert!(neural.sasp_sensitivity > isc.sasp_sensitivity,
            "Neural should be most SASP-sensitive");
    }

    #[test]
    fn test_tissue_type_roundtrip() {
        let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
        assert_eq!(hsc.tissue_type, TissueType::Hematopoietic);
        let isc = TissueSpecificParams::for_tissue(TissueType::Intestinal);
        assert_eq!(isc.tissue_type, TissueType::Intestinal);
    }
}
