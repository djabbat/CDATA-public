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

    // ── ValidationResult::is_acceptable ──────────────────────────────────────

    #[test]
    fn test_acceptable_above_threshold() {
        let r = ValidationResult {
            biomarker_name: "Test".to_string(),
            r_squared: 0.80,
            rmse: 0.05,
            mae: 0.04,
            n_points: 10,
        };
        assert!(r.is_acceptable());
    }

    #[test]
    fn test_not_acceptable_below_threshold() {
        let r = ValidationResult {
            biomarker_name: "Test".to_string(),
            r_squared: 0.70,
            rmse: 0.10,
            mae: 0.08,
            n_points: 10,
        };
        assert!(!r.is_acceptable());
    }

    #[test]
    fn test_not_acceptable_at_threshold() {
        let r = ValidationResult {
            biomarker_name: "Test".to_string(),
            r_squared: 0.75,
            rmse: 0.05,
            mae: 0.04,
            n_points: 10,
        };
        // r_squared > 0.75 → at exactly 0.75 it should fail
        assert!(!r.is_acceptable(), "r2=0.75 is not strictly > 0.75");
    }

    #[test]
    fn test_acceptable_boundary_just_above() {
        let r = ValidationResult {
            biomarker_name: "Test".to_string(),
            r_squared: 0.7501,
            rmse: 0.05,
            mae: 0.04,
            n_points: 10,
        };
        assert!(r.is_acceptable(), "r2=0.7501 should be acceptable");
    }

    #[test]
    fn test_acceptable_perfect_r2() {
        let r = ValidationResult {
            biomarker_name: "Test".to_string(),
            r_squared: 1.0,
            rmse: 0.0,
            mae: 0.0,
            n_points: 5,
        };
        assert!(r.is_acceptable());
    }

    // ── ValidationSuite::mean_r2 ──────────────────────────────────────────────

    #[test]
    fn test_mean_r2_empty_suite() {
        let suite = ValidationSuite::default();
        assert_eq!(suite.mean_r2(), 0.0, "Empty suite mean_r2 must be 0");
    }

    #[test]
    fn test_mean_r2_single_result() {
        let mut suite = ValidationSuite::default();
        suite.add_result(ValidationResult {
            biomarker_name: "X".to_string(),
            r_squared: 0.9,
            rmse: 0.01,
            mae: 0.01,
            n_points: 5,
        });
        assert!((suite.mean_r2() - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_mean_r2_two_results() {
        let mut suite = ValidationSuite::default();
        suite.add_result(ValidationResult { biomarker_name: "A".to_string(), r_squared: 0.8, rmse: 0.0, mae: 0.0, n_points: 5 });
        suite.add_result(ValidationResult { biomarker_name: "B".to_string(), r_squared: 0.6, rmse: 0.0, mae: 0.0, n_points: 5 });
        assert!((suite.mean_r2() - 0.7).abs() < 1e-9, "Mean of 0.8 and 0.6 = 0.7");
    }

    // ── ValidationSuite::all_pass ─────────────────────────────────────────────

    #[test]
    fn test_all_pass_empty_suite() {
        let suite = ValidationSuite::default();
        assert!(suite.all_pass(), "Empty suite trivially all-pass");
    }

    #[test]
    fn test_all_pass_one_failing() {
        let mut suite = ValidationSuite::default();
        suite.add_result(ValidationResult { biomarker_name: "A".to_string(), r_squared: 0.9, rmse: 0.0, mae: 0.0, n_points: 5 });
        suite.add_result(ValidationResult { biomarker_name: "B".to_string(), r_squared: 0.6, rmse: 0.0, mae: 0.0, n_points: 5 });
        assert!(!suite.all_pass(), "One failing result makes all_pass false");
    }

    #[test]
    fn test_all_pass_all_passing() {
        let mut suite = ValidationSuite::default();
        for i in 0..5 {
            suite.add_result(ValidationResult {
                biomarker_name: format!("B{}", i),
                r_squared: 0.8 + i as f64 * 0.01,
                rmse: 0.05,
                mae: 0.04,
                n_points: 5,
            });
        }
        assert!(suite.all_pass());
    }

    #[test]
    fn test_add_result_increments_count() {
        let mut suite = ValidationSuite::default();
        assert_eq!(suite.results.len(), 0);
        suite.add_result(ValidationResult { biomarker_name: "A".to_string(), r_squared: 0.9, rmse: 0.0, mae: 0.0, n_points: 3 });
        assert_eq!(suite.results.len(), 1);
        suite.add_result(ValidationResult { biomarker_name: "B".to_string(), r_squared: 0.85, rmse: 0.0, mae: 0.0, n_points: 3 });
        assert_eq!(suite.results.len(), 2);
    }

    #[test]
    fn test_validation_result_clone() {
        let r = ValidationResult {
            biomarker_name: "Test".to_string(),
            r_squared: 0.9,
            rmse: 0.05,
            mae: 0.04,
            n_points: 5,
        };
        let r2 = r.clone();
        assert_eq!(r.biomarker_name, r2.biomarker_name);
        assert!((r.r_squared - r2.r_squared).abs() < 1e-9);
    }

    #[test]
    fn test_mean_r2_three_results() {
        let mut suite = ValidationSuite::default();
        for r2 in [0.8, 0.9, 1.0] {
            suite.add_result(ValidationResult { biomarker_name: "T".to_string(), r_squared: r2, rmse: 0.0, mae: 0.0, n_points: 5 });
        }
        assert!((suite.mean_r2() - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_validation_result_debug() {
        let r = ValidationResult {
            biomarker_name: "CHIP".to_string(),
            r_squared: 0.85,
            rmse: 0.02,
            mae: 0.01,
            n_points: 5,
        };
        let dbg = format!("{:?}", r);
        assert!(dbg.contains("CHIP"));
    }

    #[test]
    fn test_suite_debug_output() {
        let suite = ValidationSuite::default();
        let dbg = format!("{:?}", suite);
        assert!(dbg.contains("ValidationSuite"));
    }

    #[test]
    fn test_acceptable_requires_positive_r2() {
        let r = ValidationResult {
            biomarker_name: "X".to_string(),
            r_squared: -0.5,
            rmse: 0.5,
            mae: 0.4,
            n_points: 5,
        };
        assert!(!r.is_acceptable(), "Negative r2 must not be acceptable");
    }
}
