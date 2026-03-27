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
