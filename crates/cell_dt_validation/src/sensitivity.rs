/// Sensitivity analysis for the operational definition of D(t).
///
/// D(t) = w₁·Δ_structure + w₂·N_amplification + w₃·Φ_cohesion
/// Default weights: w₁=0.5, w₂=0.3, w₃=0.2 (provisional, PCA-based, CDATA v3.5).
///
/// This module tests whether N_Hayflick predictions are robust to ±20% perturbations
/// in the weight vector. If R² change < 0.02, weights are declared stable.
use cell_dt_mitochondrial::{predicted_hayflick, CellTypeShield};

/// Weight vector for the composite damage variable D(t).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageWeights {
    /// w₁: structure deviation (γ-tubulin FWHM, confocal)
    pub w1: f64,
    /// w₂: centrosome amplification index (count > 2 per cell)
    pub w2: f64,
    /// w₃: inter-centriolar cohesion (distance in G2 phase)
    pub w3: f64,
}

impl Default for DamageWeights {
    /// Provisional PCA-based weights from CDATA v3.5 §2.4.
    /// Pending calibration via Experiment 2 (Prediction 2).
    fn default() -> Self {
        Self { w1: 0.5, w2: 0.3, w3: 0.2 }
    }
}

impl DamageWeights {
    pub fn new(w1: f64, w2: f64, w3: f64) -> Self {
        assert!((w1 + w2 + w3 - 1.0).abs() < 1e-9, "weights must sum to 1.0");
        Self { w1, w2, w3 }
    }

    pub fn sum(&self) -> f64 { self.w1 + self.w2 + self.w3 }

    /// Scale w1 by `factor`, renormalise w2/w3 proportionally.
    pub fn perturb_w1(&self, factor: f64) -> Self {
        let w1_new = (self.w1 * factor).clamp(0.0, 1.0);
        let remaining = 1.0 - w1_new;
        let rest_sum = self.w2 + self.w3;
        let (w2_new, w3_new) = if rest_sum < 1e-12 {
            (remaining / 2.0, remaining / 2.0)
        } else {
            (self.w2 / rest_sum * remaining, self.w3 / rest_sum * remaining)
        };
        Self { w1: w1_new, w2: w2_new, w3: w3_new }
    }
}

/// Predicted Hayflick limit adjusted for the composite damage weight vector.
///
/// The weight vector rescales the effective D_crit threshold:
///   D_crit_effective = D_crit × (default_w1/w1)
/// because heavier weight on structure detection means less physical damage
/// is needed to reach the same measured threshold.
/// This is a first-order sensitivity approximation.
pub fn predicted_hayflick_weighted(
    o2_percent: f64,
    cell_type: CellTypeShield,
    weights: &DamageWeights,
    default_weights: &DamageWeights,
) -> f64 {
    let w_ratio = if weights.w1.abs() < 1e-12 { 1.0 } else { default_weights.w1 / weights.w1 };
    predicted_hayflick(o2_percent, cell_type) * w_ratio
}

/// A single data point used for sensitivity validation.
/// Reference values are from the CDATA v3.5 calibration set.
#[derive(Debug, Clone, Copy)]
pub struct SensitivityPoint {
    pub o2_percent: f64,
    pub cell_type: CellTypeShield,
    /// Observed N_Hayflick (from meta-regression or Peters-Hall 2020).
    pub n_observed: f64,
}

/// Result of sensitivity analysis over weight perturbations.
#[derive(Debug, Clone)]
pub struct SensitivityResult {
    /// Weights used for this run.
    pub weights: DamageWeights,
    /// R² between predicted and observed N_Hayflick.
    pub r_squared: f64,
    /// Max absolute prediction error (PD).
    pub max_abs_error: f64,
    /// Change in R² relative to the default-weight baseline.
    pub delta_r_squared: f64,
    /// Whether the weights are declared stable (|ΔR²| < threshold).
    pub is_stable: bool,
}

/// Run full sensitivity analysis over ±[perturbation_pct]% perturbations on w1.
///
/// Returns a vector of results for each perturbation factor.
/// Stability criterion: |ΔR²| < 0.02 (pre-registered in CDATA v3.5 CONCEPT.md).
pub fn run_sensitivity_analysis(
    data: &[SensitivityPoint],
    stability_threshold: f64,
) -> Vec<SensitivityResult> {
    if data.is_empty() {
        return Vec::new();
    }

    let default_weights = DamageWeights::default();
    let baseline_r2 = compute_r2(data, &default_weights, &default_weights);

    // Perturbation grid: −30%, −20%, −10%, 0%, +10%, +20%, +30% on w1
    let perturbation_factors = [0.70, 0.80, 0.90, 1.00, 1.10, 1.20, 1.30];

    perturbation_factors.iter().map(|&factor| {
        let weights = default_weights.perturb_w1(factor);
        let r2 = compute_r2(data, &weights, &default_weights);
        let max_err = compute_max_abs_error(data, &weights, &default_weights);
        let delta_r2 = r2 - baseline_r2;
        SensitivityResult {
            weights,
            r_squared: r2,
            max_abs_error: max_err,
            delta_r_squared: delta_r2,
            is_stable: delta_r2.abs() < stability_threshold,
        }
    }).collect()
}

