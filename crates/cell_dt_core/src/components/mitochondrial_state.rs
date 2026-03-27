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
