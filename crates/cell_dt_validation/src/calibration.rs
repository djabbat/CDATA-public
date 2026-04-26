/// CDATA v3.2.3 — Bayesian MCMC calibration (Metropolis-Hastings)
///
/// Calibrates 2 free parameters of FixedParameters against reference datasets
/// (ROS, telomere, CHIP VAF, MCAI, epigenetic age) using a random-walk
/// Metropolis-Hastings sampler with Gaussian priors.
///
/// Convergence is assessed with a simplified split-chain R-hat (< 1.05 target).

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use cell_dt_aging_engine::{AgingEngine, SimulationConfig, SimulationPreset};
use cell_dt_core::FixedParameters;

use crate::datasets::{CalibrationDataset, ReferenceDatasets};

// ── Parameter descriptor ──────────────────────────────────────────────────────

/// One calibration parameter: current value + Gaussian prior + proposal width.
#[derive(Debug, Clone)]
pub struct CalibrationParam {
    pub name: &'static str,
    pub value: f64,
    pub prior_mean: f64,
    pub prior_sd: f64,
    /// Gaussian random-walk step width (tuned for ~23% acceptance)
    pub proposal_sd: f64,
    /// Lower bound (hard constraint)
    pub min: f64,
    /// Upper bound (hard constraint)
    pub max: f64,
}

impl CalibrationParam {
    fn log_prior(&self) -> f64 {
        let z = (self.value - self.prior_mean) / self.prior_sd;
        -0.5 * z * z  // unnormalised log-Normal
    }

    fn propose(&self, rng: &mut StdRng) -> f64 {
        // Box-Muller transform: generate N(0,1) from two U(0,1) samples.
        // BUG-C4 fix (2026-04-06): removed the dead `delta` variable that consumed
        // RNG state (shifting all subsequent draws) without contributing to the proposal.
        let u1: f64 = rng.gen::<f64>().max(1e-12);
        let u2: f64 = rng.gen::<f64>();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        (self.value + self.proposal_sd * z).clamp(self.min, self.max)
    }
}

/// Calibrated (free) 2-parameter set for MCMC.
///
/// Values represent the **post-Round-7 recalibration** (2026-03-29):
/// - Telomere term added back to MCAI via differentiated_telomere_length (M1b)
/// - Age-dependent epi-age acceleration multiplier (M2)
/// - MCAI changed to unweighted 5-component mean (v3.2.3)
/// MCMC chain: pilot=1000, main=5000, adaptive proposal; R-hat < 1.05.
/// Posterior means: tau_protection=24.3 yr, pi_0=0.87 (stable across Round-7 fixes).
///
/// Fixed parameters (excluded from MCMC):
/// - `alpha`          = 0.0082 — fixed at literature value (PMID: 36583780);
///                      collinear with tau_protection (posterior r = 0.858).
/// - `hsc_nu`         = 1.2    — divisions/year (Wilson 2008 Nature standard for murine HSC); insensitive: ΔR² ≈ 0 at ±20% perturbation.
/// - `dnmt3a_fitness` = 0.15   — insensitive: ΔR² ≈ 0 at ±20% perturbation.
///
/// These parameters take their default values from `FixedParameters::default()`.
pub fn default_calibration_params() -> Vec<CalibrationParam> {
    vec![
        CalibrationParam {
            name: "tau_protection",
            // Post-Round-7 posterior mean: 24.3 yr (95% CI: 19.1–29.7).
            // Stable across fixes — confirms tau_protection robustness.
            value: 24.3, prior_mean: 24.3, prior_sd: 5.0,
            proposal_sd: 5.0, min: 5.0, max: 60.0,
        },
        CalibrationParam {
            name: "pi_0",
            // Post-Round-7 posterior mean: 0.87 (95% CI: 0.82–0.92).
            // Slight constraint tightening after MCAI formula change (v3.2.3: unweighted mean).
            value: 0.87, prior_mean: 0.87, prior_sd: 0.05,
            proposal_sd: 0.05, min: 0.50, max: 0.99,
        },
    ]
}

// ── Forward model ─────────────────────────────────────────────────────────────

/// Apply a calibration parameter vector to FixedParameters.
fn apply_params(params: &[CalibrationParam], fp: &mut FixedParameters) {
    for p in params {
        match p.name {
            "alpha"           => fp.alpha           = p.value,
            "tau_protection"  => fp.tau_protection  = p.value,
            "pi_0"            => fp.pi_0            = p.value,
            "hsc_nu"          => fp.hsc_nu          = p.value,
            "dnmt3a_fitness"  => fp.dnmt3a_fitness  = p.value,
            _                 => {}
        }
    }
}

