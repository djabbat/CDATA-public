use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChipState {
    pub total_chip_frequency: f64,
    pub dnmt3a_frequency: f64,
    pub tet2_frequency: f64,
    pub dominant_clone_size: f64,
    pub detection_age: Option<f64>,
    pub hematologic_risk: f64,
}

impl Default for ChipState {
    fn default() -> Self {
        Self {
            total_chip_frequency: 0.0,
            dnmt3a_frequency: 0.0,
            tet2_frequency: 0.0,
            dominant_clone_size: 0.0,
            detection_age: None,
            hematologic_risk: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_all_zero() {
        let s = ChipState::default();
        assert_eq!(s.total_chip_frequency, 0.0);
        assert_eq!(s.dnmt3a_frequency, 0.0);
        assert_eq!(s.tet2_frequency, 0.0);
        assert_eq!(s.dominant_clone_size, 0.0);
        assert_eq!(s.hematologic_risk, 0.0);
    }

    #[test]
    fn test_default_no_detection_age() {
        let s = ChipState::default();
        assert!(s.detection_age.is_none());
    }

    #[test]
    fn test_clone_independent() {
        let s1 = ChipState::default();
        let mut s2 = s1.clone();
        s2.dnmt3a_frequency = 0.1;
        assert_eq!(s1.dnmt3a_frequency, 0.0);
    }

    #[test]
    fn test_debug_output() {
        let s = ChipState::default();
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("ChipState"));
    }

    #[test]
    fn test_detection_age_can_be_set() {
        let mut s = ChipState::default();
        s.detection_age = Some(65.0);
        assert_eq!(s.detection_age, Some(65.0));
    }

    #[test]
    fn test_chip_frequencies_non_negative_default() {
        let s = ChipState::default();
        assert!(s.dnmt3a_frequency >= 0.0);
        assert!(s.tet2_frequency >= 0.0);
        assert!(s.total_chip_frequency >= 0.0);
    }
}
