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
