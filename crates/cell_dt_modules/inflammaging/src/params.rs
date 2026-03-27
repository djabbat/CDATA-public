use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflammagingParams {
    pub damps_rate: f64,
    pub cgas_sensitivity: f64,
    pub sasp_decay: f64,
    pub nk_age_decay: f64,
    pub fibrosis_rate: f64,
}

impl Default for InflammagingParams {
    fn default() -> Self {
        Self {
            damps_rate: 0.05,
            cgas_sensitivity: 0.8,
            sasp_decay: 0.1,
            nk_age_decay: 0.005,
            fibrosis_rate: 0.02,
        }
    }
}

pub fn sasp_to_ros_contribution(sasp_level: f64) -> f64 {
    sasp_level * 0.3
}

pub fn sasp_damage_multiplier(sasp_level: f64) -> f64 {
    1.0 + sasp_level * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sasp_ros_feedback() {
        let ros = sasp_to_ros_contribution(0.5);
        assert!(ros > 0.0 && ros < 0.5);
    }

    #[test]
    fn test_sasp_multiplier() {
        assert!((sasp_damage_multiplier(0.0) - 1.0).abs() < 1e-6);
        assert!(sasp_damage_multiplier(1.0) > 1.0);
    }
}