/// Raw (un-scaled) biomarker value at the nearest snapshot to `age`.
///
/// Active biomarkers (used in calibration):
/// - "ROS level"    → `ros_level`  (normalised at age 20 in the ros-normalised snap vec)
/// - "CHIP VAF"     → `chip_vaf`   (total CHIP clone frequency from ChipSystem)
/// - "MCAI"→ `mcai` (unweighted 5-component)
/// - "Telomere length" → `differentiated_telomere_length` (differentiated progeny; shortens with age)
///
/// Excluded biomarkers (always return `None`):
/// - "Epi-age acceleration" — ≈0 in 20–50 yr range due to init lag (epi_age starts at 0)
fn extract_biomarker(
    snaps: &[cell_dt_aging_engine::AgeSnapshot],
    age: f64,
    biomarker: &str,
) -> Option<f64> {
    if matches!(biomarker, "Epi-age acceleration") {
        return None;
    }

    let snap = snaps.iter().min_by(|a, b| {
        (a.age_years - age).abs()
            .partial_cmp(&(b.age_years - age).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;

    let v = match biomarker {
        "ROS level"     => snap.ros_level,
        "CHIP VAF"      => snap.chip_vaf,
        "MCAI" => snap.mcai,
        // Differentiated telomere — shortens with age (validated vs Lansdorp 2005)
        "Telomere length" => snap.differentiated_telomere_length,
        _               => return None,
    };
    Some(v)
}

/// Compute a single-point scale factor that anchors simulated biomarker values
/// to the reference value at age 20.  Used for trend-based R² / RMSE:
///
///   scaled_pred(age) = raw_pred(age) × scale_factor
///
/// This converts the comparison from "do absolute values match?" to
/// "does the MODEL correctly predict the age-dependent TRAJECTORY,
///  given a calibration at the young-adult baseline (age 20)?"
///
/// Returns 1.0 if the simulation value at age 20 is below a minimum threshold
/// (i.e., no meaningful anchoring is possible).
fn scale_factor_at_20(
    snaps: &[cell_dt_aging_engine::AgeSnapshot],
    biomarker: &str,
    ref_at_20: f64,
) -> f64 {
    let sim_at_20 = extract_biomarker(snaps, 20.0, biomarker).unwrap_or(0.0);
    if sim_at_20 < 1e-9 { return 1.0; }  // can't anchor a near-zero value
    (ref_at_20 / sim_at_20).clamp(0.01, 1000.0)
}

/// Normalise simulated ROS to reference scale (sim starts at ~0.12 at age 0;
/// dataset is normalised to 1.0 at age 20).
fn normalise_ros(snaps: &[cell_dt_aging_engine::AgeSnapshot]) -> Vec<cell_dt_aging_engine::AgeSnapshot> {
    let ros_at_20 = snaps.iter()
        .find(|s| (s.age_years - 20.0).abs() < 1.5)
        .map(|s| s.ros_level)
        .unwrap_or(1.0)
        .max(1e-6);
    snaps.iter().cloned().map(|mut s| { s.ros_level /= ros_at_20; s }).collect()
}

/// Run AgingEngine with given param vector; return snapshots (one per year).
fn run_simulation(params: &[CalibrationParam]) -> Option<Vec<cell_dt_aging_engine::AgeSnapshot>> {
    let mut fp = FixedParameters::default();
    apply_params(params, &mut fp);

    // Temporarily apply via custom config — we patch params after construction
    let config = SimulationConfig {
        preset: SimulationPreset::Normal,
        ..SimulationConfig::default()
    };
    let mut engine = AgingEngine::new(config).ok()?;
    // Override the params with calibrated values
    engine.params = fp;

    Some(engine.run(1))
}

// ── Log-posterior ─────────────────────────────────────────────────────────────

/// Gaussian log-likelihood with scale-anchored predictions.
///
/// For each active biomarker, a single scale factor is computed by anchoring
/// the simulation to the reference value at age 20.  All predicted values for
/// that biomarker are multiplied by this factor before computing the likelihood.
///
/// This measures whether the model captures the correct AGE-DEPENDENT TRAJECTORY
/// (trend) rather than requiring exact absolute level matching.
fn log_likelihood(
    snaps: &[cell_dt_aging_engine::AgeSnapshot],
    snaps_ros: &[cell_dt_aging_engine::AgeSnapshot],
    ds: &ReferenceDatasets,
) -> f64 {
    let datasets: &[(&CalibrationDataset, &str)] = &[
        (&ds.ros,      "ROS level"),
        (&ds.chip_vaf, "CHIP VAF"),
        (&ds.mcai,  "MCAI"),
    ];

    let mut ll = 0.0f64;
    for (dataset, biomarker) in datasets {
        let snap_src: &[cell_dt_aging_engine::AgeSnapshot] = if *biomarker == "ROS level" {
            snaps_ros
        } else {
            snaps
        };

        // Anchor: ref value at age 20 (first dataset point is age 20)
        let ref_at_20 = dataset.observed[0];
        let sf = scale_factor_at_20(snap_src, biomarker, ref_at_20);

        for (i, &age) in dataset.ages.iter().enumerate() {
            let pred = match extract_biomarker(snap_src, age, biomarker) {
                Some(v) => v * sf,
                None    => continue,
            };
            let obs   = dataset.observed[i];
            let sigma = dataset.noise_sd[i].max(1e-6);
            let z     = (pred - obs) / sigma;
            ll -= 0.5 * z * z;
        }
    }
    ll
}

fn log_prior_total(params: &[CalibrationParam]) -> f64 {
    params.iter().map(|p| p.log_prior()).sum()
}

fn log_posterior(params: &[CalibrationParam], ds: &ReferenceDatasets) -> f64 {
    let snaps = match run_simulation(params) {
        Some(s) => s,
        None    => return f64::NEG_INFINITY,
    };
    let snaps_ros = normalise_ros(&snaps);
    log_prior_total(params) + log_likelihood(&snaps, &snaps_ros, ds)
}

// ── MCMC result ───────────────────────────────────────────────────────────────

/// One accepted MCMC sample.
#[derive(Debug, Clone)]
pub struct McmcSample {
    pub param_values: Vec<f64>,   // same order as `param_names`
    pub log_posterior: f64,
}

/// Result of a completed MCMC run.
#[derive(Debug)]
pub struct McmcResult {
    /// Names of calibrated parameters (same order as sample.param_values).
    pub param_names: Vec<&'static str>,
    /// All accepted samples (after burn-in is removed by the caller if desired).
    pub samples: Vec<McmcSample>,
    /// Fraction of proposals that were accepted.
    pub acceptance_rate: f64,
    /// R-hat convergence diagnostic (split-chain).  < 1.05 = converged.
    pub r_hat: Vec<f64>,
    /// Posterior mean for each parameter.
    pub posterior_mean: Vec<f64>,
    /// Posterior standard deviation for each parameter.
    pub posterior_sd: Vec<f64>,
    /// R² on the training datasets using the posterior-mean parameters.
    pub r2_training: f64,
    /// RMSE on the training datasets using the posterior-mean parameters.
    pub rmse_training: f64,
}

impl McmcResult {
    /// Pearson correlation matrix of the posterior samples.
    ///
    /// Returns an `n×n` matrix (row-major, flattened `Vec<f64>`) where entry
    /// `[i * n + j]` is the correlation between parameters `i` and `j`.
    /// Diagonal is always 1.0.  Off-diagonal |r| > 0.7 indicates strong
    /// posterior correlation (potential identifiability concern).
    pub fn correlation_matrix(&self) -> Vec<f64> {
        let n = self.param_names.len();
        if self.samples.is_empty() || n == 0 {
            return vec![1.0; n * n];
        }
        let m = self.samples.len() as f64;

        // Column means
        let means: Vec<f64> = (0..n)
            .map(|i| self.samples.iter().map(|s| s.param_values[i]).sum::<f64>() / m)
            .collect();

        // Column standard deviations
        let sds: Vec<f64> = (0..n)
            .map(|i| {
                let mu = means[i];
                let var = self.samples.iter()
                    .map(|s| (s.param_values[i] - mu).powi(2))
                    .sum::<f64>() / m;
                var.sqrt().max(1e-12)
            })
            .collect();

        // Correlation matrix
        let mut corr = vec![0.0_f64; n * n];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    corr[i * n + j] = 1.0;
                } else {
                    let cov = self.samples.iter()
                        .map(|s| (s.param_values[i] - means[i]) * (s.param_values[j] - means[j]))
                        .sum::<f64>() / m;
                    corr[i * n + j] = (cov / (sds[i] * sds[j])).clamp(-1.0, 1.0);
                }
            }
        }
        corr
    }
}

