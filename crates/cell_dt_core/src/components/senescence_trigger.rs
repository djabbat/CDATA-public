/// Dual-clock senescence trigger (CDATA v3.5; updated v4.6).
///
/// Replicative senescence can be triggered by either molecular clock independently:
///   1. Centriolar damage: D(t) ≥ D_crit (the primary CDATA clock)
///   2. Telomere shortening: TL(n) ≤ TL_crit (classical Hayflick mechanism)
///
/// ## Scientific basis (CDATA v4.6)
///
/// **C3 — Centriolar dysfunction directly induces p16-senescence:**
/// - SVBP/VASH pathway: biallelic SVBP variant (p.Leu49Pro) → centrosome cohesion
///   abnormalities → p16^INK4a ×3.4 in patient PBMCs → premature senescence
///   (Launay et al. 2025, Aging Cell 24:e14355, PMID 39412222).
/// - PLK4-inhibition pathway: prolonged PLK4 inhibition → centriole loss →
///   senescence, polyploidy, defective cytokinesis
///   (Dang et al. 2023, Blood 142:2002; Hamzah et al. 2025, Cytoskeleton, PMID 40257113).
/// - Stem cells in telomerase-active niches still exhibit finite replicative
///   lifespan (Peters-Hall et al. 2020: >200 PD at 2% O₂) — centriolar clock.
/// - Somatic differentiated cells: both clocks active; whichever reaches
///   threshold first triggers p16/p21 pathway and permanent cell cycle arrest.
///
/// Reference: Tkemaladze & Chichinadze (2005) proposed centrosome-driven
/// replicative aging. CDATA v3.5 formalises the dual-clock model.
use serde::{Deserialize, Serialize};

/// Which molecular clock triggered senescence onset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SenescenceTrigger {
    /// D(t) reached D_crit (1000 a.u.).
    /// Primary mechanism in stem cells / telomerase-positive niches.
    CentriolarDamage,
    /// Telomere length fell below TL_crit (≈5 kb for human fibroblasts).
    /// Primary mechanism in differentiated somatic cells.
    TelomereShortening,
    /// Both thresholds reached simultaneously (rare; seen in very old cells).
    Both,
    /// Cell has not yet reached senescence.
    None,
}

impl SenescenceTrigger {
    /// Determine which clock triggered given current damage and telomere values.
    ///
    /// Thresholds:
    /// - `d_crit`: default 1000.0 a.u. (normalised)
    /// - `tl_crit`: default 5.0 kb (human fibroblast; adjust per cell type)
    pub fn evaluate(
        damage_normalized: f64,
        d_crit: f64,
        telomere_kb: f64,
        tl_crit: f64,
    ) -> Self {
        let centriolar = damage_normalized >= d_crit;
        let telomere = telomere_kb <= tl_crit;
        match (centriolar, telomere) {
            (true, true)   => SenescenceTrigger::Both,
            (true, false)  => SenescenceTrigger::CentriolarDamage,
            (false, true)  => SenescenceTrigger::TelomereShortening,
            (false, false) => SenescenceTrigger::None,
        }
    }

    /// Returns true if any senescence has been triggered.
    pub fn is_senescent(&self) -> bool {
        !matches!(self, SenescenceTrigger::None)
    }

    /// Returns the dominant clock (for logging/reporting).
    /// CentriolarDamage is returned for Both (to preserve CDATA primary narrative).
    pub fn dominant_clock(&self) -> Option<&'static str> {
        match self {
            SenescenceTrigger::CentriolarDamage => Some("centriolar"),
            SenescenceTrigger::TelomereShortening => Some("telomere"),
            SenescenceTrigger::Both => Some("centriolar"), // CDATA dominant
            SenescenceTrigger::None => None,
        }
    }
}

impl std::fmt::Display for SenescenceTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CentriolarDamage    => write!(f, "CentriolarDamage"),
            Self::TelomereShortening  => write!(f, "TelomereShortening"),
            Self::Both                => write!(f, "Both"),
            Self::None                => write!(f, "None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const D_CRIT: f64 = 1000.0;
    const TL_CRIT: f64 = 5.0;

    #[test]
    fn test_no_senescence_below_both_thresholds() {
        let t = SenescenceTrigger::evaluate(500.0, D_CRIT, 8.0, TL_CRIT);
        assert_eq!(t, SenescenceTrigger::None);
        assert!(!t.is_senescent());
    }

    #[test]
    fn test_centriolar_trigger_only() {
        let t = SenescenceTrigger::evaluate(1000.0, D_CRIT, 8.0, TL_CRIT);
        assert_eq!(t, SenescenceTrigger::CentriolarDamage);
        assert!(t.is_senescent());
        assert_eq!(t.dominant_clock(), Some("centriolar"));
    }

    #[test]
    fn test_telomere_trigger_only() {
        let t = SenescenceTrigger::evaluate(500.0, D_CRIT, 4.9, TL_CRIT);
        assert_eq!(t, SenescenceTrigger::TelomereShortening);
        assert!(t.is_senescent());
        assert_eq!(t.dominant_clock(), Some("telomere"));
    }

    #[test]
    fn test_both_triggered() {
        let t = SenescenceTrigger::evaluate(1001.0, D_CRIT, 4.0, TL_CRIT);
        assert_eq!(t, SenescenceTrigger::Both);
        assert!(t.is_senescent());
        assert_eq!(t.dominant_clock(), Some("centriolar")); // CDATA dominant
    }

    #[test]
    fn test_boundary_damage_exactly_at_crit() {
        // d == d_crit triggers (>=)
        let t = SenescenceTrigger::evaluate(D_CRIT, D_CRIT, 8.0, TL_CRIT);
        assert_eq!(t, SenescenceTrigger::CentriolarDamage);
    }

    #[test]
    fn test_boundary_telomere_exactly_at_crit() {
        // tl == tl_crit triggers (<=)
        let t = SenescenceTrigger::evaluate(500.0, D_CRIT, TL_CRIT, TL_CRIT);
        assert_eq!(t, SenescenceTrigger::TelomereShortening);
    }

    #[test]
    fn test_display_none() {
        assert_eq!(format!("{}", SenescenceTrigger::None), "None");
    }

    #[test]
    fn test_display_both() {
        assert_eq!(format!("{}", SenescenceTrigger::Both), "Both");
    }

    #[test]
    fn test_clone_eq() {
        let t = SenescenceTrigger::CentriolarDamage;
        assert_eq!(t, t.clone());
    }
}
