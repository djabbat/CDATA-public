use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouthProtection {
    pub current_level: f64,
    pub tert_activity: f64,
    pub foxo_activity: f64,
    pub sirt_activity: f64,
    pub nrf2_activity: f64,
    pub repair_efficiency: f64,
}

impl Default for YouthProtection {
    fn default() -> Self {
        Self {
            current_level: 1.0,
            tert_activity: 1.0,
            foxo_activity: 1.0,
            sirt_activity: 1.0,
            nrf2_activity: 1.0,
            repair_efficiency: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_all_one() {
        let y = YouthProtection::default();
        assert!((y.current_level  - 1.0).abs() < 1e-9);
        assert!((y.tert_activity  - 1.0).abs() < 1e-9);
        assert!((y.foxo_activity  - 1.0).abs() < 1e-9);
        assert!((y.sirt_activity  - 1.0).abs() < 1e-9);
        assert!((y.nrf2_activity  - 1.0).abs() < 1e-9);
        assert!((y.repair_efficiency - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_all_fields_non_negative() {
        let y = YouthProtection::default();
        assert!(y.current_level >= 0.0);
        assert!(y.tert_activity >= 0.0);
        assert!(y.foxo_activity >= 0.0);
        assert!(y.sirt_activity >= 0.0);
        assert!(y.nrf2_activity >= 0.0);
        assert!(y.repair_efficiency >= 0.0);
    }

    #[test]
    fn test_clone_independent() {
        let y1 = YouthProtection::default();
        let mut y2 = y1.clone();
        y2.current_level = 0.5;
        assert!((y1.current_level - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_debug_output() {
        let y = YouthProtection::default();
        let dbg = format!("{:?}", y);
        assert!(dbg.contains("YouthProtection"));
    }

    #[test]
    fn test_repair_efficiency_initially_maximal() {
        let y = YouthProtection::default();
        assert!((y.repair_efficiency - 1.0).abs() < 1e-9,
            "Repair efficiency should be maximal at initialization");
    }
}