// ── R-hat (split-chain Gelman-Rubin) ─────────────────────────────────────────

fn r_hat_single(chain: &[f64]) -> f64 {
    let n = chain.len();
    if n < 4 { return f64::NAN; }
    let half = n / 2;
    let a = &chain[..half];
    let b = &chain[half..];

    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let var   = |v: &[f64]| {
        let m = mean(v);
        v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64
    };

    let ma = mean(a); let mb = mean(b);
    let va = var(a);  let vb = var(b);
    let w     = (va + vb) / 2.0;                  // within-chain variance
    let b_var = ((ma - mb).powi(2)) / 2.0;        // between-chain (2 sub-chains)
    let var_plus = w + b_var;
    // If both halves are constant but differ in mean → not converged → return large value
    if w < 1e-12 {
        return if b_var < 1e-12 { 1.0 } else { f64::INFINITY };
    }
    (var_plus / w).sqrt()
}

// ── Metropolis sampler ────────────────────────────────────────────────────────

/// Metropolis-Hastings MCMC calibrator.
pub struct Metropolis {
    /// Number of warm-up steps (discarded from result).
    pub burn_in: usize,
    /// Number of post-warm-up samples to collect.
    pub n_samples: usize,
    /// RNG seed for reproducibility.
    pub seed: u64,
}

impl Default for Metropolis {
    fn default() -> Self {
        Self { burn_in: 200, n_samples: 500, seed: 12345 }
    }
}

impl Metropolis {
    pub fn new(burn_in: usize, n_samples: usize, seed: u64) -> Self {
        Self { burn_in, n_samples, seed }
    }

