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
