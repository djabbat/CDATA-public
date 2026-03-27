pub struct Calibrator {
    pub training_age_range: (f64, f64),
}

impl Calibrator {
    pub fn new() -> Self {
        Self { training_age_range: (20.0, 50.0) }
    }

    pub fn calculate_r2(observed: &[f64], predicted: &[f64]) -> f64 {
        if observed.len() != predicted.len() || observed.is_empty() { return 0.0; }
        let mean_obs: f64 = observed.iter().sum::<f64>() / observed.len() as f64;
        let ss_tot: f64 = observed.iter().map(|&o| (o - mean_obs).powi(2)).sum();
        let ss_res: f64 = observed.iter().zip(predicted.iter()).map(|(&o, &p)| (o - p).powi(2)).sum();
        if ss_tot < 1e-10 { return 1.0; }
        1.0 - ss_res / ss_tot
    }

    pub fn calculate_rmse(observed: &[f64], predicted: &[f64]) -> f64 {
        if observed.len() != predicted.len() || observed.is_empty() { return f64::INFINITY; }
        let mse = observed.iter().zip(predicted.iter())
            .map(|(&o, &p)| (o - p).powi(2)).sum::<f64>() / observed.len() as f64;
        mse.sqrt()
    }
}

impl Default for Calibrator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r2_perfect() {
        let v = vec![1.0, 2.0, 3.0, 4.0];
        assert!((Calibrator::calculate_r2(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_rmse_zero() {
        let v = vec![1.0, 2.0, 3.0];
        assert!(Calibrator::calculate_rmse(&v, &v) < 1e-6);
    }

    // ── Calibrator construction ───────────────────────────────────────────────

    #[test]
    fn test_default_training_range() {
        let c = Calibrator::new();
        assert!((c.training_age_range.0 - 20.0).abs() < 1e-9);
        assert!((c.training_age_range.1 - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_default_eq_new() {
        let c1 = Calibrator::new();
        let c2 = Calibrator::default();
        assert_eq!(c1.training_age_range, c2.training_age_range);
    }

    // ── calculate_r2 ──────────────────────────────────────────────────────────

    #[test]
    fn test_r2_empty_returns_zero() {
        let r2 = Calibrator::calculate_r2(&[], &[]);
        assert_eq!(r2, 0.0);
    }

    #[test]
    fn test_r2_mismatched_lengths_returns_zero() {
        let obs  = vec![1.0, 2.0, 3.0];
        let pred = vec![1.0, 2.0];
        assert_eq!(Calibrator::calculate_r2(&obs, &pred), 0.0);
    }

    #[test]
    fn test_r2_perfect_single_element() {
        let obs  = vec![5.0];
        let pred = vec![5.0];
        // ss_tot = 0 → returns 1.0
        assert!((Calibrator::calculate_r2(&obs, &pred) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_r2_constant_observed_perfect_prediction() {
        // All obs equal → ss_tot < 1e-10 → returns 1.0
        let obs  = vec![3.0, 3.0, 3.0];
        let pred = vec![3.0, 3.0, 3.0];
        assert!((Calibrator::calculate_r2(&obs, &pred) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_r2_negative_for_terrible_fit() {
        let obs  = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let pred = vec![5.0, 4.0, 3.0, 2.0, 1.0]; // perfectly wrong
        let r2 = Calibrator::calculate_r2(&obs, &pred);
        assert!(r2 < 0.0, "Inverted prediction should have r2 < 0, got {}", r2);
    }

    #[test]
    fn test_r2_known_value() {
        // Simple 4-point dataset with known R²
        let obs  = vec![2.0, 4.0, 5.0, 4.0];
        let pred = vec![2.1, 3.9, 5.2, 3.8];
        let r2 = Calibrator::calculate_r2(&obs, &pred);
        assert!(r2 > 0.9, "Near-perfect fit should have r2 > 0.9, got {}", r2);
    }

    #[test]
    fn test_r2_two_point_perfect() {
        let obs  = vec![0.0, 1.0];
        let pred = vec![0.0, 1.0];
        assert!((Calibrator::calculate_r2(&obs, &pred) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_r2_predicting_mean_gives_zero() {
        let obs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = obs.iter().sum::<f64>() / obs.len() as f64;
        let pred = vec![mean; 5];
        let r2 = Calibrator::calculate_r2(&obs, &pred);
        assert!(r2.abs() < 1e-9, "Predicting mean gives r2=0, got {}", r2);
    }

    // ── calculate_rmse ────────────────────────────────────────────────────────

    #[test]
    fn test_rmse_empty_returns_infinity() {
        let rmse = Calibrator::calculate_rmse(&[], &[]);
        assert!(rmse.is_infinite());
    }

    #[test]
    fn test_rmse_mismatched_lengths_returns_infinity() {
        let obs  = vec![1.0, 2.0];
        let pred = vec![1.0];
        assert!(Calibrator::calculate_rmse(&obs, &pred).is_infinite());
    }

    #[test]
    fn test_rmse_known_value() {
        // errors: [1, -1, 1, -1] → mse = 1.0 → rmse = 1.0
        let obs  = vec![1.0, 2.0, 3.0, 4.0];
        let pred = vec![2.0, 1.0, 4.0, 3.0];
        let rmse = Calibrator::calculate_rmse(&obs, &pred);
        assert!((rmse - 1.0).abs() < 1e-9, "Expected rmse=1.0, got {}", rmse);
    }

    #[test]
    fn test_rmse_positive_always() {
        let obs  = vec![1.0, 2.0, 3.0];
        let pred = vec![1.5, 1.8, 3.3];
        let rmse = Calibrator::calculate_rmse(&obs, &pred);
        assert!(rmse >= 0.0);
    }

    #[test]
    fn test_rmse_symmetric() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.5, 1.8, 3.3];
        let rmse_ab = Calibrator::calculate_rmse(&a, &b);
        let rmse_ba = Calibrator::calculate_rmse(&b, &a);
        assert!((rmse_ab - rmse_ba).abs() < 1e-9, "RMSE must be symmetric");
    }

    #[test]
    fn test_rmse_increases_with_error() {
        let obs   = vec![1.0, 2.0, 3.0, 4.0];
        let pred1 = vec![1.1, 2.1, 3.1, 4.1]; // small error
        let pred2 = vec![2.0, 3.0, 4.0, 5.0]; // larger error
        let rmse1 = Calibrator::calculate_rmse(&obs, &pred1);
        let rmse2 = Calibrator::calculate_rmse(&obs, &pred2);
        assert!(rmse2 > rmse1, "Larger error should give larger RMSE");
    }

    #[test]
    fn test_rmse_single_point() {
        let obs  = vec![3.0];
        let pred = vec![5.0];
        let rmse = Calibrator::calculate_rmse(&obs, &pred);
        assert!((rmse - 2.0).abs() < 1e-9, "Single-point RMSE = |3-5| = 2, got {}", rmse);
    }

    #[test]
    fn test_r2_good_chip_model() {
        // Synthetic CHIP data vs near-perfect prediction
        let obs  = vec![0.005, 0.015, 0.040, 0.070, 0.120];
        let pred = vec![0.006, 0.014, 0.042, 0.068, 0.115];
        let r2 = Calibrator::calculate_r2(&obs, &pred);
        assert!(r2 > 0.99, "Near-perfect CHIP model should have r2 > 0.99, got {}", r2);
    }

    #[test]
    fn test_rmse_chip_model() {
        let obs  = vec![0.005, 0.015, 0.040, 0.070, 0.120];
        let pred = vec![0.006, 0.014, 0.042, 0.068, 0.115];
        let rmse = Calibrator::calculate_rmse(&obs, &pred);
        assert!(rmse < 0.005, "Near-perfect CHIP model RMSE should be small, got {}", rmse);
    }

    #[test]
    fn test_r2_ros_model() {
        let obs  = vec![0.15, 0.25, 0.45, 0.65];
        let pred = vec![0.16, 0.24, 0.46, 0.63];
        let r2 = Calibrator::calculate_r2(&obs, &pred);
        assert!(r2 > 0.99, "Near-perfect ROS model r2={}", r2);
    }

    #[test]
    fn test_rmse_ros_model() {
        let obs  = vec![0.15, 0.25, 0.45, 0.65];
        let pred = vec![0.16, 0.24, 0.46, 0.63];
        let rmse = Calibrator::calculate_rmse(&obs, &pred);
        assert!(rmse < 0.02, "Near-perfect ROS RMSE should be small, got {}", rmse);
    }

    #[test]
    fn test_r2_scale_invariance() {
        // Scaling predicted by a constant doesn't preserve r2 unless 1.0
        let obs  = vec![1.0, 2.0, 3.0, 4.0];
        let pred_scaled: Vec<f64> = obs.iter().map(|x| x * 2.0).collect();
        let r2 = Calibrator::calculate_r2(&obs, &pred_scaled);
        // r2 can be negative — just verify it's finite
        assert!(r2.is_finite(), "r2 should be finite even for poor fit");
    }

    #[test]
    fn test_training_range_lower_less_than_upper() {
        let c = Calibrator::new();
        assert!(c.training_age_range.0 < c.training_age_range.1);
    }

    #[test]
    fn test_rmse_multiple_scenarios() {
        // Larger dataset
        let obs:  Vec<f64> = (0..10).map(|i| i as f64 * 0.1).collect();
        let pred: Vec<f64> = obs.iter().map(|x| x + 0.01).collect();
        let rmse = Calibrator::calculate_rmse(&obs, &pred);
        assert!((rmse - 0.01).abs() < 1e-9, "Uniform offset 0.01 → RMSE=0.01, got {}", rmse);
    }

    #[test]
    fn test_r2_one_when_perfect_match() {
        let obs: Vec<f64> = vec![0.1, 0.5, 0.9, 0.3, 0.7];
        let r2 = Calibrator::calculate_r2(&obs, &obs);
        assert!((r2 - 1.0).abs() < 1e-6);
    }
}