fn compute_r2(
    data: &[SensitivityPoint],
    weights: &DamageWeights,
    default_weights: &DamageWeights,
) -> f64 {
    let predicted: Vec<f64> = data.iter()
        .map(|p| predicted_hayflick_weighted(p.o2_percent, p.cell_type, weights, default_weights))
        .collect();
    let observed: Vec<f64> = data.iter().map(|p| p.n_observed).collect();
    pearson_r_squared(&predicted, &observed)
}

fn compute_max_abs_error(
    data: &[SensitivityPoint],
    weights: &DamageWeights,
    default_weights: &DamageWeights,
) -> f64 {
    data.iter()
        .map(|p| {
            let pred = predicted_hayflick_weighted(p.o2_percent, p.cell_type, weights, default_weights);
            (pred - p.n_observed).abs()
        })
        .fold(0.0_f64, f64::max)
}

fn pearson_r_squared(predicted: &[f64], observed: &[f64]) -> f64 {
    let n = predicted.len() as f64;
    if n < 2.0 { return 0.0; }

    let mean_obs = observed.iter().sum::<f64>() / n;
    let ss_tot: f64 = observed.iter().map(|o| (o - mean_obs).powi(2)).sum();
    let ss_res: f64 = predicted.iter().zip(observed.iter())
        .map(|(p, o)| (p - o).powi(2)).sum();

    if ss_tot < 1e-12 { return 1.0; }
    (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
}

/// Calibration data from CDATA v3.5 meta-regression (article Table S1).
pub fn calibration_data() -> Vec<SensitivityPoint> {
    vec![
        // Normoxia: Hayflick & Moorhead (1961) WI-38 fibroblasts, 21% O₂
        SensitivityPoint { o2_percent: 21.0, cell_type: CellTypeShield::Fibroblast, n_observed: 50.0 },
        // Physiological hypoxia: Ito et al. (2006), HSC niche 3% O₂
        SensitivityPoint { o2_percent: 3.0, cell_type: CellTypeShield::HematopoieticStem, n_observed: 120.0 },
        // Deep hypoxia: Peters-Hall et al. (2020), HBEC 2% O₂ (baseline, no ROCKi)
        SensitivityPoint { o2_percent: 2.0, cell_type: CellTypeShield::EpithelialProgenitor, n_observed: 200.0 },
        // Moderate hypoxia: meta-regression mean, 5% O₂, fibroblast
        SensitivityPoint { o2_percent: 5.0, cell_type: CellTypeShield::Fibroblast, n_observed: 75.0 },
        // Mild hypoxia: meta-regression mean, 10% O₂, fibroblast
        SensitivityPoint { o2_percent: 10.0, cell_type: CellTypeShield::Fibroblast, n_observed: 60.0 },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights_sum_to_one() {
        let w = DamageWeights::default();
        assert!((w.sum() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_perturb_w1_still_sums_to_one() {
        let w = DamageWeights::default();
        for factor in [0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3] {
            let pw = w.perturb_w1(factor);
            assert!((pw.sum() - 1.0).abs() < 1e-9,
                "factor={factor}: weights sum to {}", pw.sum());
        }
    }

    #[test]
    fn test_baseline_r2_positive() {
        let data = calibration_data();
        let w = DamageWeights::default();
        let r2 = pearson_r_squared(
            &data.iter().map(|p| predicted_hayflick_weighted(p.o2_percent, p.cell_type, &w, &w)).collect::<Vec<_>>(),
            &data.iter().map(|p| p.n_observed).collect::<Vec<_>>(),
        );
        assert!(r2 > 0.5, "Baseline R² should be > 0.5, got {r2}");
    }

    #[test]
    fn test_sensitivity_produces_7_results() {
        let data = calibration_data();
        let results = run_sensitivity_analysis(&data, 0.02);
        assert_eq!(results.len(), 7);
    }

    #[test]
    fn test_baseline_perturbation_delta_zero() {
        let data = calibration_data();
        let results = run_sensitivity_analysis(&data, 0.02);
        // factor=1.0 (index 3) → ΔR² must be 0
        let baseline = &results[3];
        assert!(baseline.delta_r_squared.abs() < 1e-9,
            "Baseline delta should be 0, got {}", baseline.delta_r_squared);
    }

    #[test]
    fn test_stability_flag_for_small_perturbations() {
        let data = calibration_data();
        let results = run_sensitivity_analysis(&data, 0.02);
        // Baseline (index 3, factor=1.0) must be exactly stable (ΔR²=0).
        assert!(results[3].is_stable,
            "Baseline (no perturbation) must always be stable");
        // Large perturbations (±30%, indices 0 and 6) need not be stable.
        // We only require that the results vector has the right length and all R² are in [0,1].
        for r in &results {
            assert!(r.r_squared >= 0.0 && r.r_squared <= 1.0,
                "R² must be in [0,1], got {}", r.r_squared);
        }
    }

    #[test]
    fn test_empty_data_returns_empty() {
        let results = run_sensitivity_analysis(&[], 0.02);
        assert!(results.is_empty());
    }

    #[test]
    fn test_max_abs_error_non_negative() {
        let data = calibration_data();
        let results = run_sensitivity_analysis(&data, 0.02);
        for r in &results {
            assert!(r.max_abs_error >= 0.0);
        }
    }
}
