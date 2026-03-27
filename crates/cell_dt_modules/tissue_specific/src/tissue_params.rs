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
}
