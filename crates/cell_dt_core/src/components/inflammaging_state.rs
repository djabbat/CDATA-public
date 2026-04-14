use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflammagingState {
    pub sasp_level: f64,
    pub cgas_sting_activity: f64,
    pub damps_level: f64,
    pub nk_efficiency: f64,
    pub fibrosis_level: f64,
    pub senescent_cell_fraction: f64,
    pub nfkb_activity: f64,
}

impl Default for InflammagingState {
    fn default() -> Self {
        Self {
            sasp_level: 0.0,
            cgas_sting_activity: 0.0,
            damps_level: 0.0,
            nk_efficiency: 1.0,
            fibrosis_level: 0.0,
            senescent_cell_fraction: 0.0,
            nfkb_activity: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_sasp_zero() {
        let s = InflammagingState::default();
        assert_eq!(s.sasp_level, 0.0);
    }

    #[test]
    fn test_default_cgas_zero() {
        let s = InflammagingState::default();
        assert_eq!(s.cgas_sting_activity, 0.0);
    }

    #[test]
    fn test_default_damps_zero() {
        let s = InflammagingState::default();
        assert_eq!(s.damps_level, 0.0);
    }

    #[test]
    fn test_default_nk_efficiency_one() {
        let s = InflammagingState::default();
        assert!((s.nk_efficiency - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_default_fibrosis_zero() {
        let s = InflammagingState::default();
        assert_eq!(s.fibrosis_level, 0.0);
    }

    #[test]
    fn test_default_senescent_fraction_zero() {
        let s = InflammagingState::default();
        assert_eq!(s.senescent_cell_fraction, 0.0);
    }

    #[test]
    fn test_default_nfkb_basal() {
        let s = InflammagingState::default();
        assert!((s.nfkb_activity - 0.1).abs() < 1e-9, "Basal NF-kB should be 0.1");
    }

    #[test]
    fn test_all_defaults_non_negative() {
        let s = InflammagingState::default();
        assert!(s.sasp_level >= 0.0);
        assert!(s.cgas_sting_activity >= 0.0);
        assert!(s.damps_level >= 0.0);
        assert!(s.nk_efficiency >= 0.0);
        assert!(s.fibrosis_level >= 0.0);
        assert!(s.senescent_cell_fraction >= 0.0);
        assert!(s.nfkb_activity >= 0.0);
    }

    #[test]
    fn test_clone_independent() {
        let s1 = InflammagingState::default();
        let mut s2 = s1.clone();
        s2.sasp_level = 0.9;
        assert_eq!(s1.sasp_level, 0.0);
    }

    #[test]
    fn test_debug_output() {
        let s = InflammagingState::default();
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("InflammagingState"));
    }
}