    /// Run MCMC and return results.
    pub fn run(
        &self,
        mut params: Vec<CalibrationParam>,
        ds: &ReferenceDatasets,
    ) -> McmcResult {
        let mut rng = StdRng::seed_from_u64(self.seed);

        let mut current_lp = log_posterior(&params, ds);
        let mut accepted   = 0usize;
        let total_steps    = self.burn_in + self.n_samples;

        let param_names: Vec<&'static str> = params.iter().map(|p| p.name).collect();
        let n_params = params.len();
        let mut samples: Vec<McmcSample> = Vec::with_capacity(self.n_samples);

        // ── Main loop ─────────────────────────────────────────────────────────
        for step in 0..total_steps {
            // Propose a change to one parameter at a time (component-wise)
            let idx = step % n_params;
            let old_val = params[idx].value;
            let new_val = params[idx].propose(&mut rng);

            params[idx].value = new_val;
            let proposed_lp = log_posterior(&params, ds);

            // Metropolis-Hastings acceptance criterion
            let log_alpha = proposed_lp - current_lp;
            let u: f64 = rng.gen::<f64>().max(1e-300).ln();

            if u < log_alpha {
                // Accept
                current_lp = proposed_lp;
                if step >= self.burn_in { accepted += 1; }
            } else {
                // Reject — revert
                params[idx].value = old_val;
            }

            // Record sample (post burn-in only)
            if step >= self.burn_in {
                samples.push(McmcSample {
                    param_values: params.iter().map(|p| p.value).collect(),
                    log_posterior: current_lp,
                });
            }
        }

        // ── Posterior statistics ──────────────────────────────────────────────
        let acceptance_rate = accepted as f64 / self.n_samples as f64;

        let posterior_mean: Vec<f64> = (0..n_params).map(|i| {
            samples.iter().map(|s| s.param_values[i]).sum::<f64>() / samples.len() as f64
        }).collect();

        let posterior_sd: Vec<f64> = (0..n_params).map(|i| {
            let m = posterior_mean[i];
            let v = samples.iter().map(|s| (s.param_values[i] - m).powi(2)).sum::<f64>()
                / samples.len() as f64;
            v.sqrt()
        }).collect();

        let r_hat: Vec<f64> = (0..n_params).map(|i| {
            let chain: Vec<f64> = samples.iter().map(|s| s.param_values[i]).collect();
            r_hat_single(&chain)
        }).collect();

        // ── Fitness on training data using posterior mean ──────────────────────
        let mut mean_params = params.clone();
        for (i, p) in mean_params.iter_mut().enumerate() {
            p.value = posterior_mean[i];
        }
        let (r2, rmse) = training_fitness(&mean_params, ds);

        McmcResult {
            param_names,
            samples,
            acceptance_rate,
            r_hat,
            posterior_mean,
            posterior_sd,
            r2_training: r2,
            rmse_training: rmse,
        }
    }

    /// Adaptive Metropolis-Hastings (Haario et al. 2001).
    ///
    /// Phase 1 — pilot: `pilot_samples` steps with fixed proposals.
    /// Phase 2 — adapt: set `proposal_sd[i] = scale * posterior_sd[i]` from pilot.
    /// Phase 3 — main: `burn_in + n_samples` with adapted proposals.
    ///
    /// `scale` defaults to `2.38 / sqrt(n_params)` (optimal for normal posteriors).
    pub fn run_adaptive(
        &self,
        params:       Vec<CalibrationParam>,
        ds:           &ReferenceDatasets,
        pilot_samples: usize,
    ) -> McmcResult {
        let n_params = params.len();
        let scale    = 2.38 / (n_params as f64).sqrt();

        // ── Phase 1: pilot run ────────────────────────────────────────────────
        let pilot = Metropolis::new(pilot_samples / 2, pilot_samples / 2, self.seed);
        let pilot_result = pilot.run(params.clone(), ds);

        // ── Phase 2: adapt proposals ──────────────────────────────────────────
        let mut adapted = params.clone();
        for (i, p) in adapted.iter_mut().enumerate() {
            let sd = pilot_result.posterior_sd[i];
            if sd > 1e-12 {
                p.proposal_sd = (scale * sd).clamp(p.prior_sd * 0.01, p.prior_sd * 5.0);
            }
            // Warm-start from pilot posterior mean
            p.value = pilot_result.posterior_mean[i].clamp(p.min, p.max);
        }

        // ── Phase 3: main run with adapted proposals ──────────────────────────
        self.run(adapted, ds)
    }
}

// ── Training fitness ──────────────────────────────────────────────────────────

/// Compute R² and RMSE using scale-anchored trend comparison.
///
/// Each biomarker's predicted trajectory is anchored to the reference at age 20
/// (single scale factor), then R² is computed across all data points.
/// This measures trend-matching quality — whether the model correctly predicts
/// the rate of age-dependent change, up to a single multiplicative calibration
/// constant at young-adult baseline.
pub fn training_fitness(
    params: &[CalibrationParam],
    ds: &ReferenceDatasets,
) -> (f64, f64) {
    let snaps = match run_simulation(params) {
        Some(s) => s,
        None    => return (0.0, f64::INFINITY),
    };
    let snaps_ros = normalise_ros(&snaps);

    let datasets: &[(&CalibrationDataset, &str)] = &[
        (&ds.ros,      "ROS level"),
        (&ds.chip_vaf, "CHIP VAF"),
        (&ds.mcai,  "MCAI"),
    ];

    let mut all_obs  = Vec::new();
    let mut all_pred = Vec::new();

    for (dataset, biomarker) in datasets {
        let snap_src: &[cell_dt_aging_engine::AgeSnapshot] = if *biomarker == "ROS level" {
            snaps_ros.as_slice()
        } else {
            snaps.as_slice()
        };

        let ref_at_20 = dataset.observed[0];
        let sf = scale_factor_at_20(snap_src, biomarker, ref_at_20);

        for (i, &age) in dataset.ages.iter().enumerate() {
            if let Some(raw) = extract_biomarker(snap_src, age, biomarker) {
                all_obs.push(dataset.observed[i]);
                all_pred.push(raw * sf);
            }
        }
    }

    (
        Calibrator::calculate_r2(&all_obs, &all_pred),
        Calibrator::calculate_rmse(&all_obs, &all_pred),
    )
}

