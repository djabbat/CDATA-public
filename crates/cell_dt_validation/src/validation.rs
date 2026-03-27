use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub biomarker_name: String,
    pub r_squared: f64,
    pub rmse: f64,
    pub mae: f64,
    pub n_points: usize,
}

impl ValidationResult {
    pub fn is_acceptable(&self) -> bool {
        self.r_squared > 0.75
    }
}

#[derive(Debug, Default)]
pub struct ValidationSuite {
    pub results: Vec<ValidationResult>,
}

impl ValidationSuite {
    pub fn add_result(&mut self, result: ValidationResult) {
        self.results.push(result);
    }

    pub fn mean_r2(&self) -> f64 {
        if self.results.is_empty() { return 0.0; }
        self.results.iter().map(|r| r.r_squared).sum::<f64>() / self.results.len() as f64
    }

    pub fn all_pass(&self) -> bool {
        self.results.iter().all(|r| r.is_acceptable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_suite() {
        let mut suite = ValidationSuite::default();
        suite.add_result(ValidationResult {
            biomarker_name: "CHIP".to_string(),
            r_squared: 0.79,
            rmse: 0.05,
            mae: 0.04,
            n_points: 5,
        });
        suite.add_result(ValidationResult {
            biomarker_name: "ROS".to_string(),
            r_squared: 0.84,
            rmse: 0.07,
            mae: 0.05,
            n_points: 4,
        });
        assert!(suite.mean_r2() > 0.75);
        assert!(suite.all_pass());
    }
}
