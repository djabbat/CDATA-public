use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitochondrialState {
    pub mtdna_mutations: f64,
    pub ros_level: f64,
    pub mito_shield: f64,
    pub mitophagy_efficiency: f64,
    pub membrane_potential: f64,
    pub fusion_frequency: f64,
    pub base_ros: f64,
}

impl Default for MitochondrialState {
    fn default() -> Self {
        Self {
            mtdna_mutations: 0.0,
            ros_level: 0.12,
            mito_shield: 1.0,
            mitophagy_efficiency: 1.0,
            membrane_potential: 1.0,
            fusion_frequency: 1.0,
            base_ros: 0.12,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mtdna_zero() {
        let s = MitochondrialState::default();
        assert_eq!(s.mtdna_mutations, 0.0);
    }

    #[test]
    fn test_default_ros_young() {
        let s = MitochondrialState::default();
        assert!((s.ros_level - 0.12).abs() < 1e-9, "Young ROS = 0.12");
    }

    #[test]
    fn test_default_mito_shield_one() {
        let s = MitochondrialState::default();
        assert!((s.mito_shield - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_default_mitophagy_full() {
        let s = MitochondrialState::default();
        assert!((s.mitophagy_efficiency - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_default_membrane_potential_max() {
        let s = MitochondrialState::default();
        assert!((s.membrane_potential - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_default_fusion_full() {
        let s = MitochondrialState::default();
        assert!((s.fusion_frequency - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_base_ros_equals_ros_young() {
        let s = MitochondrialState::default();
        assert!((s.base_ros - s.ros_level).abs() < 1e-9);
    }

    #[test]
    fn test_clone_independent() {
        let s1 = MitochondrialState::default();
        let mut s2 = s1.clone();
        s2.ros_level = 0.9;
        assert!((s1.ros_level - 0.12).abs() < 1e-9);
    }

    #[test]
    fn test_debug_output() {
        let s = MitochondrialState::default();
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("MitochondrialState"));
    }

    #[test]
    fn test_all_defaults_non_negative() {
        let s = MitochondrialState::default();
        assert!(s.mtdna_mutations >= 0.0);
        assert!(s.ros_level >= 0.0);
        assert!(s.mito_shield >= 0.0);
        assert!(s.mitophagy_efficiency >= 0.0);
        assert!(s.membrane_potential >= 0.0);
        assert!(s.fusion_frequency >= 0.0);
    }
}