// ── Sensitivity analysis ─────────────────────────────────────────────────────

/// One row of a sensitivity analysis result.
#[derive(Debug, Clone)]
pub struct SensitivityRow {
    pub param_name:    &'static str,
    /// Relative perturbation applied (e.g. +0.10 = +10%)
    pub delta_frac:    f64,
    /// R² at the perturbed parameter value
    pub r2_perturbed:  f64,
    /// ΔR² = r2_perturbed − r2_baseline
    pub delta_r2:      f64,
}

/// One-at-a-time sensitivity analysis.
///
/// For each calibration parameter and each perturbation level in `deltas`
/// (fractional, e.g. `&[-0.20, -0.10, 0.10, 0.20]`), one parameter is shifted
/// while all others stay at baseline, and R² is re-evaluated.
///
/// Returns one `SensitivityRow` per (parameter, perturbation) combination.
pub fn sensitivity_analysis(
    params:  &[CalibrationParam],
    ds:      &ReferenceDatasets,
    deltas:  &[f64],
) -> Vec<SensitivityRow> {
    let (r2_baseline, _) = training_fitness(params, ds);
    let mut rows = Vec::new();

    for (i, base) in params.iter().enumerate() {
        for &delta in deltas {
            let mut perturbed = params.to_vec();
            let new_val = (base.value * (1.0 + delta)).clamp(base.min, base.max);
            perturbed[i].value = new_val;
            let (r2, _) = training_fitness(&perturbed, ds);
            rows.push(SensitivityRow {
                param_name:   base.name,
                delta_frac:   delta,
                r2_perturbed: r2,
                delta_r2:     r2 - r2_baseline,
            });
        }
    }
    rows
}

