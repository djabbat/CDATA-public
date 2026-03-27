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
}
