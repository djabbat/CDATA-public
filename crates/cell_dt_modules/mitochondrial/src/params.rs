use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitochondrialParams {
    pub mitophagy_threshold: f64,
    pub ros_steepness: f64,
    pub max_ros: f64,
    pub base_ros_young: f64,
    pub hormesis_factor: f64,
}

impl Default for MitochondrialParams {
    fn default() -> Self {
        Self {
            mitophagy_threshold: 0.35,
            ros_steepness: 10.0,
            max_ros: 1.0,
            base_ros_young: 0.12,
            hormesis_factor: 1.3,
        }
    }
}

pub fn sigmoid_ros(damage: f64, oxidative_input: f64, steepness: f64, threshold: f64) -> f64 {
    let x = damage + oxidative_input;
    1.0 / (1.0 + (-steepness * (x - threshold)).exp())
}

pub fn compute_mitophagy(ros_level: f64, age_years: f64, threshold: f64) -> f64 {
    if ros_level <= threshold {
        return 1.0;
    }
    let age_penalty = (age_years / 100.0).min(0.8);
    ((1.0 - age_penalty) * (1.0 - (ros_level - threshold))).max(0.0)
}

pub fn accumulate_mtdna(current: f64, ros_level: f64, dt: f64) -> f64 {
    (current + 0.001 * ros_level * ros_level * dt).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid_low_high() {
        assert!(sigmoid_ros(0.0, 0.0, 10.0, 0.35) < 0.5);
        assert!(sigmoid_ros(0.8, 0.2, 10.0, 0.35) > 0.5);
    }

    #[test]
    fn test_mitophagy_threshold() {
        assert!((compute_mitophagy(0.2, 30.0, 0.35) - 1.0).abs() < 1e-6);
        assert!(compute_mitophagy(0.6, 30.0, 0.35) < 1.0);
    }

    #[test]
    fn test_mtdna_accumulation() {
        let after = accumulate_mtdna(0.0, 0.5, 10.0);
        assert!(after > 0.0 && after <= 1.0);
    }
}