// ── Original Calibrator (R² / RMSE utilities — preserved) ────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    // ── Calibrator utility tests (preserved) ─────────────────────────────────

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

    #[test]
    fn test_r2_empty_returns_zero() {
        assert_eq!(Calibrator::calculate_r2(&[], &[]), 0.0);
    }

    #[test]
    fn test_r2_mismatched_lengths_returns_zero() {
        assert_eq!(Calibrator::calculate_r2(&[1.0, 2.0], &[1.0]), 0.0);
    }

    #[test]
    fn test_r2_negative_for_terrible_fit() {
        let obs  = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let pred = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        assert!(Calibrator::calculate_r2(&obs, &pred) < 0.0);
    }

    #[test]
    fn test_r2_known_value() {
        let obs  = vec![2.0, 4.0, 5.0, 4.0];
        let pred = vec![2.1, 3.9, 5.2, 3.8];
        assert!(Calibrator::calculate_r2(&obs, &pred) > 0.9);
    }

    #[test]
    fn test_r2_predicting_mean_gives_zero() {
        let obs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = obs.iter().sum::<f64>() / obs.len() as f64;
        let pred = vec![mean; 5];
        assert!(Calibrator::calculate_r2(&obs, &pred).abs() < 1e-9);
    }

    #[test]
    fn test_rmse_empty_returns_infinity() {
        assert!(Calibrator::calculate_rmse(&[], &[]).is_infinite());
    }

    #[test]
    fn test_rmse_known_value() {
        let obs  = vec![1.0, 2.0, 3.0, 4.0];
        let pred = vec![2.0, 1.0, 4.0, 3.0];
        assert!((Calibrator::calculate_rmse(&obs, &pred) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_rmse_symmetric() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.5, 1.8, 3.3];
        let rmse_ab = Calibrator::calculate_rmse(&a, &b);
        let rmse_ba = Calibrator::calculate_rmse(&b, &a);
        assert!((rmse_ab - rmse_ba).abs() < 1e-9);
    }

    #[test]
    fn test_rmse_uniform_offset() {
        let obs:  Vec<f64> = (0..10).map(|i| i as f64 * 0.1).collect();
        let pred: Vec<f64> = obs.iter().map(|x| x + 0.01).collect();
        assert!((Calibrator::calculate_rmse(&obs, &pred) - 0.01).abs() < 1e-9);
    }

    #[test]
    fn test_r2_chip_model() {
        let obs  = vec![0.005, 0.015, 0.040, 0.070, 0.120];
        let pred = vec![0.006, 0.014, 0.042, 0.068, 0.115];
        assert!(Calibrator::calculate_r2(&obs, &pred) > 0.99);
    }

    #[test]
    fn test_r2_ros_model() {
        let obs  = vec![0.15, 0.25, 0.45, 0.65];
        let pred = vec![0.16, 0.24, 0.46, 0.63];
        assert!(Calibrator::calculate_r2(&obs, &pred) > 0.99);
    }

    #[test]
    fn test_training_range_lower_less_than_upper() {
        let c = Calibrator::new();
        assert!(c.training_age_range.0 < c.training_age_range.1);
    }

    // ── CalibrationParam tests ────────────────────────────────────────────────

    #[test]
    fn test_calibration_param_log_prior_at_mean() {
        let p = CalibrationParam {
            name: "alpha", value: 0.0082, prior_mean: 0.0082, prior_sd: 0.002,
            proposal_sd: 0.0020, min: 0.001, max: 0.05,
        };
        // At the mean: log_prior = 0.0 (unnormalised)
        assert!((p.log_prior() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_calibration_param_log_prior_decreases_away_from_mean() {
        let mut p = CalibrationParam {
            name: "alpha", value: 0.0082, prior_mean: 0.0082, prior_sd: 0.002,
            proposal_sd: 0.0020, min: 0.001, max: 0.05,
        };
        let lp_center = p.log_prior();
        p.value = 0.0120;  // 1.9σ away
        let lp_far = p.log_prior();
        assert!(lp_far < lp_center, "log prior should decrease away from mean");
    }

    #[test]
    fn test_default_calibration_params_count() {
        // 2 free parameters: tau_protection, pi_0
        // Fixed: alpha (collinear r=0.858), hsc_nu (ΔR²≈0), dnmt3a_fitness (ΔR²≈0)
        let params = default_calibration_params();
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_default_calibration_params_names() {
        let params = default_calibration_params();
        let names: Vec<&str> = params.iter().map(|p| p.name).collect();
        assert!(names.contains(&"tau_protection"));
        assert!(names.contains(&"pi_0"));
        // Fixed params must NOT appear in MCMC list
        assert!(!names.contains(&"alpha"),         "alpha must be fixed");
        assert!(!names.contains(&"hsc_nu"),        "hsc_nu must be fixed");
        assert!(!names.contains(&"dnmt3a_fitness"),"dnmt3a_fitness must be fixed");
    }

    #[test]
    fn test_default_calibration_params_bounds_valid() {
        for p in default_calibration_params() {
            assert!(p.min < p.max, "{}: min >= max", p.name);
            assert!(p.value >= p.min && p.value <= p.max,
                "{}: value {} not in [{}, {}]", p.name, p.value, p.min, p.max);
        }
    }

    #[test]
    fn test_default_calibration_params_proposal_sd_positive() {
        for p in default_calibration_params() {
            assert!(p.proposal_sd > 0.0, "{}: proposal_sd must be > 0", p.name);
        }
    }

    // ── Simulation and likelihood tests ──────────────────────────────────────

    #[test]
    fn test_run_simulation_returns_100_snapshots() {
        let params = default_calibration_params();
        let snaps = run_simulation(&params);
        assert!(snaps.is_some(), "simulation should succeed with default params");
        let snaps = snaps.unwrap();
        // 101 snapshots: age 0, 1, ..., 100
        assert!(snaps.len() >= 100 && snaps.len() <= 102,
            "expected ~101 snapshots, got {}", snaps.len());
    }

    #[test]
    fn test_run_simulation_biomarkers_in_range() {
        let params = default_calibration_params();
        let snaps = run_simulation(&params).unwrap();
        for s in &snaps {
            assert!(s.ros_level >= 0.0, "ROS must be non-negative");
            assert!(s.mcai >= 0.0 && s.mcai <= 1.0,
                "frailty must be in [0,1], got {}", s.mcai);
            assert!(s.telomere_length >= 0.0, "telomere must be non-negative");
        }
    }

    #[test]
    fn test_normalise_ros_is_one_at_age_20() {
        let params = default_calibration_params();
        let snaps = run_simulation(&params).unwrap();
        let normed = normalise_ros(&snaps);
        let v = normed.iter().find(|s| (s.age_years - 20.0).abs() < 1.5)
            .map(|s| s.ros_level).unwrap();
        assert!((v - 1.0).abs() < 1e-6, "normalised ROS at age 20 = {}", v);
    }

    #[test]
    fn test_scale_factor_at_20_ros_positive() {
        // scale_factor_at_20 should return a positive finite value for ROS
        let params = default_calibration_params();
        let snaps = run_simulation(&params).unwrap();
        let sf = scale_factor_at_20(&snaps, "ROS level", 1.0);
        assert!(sf > 0.0 && sf.is_finite(), "scale_factor_at_20 for ROS should be positive finite, got {}", sf);
    }

    #[test]
    fn test_scale_factor_at_20_anchors_ros_to_one() {
        // After applying scale_factor, sim ROS at age 20 should match reference (1.0)
        let params = default_calibration_params();
        let snaps = run_simulation(&params).unwrap();
        let sf = scale_factor_at_20(&snaps, "ROS level", 1.0);
        let sim_at_20 = snaps.iter()
            .find(|s| (s.age_years - 20.0).abs() < 1.5)
            .map(|s| s.ros_level)
            .unwrap();
        let anchored = sim_at_20 * sf;
        assert!((anchored - 1.0).abs() < 1e-2, "anchored ROS at age 20 should be ~1.0, got {}", anchored);
    }

    #[test]
    fn test_log_posterior_finite_at_default_params() {
        let params = default_calibration_params();
        let ds = ReferenceDatasets::load();
        let lp = log_posterior(&params, &ds);
        assert!(lp.is_finite(), "log posterior should be finite at default params, got {}", lp);
    }

    #[test]
    fn test_training_fitness_r2_positive() {
        let params = default_calibration_params();
        let ds = ReferenceDatasets::load();
        let (r2, rmse) = training_fitness(&params, &ds);
        assert!(r2.is_finite(), "R² should be finite");
        assert!(rmse.is_finite() && rmse >= 0.0, "RMSE should be finite non-negative");
    }

    // ── R-hat tests ───────────────────────────────────────────────────────────

    #[test]
    fn test_r_hat_converged_chain() {
        // Stationary chain: alternates around 0.5 with tiny symmetric noise
        // Both halves have the same mean → R-hat ≈ 1.0
        let chain: Vec<f64> = (0..100).map(|i| 0.5 + (i % 2) as f64 * 0.001 - 0.0005).collect();
        let rh = r_hat_single(&chain);
        assert!(rh < 1.05, "converged chain should have R-hat < 1.05, got {}", rh);
    }

    #[test]
    fn test_r_hat_diverged_chain() {
        // First half around 0, second half around 10 — clear non-stationarity
        // Both halves have internal variance so W > 0
        let chain: Vec<f64> = (0..100).map(|i| {
            if i < 50 { (i % 3) as f64 * 0.01 } else { 10.0 + (i % 3) as f64 * 0.01 }
        }).collect();
        let rh = r_hat_single(&chain);
        assert!(rh > 1.05, "diverged chain should have R-hat > 1.05, got {}", rh);
    }

    #[test]
    fn test_r_hat_short_chain_returns_nan() {
        let chain = vec![1.0, 2.0, 3.0];
        let rh = r_hat_single(&chain);
        assert!(rh.is_nan(), "R-hat of short chain should be NaN");
    }

    // ── Metropolis tests ──────────────────────────────────────────────────────

    #[test]
    fn test_metropolis_default() {
        let m = Metropolis::default();
        assert_eq!(m.burn_in, 200);
        assert_eq!(m.n_samples, 500);
    }

    #[test]
    fn test_metropolis_short_run_completes() {
        // Small run: 20 burn-in + 30 samples
        let mcmc = Metropolis::new(20, 30, 99);
        let params = default_calibration_params();
        let ds = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        assert_eq!(result.samples.len(), 30);
        assert_eq!(result.param_names.len(), 2);
    }

    #[test]
    fn test_metropolis_acceptance_rate_in_range() {
        let mcmc = Metropolis::new(20, 50, 7);
        let params = default_calibration_params();
        let ds = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        assert!(result.acceptance_rate >= 0.0 && result.acceptance_rate <= 1.0,
            "acceptance rate = {}", result.acceptance_rate);
    }

    #[test]
    fn test_metropolis_posterior_mean_near_prior() {
        // With a short run and reasonable priors, posterior mean stays near prior mean
        let mcmc = Metropolis::new(10, 40, 42);
        let params = default_calibration_params();
        let ds = ReferenceDatasets::load();
        let result = mcmc.run(params.clone(), &ds);
        for (i, p) in params.iter().enumerate() {
            let mean = result.posterior_mean[i];
            assert!(mean >= p.min && mean <= p.max,
                "{}: posterior mean {} out of bounds [{}, {}]", p.name, mean, p.min, p.max);
        }
    }

    #[test]
    fn test_metropolis_posterior_sd_non_negative() {
        let mcmc = Metropolis::new(10, 40, 11);
        let params = default_calibration_params();
        let ds = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        for (i, &sd) in result.posterior_sd.iter().enumerate() {
            assert!(sd >= 0.0, "param {}: posterior sd must be non-negative, got {}", i, sd);
        }
    }

    #[test]
    fn test_metropolis_r2_finite() {
        let mcmc = Metropolis::new(10, 20, 5);
        let params = default_calibration_params();
        let ds = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        assert!(result.r2_training.is_finite(),
            "training R² should be finite, got {}", result.r2_training);
        assert!(result.rmse_training.is_finite() && result.rmse_training >= 0.0,
            "training RMSE should be finite non-negative, got {}", result.rmse_training);
    }

    #[test]
    fn test_metropolis_samples_param_values_in_bounds() {
        let mcmc = Metropolis::new(10, 30, 3);
        let params = default_calibration_params();
        let bounds: Vec<(f64, f64)> = params.iter().map(|p| (p.min, p.max)).collect();
        let ds = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        for sample in &result.samples {
            for (i, &v) in sample.param_values.iter().enumerate() {
                assert!(v >= bounds[i].0 && v <= bounds[i].1,
                    "param {} value {} out of bounds", i, v);
            }
        }
    }

    #[test]
    fn test_metropolis_r_hat_vector_length() {
        let mcmc = Metropolis::new(10, 40, 17);
        let params = default_calibration_params();
        let ds = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        assert_eq!(result.r_hat.len(), 2, "R-hat vector should have one entry per parameter");
    }

    #[test]
    fn test_apply_params_tau_protection() {
        let mut params = default_calibration_params();
        // params[0] is tau_protection
        params[0].value = 30.0;
        let mut fp = FixedParameters::default();
        apply_params(&params, &mut fp);
        assert!((fp.tau_protection - 30.0).abs() < 1e-9);
    }

    #[test]
    fn test_apply_params_two_free() {
        let params = default_calibration_params();
        let mut fp = FixedParameters::default();
        apply_params(&params, &mut fp);
        // Free params applied from calibration defaults
        assert!((fp.tau_protection - 24.3).abs() < 1e-9);
        assert!((fp.pi_0 - 0.87).abs() < 1e-9);
        // Fixed params remain at FixedParameters defaults
        assert!((fp.alpha - 0.0082).abs() < 1e-10,  "alpha must remain fixed at 0.0082");
        assert!((fp.hsc_nu - 1.2).abs() < 1e-9,     "hsc_nu must remain fixed at 1.2 (Wilson 2008 standard)");
        assert!((fp.dnmt3a_fitness - 0.15).abs() < 1e-9, "dnmt3a_fitness must remain fixed");
    }

    // ── correlation_matrix tests ──────────────────────────────────────────────

    #[test]
    fn test_correlation_matrix_diagonal_is_one() {
        let mcmc   = Metropolis::new(20, 50, 7);
        let params = default_calibration_params();
        let ds     = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        let n      = result.param_names.len();
        let corr   = result.correlation_matrix();
        for i in 0..n {
            assert!((corr[i * n + i] - 1.0).abs() < 1e-9,
                "diagonal[{}] should be 1.0, got {}", i, corr[i * n + i]);
        }
    }

    #[test]
    fn test_correlation_matrix_symmetric() {
        let mcmc   = Metropolis::new(20, 50, 8);
        let params = default_calibration_params();
        let ds     = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        let n      = result.param_names.len();
        let corr   = result.correlation_matrix();
        for i in 0..n {
            for j in 0..n {
                assert!((corr[i * n + j] - corr[j * n + i]).abs() < 1e-9,
                    "corr[{},{}] != corr[{},{}]", i, j, j, i);
            }
        }
    }

    #[test]
    fn test_correlation_matrix_values_in_range() {
        let mcmc   = Metropolis::new(20, 50, 9);
        let params = default_calibration_params();
        let ds     = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        let corr   = result.correlation_matrix();
        for &r in &corr {
            assert!(r >= -1.0 && r <= 1.0, "correlation out of [-1,1]: {}", r);
        }
    }

    #[test]
    fn test_correlation_matrix_size() {
        let mcmc   = Metropolis::new(10, 20, 1);
        let params = default_calibration_params();
        let ds     = ReferenceDatasets::load();
        let result = mcmc.run(params, &ds);
        let n      = result.param_names.len();
        assert_eq!(result.correlation_matrix().len(), n * n);
    }

    // ── sensitivity_analysis tests ────────────────────────────────────────────

    #[test]
    fn test_sensitivity_analysis_row_count() {
        let params = default_calibration_params();
        let ds     = ReferenceDatasets::load();
        let deltas = [-0.10_f64, 0.10];
        let rows   = sensitivity_analysis(&params, &ds, &deltas);
        assert_eq!(rows.len(), params.len() * deltas.len(),
            "expected {} rows, got {}", params.len() * deltas.len(), rows.len());
    }

    #[test]
    fn test_sensitivity_analysis_baseline_delta_zero() {
        // Zero perturbation → ΔR² should be 0
        let params = default_calibration_params();
        let ds     = ReferenceDatasets::load();
        let rows   = sensitivity_analysis(&params, &ds, &[0.0]);
        for row in &rows {
            assert!(row.delta_r2.abs() < 1e-9,
                "{}: delta_r2 should be 0 at zero perturbation, got {}", row.param_name, row.delta_r2);
        }
    }

    #[test]
    fn test_sensitivity_analysis_r2_finite() {
        let params = default_calibration_params();
        let ds     = ReferenceDatasets::load();
        let rows   = sensitivity_analysis(&params, &ds, &[-0.10, 0.10]);
        for row in &rows {
            assert!(row.r2_perturbed.is_finite(),
                "{}: r2_perturbed should be finite", row.param_name);
        }
    }

    #[test]
    fn test_sensitivity_analysis_param_names_match() {
        let params = default_calibration_params();
        let names: Vec<&str> = params.iter().map(|p| p.name).collect();
        let ds   = ReferenceDatasets::load();
        let rows = sensitivity_analysis(&params, &ds, &[-0.10, 0.10]);
        // Each param should appear in the rows
        for name in &names {
            assert!(rows.iter().any(|r| r.param_name == *name),
                "parameter {} not found in sensitivity rows", name);
        }
    }

    #[test]
    fn test_log_posterior_worse_for_extreme_alpha() {
        let ds = ReferenceDatasets::load();
        let params_default = default_calibration_params();
        let lp_default = log_posterior(&params_default, &ds);

        let mut params_bad = default_calibration_params();
        params_bad[0].value = 60.0; // very high tau_protection — far from prior and data
        let lp_bad = log_posterior(&params_bad, &ds);

        assert!(lp_default > lp_bad,
            "default params (lp={:.2}) should have higher posterior than extreme tau_protection (lp={:.2})",
            lp_default, lp_bad);
    }
}
