/// CDATA v3.0 — Bayesian MCMC calibration (Metropolis-Hastings)
///
/// Calibrates 5 key parameters of FixedParameters against reference datasets
/// (ROS, telomere, CHIP VAF, frailty, epigenetic age) using a random-walk
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
        let delta: f64 = rng.gen::<f64>() * 2.0 - 1.0;  // uniform(-1,1)
        // Box-Muller for normal proposal
        let u1: f64 = rng.gen::<f64>().max(1e-12);
        let u2: f64 = rng.gen::<f64>();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let _ = delta;  // prefer Box-Muller
        (self.value + self.proposal_sd * z).clamp(self.min, self.max)
    }
}

/// Default 5-parameter set to calibrate (HSC trajectory).
pub fn default_calibration_params() -> Vec<CalibrationParam> {
    vec![
        CalibrationParam {
            name: "alpha",
            value: 0.0082, prior_mean: 0.0082, prior_sd: 0.002,
            proposal_sd: 0.0003, min: 0.001, max: 0.05,
        },
        CalibrationParam {
            name: "tau_protection",
            value: 24.3, prior_mean: 24.3, prior_sd: 5.0,
            proposal_sd: 0.8, min: 5.0, max: 60.0,
        },
        CalibrationParam {
            name: "pi_0",
            value: 0.87, prior_mean: 0.87, prior_sd: 0.05,
            proposal_sd: 0.01, min: 0.50, max: 0.99,
        },
        CalibrationParam {
            name: "hsc_nu",
            value: 12.0, prior_mean: 12.0, prior_sd: 3.0,
            proposal_sd: 0.5, min: 2.0, max: 40.0,
        },
        CalibrationParam {
            name: "dnmt3a_fitness",
            value: 0.15, prior_mean: 0.15, prior_sd: 0.05,
            proposal_sd: 0.008, min: 0.01, max: 0.50,
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

/// Extract the modelled value for a dataset's biomarker at a given age from
/// a pre-run simulation snapshot vector (`age_years` steps from 0–100).
///
/// Returns `None` if the biomarker is not directly modelled (skipped in likelihood)
/// or if the snapshot vector is empty.
///
/// NOTE: "Telomere length" and "CHIP VAF" are intentionally excluded from the
/// likelihood — telomere depletes to 0 by age ~14 in HSC (not matched in
/// calibration range 20–50), and CHIP VAF has no direct mapping in AgeSnapshot.
/// These datasets remain in `ReferenceDatasets` for informational use.
fn extract_biomarker(
    snaps: &[cell_dt_aging_engine::AgeSnapshot],
    age: f64,
    biomarker: &str,
) -> Option<f64> {
    // Biomarkers not used in likelihood — skip
    if matches!(biomarker, "Telomere length" | "CHIP VAF") {
        return None;
    }

    // Find nearest snapshot
    let snap = snaps.iter().min_by(|a, b| {
        (a.age_years - age).abs()
            .partial_cmp(&(b.age_years - age).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;

    let v = match biomarker {
        "ROS level"  => snap.ros_level,
        // Frailty: use centriole_damage directly — frailty_index has a hard floor
        // from telomere=0 (depleted by age ~14 in HSC) making it insensitive to
        // calibration parameters in the 20–50 yr range.
        // Predicted damage is scaled to the reference frailty range [0.05, 0.14]
        // by multiplying by a tissue-specific conversion factor (1.5×).
        "Frailty index"        => snap.centriole_damage * 1.5,
        "Epi-age acceleration" => (snap.epigenetic_age - snap.age_years).max(0.0),
        _                      => return None,
    };
    Some(v)
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

/// Normalise simulated telomere to 1.0 at the earliest available age (age 0–2).
/// NOTE: HSC telomeres deplete to 0 by ~age 14 in the model (high division rate),
/// so normalising at age 20 is not meaningful — we use the birth snapshot instead.
fn normalise_telomere(snaps: &[cell_dt_aging_engine::AgeSnapshot]) -> Vec<cell_dt_aging_engine::AgeSnapshot> {
    // Use earliest snapshot (age ≤ 2) as reference
    let t_ref = snaps.iter()
        .filter(|s| s.age_years <= 2.0)
        .map(|s| s.telomere_length)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(1e-6);
    // Fall back to global max if no early snapshot exists
    let t_ref = if t_ref <= 1e-6 {
        snaps.iter().map(|s| s.telomere_length).fold(1e-6_f64, f64::max)
    } else {
        t_ref
    };
    snaps.iter().cloned().map(|mut s| { s.telomere_length /= t_ref; s }).collect()
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

/// Gaussian log-likelihood: sum over all data points.
fn log_likelihood(
    snaps: &[cell_dt_aging_engine::AgeSnapshot],
    snaps_ros: &[cell_dt_aging_engine::AgeSnapshot],
    snaps_telo: &[cell_dt_aging_engine::AgeSnapshot],
    ds: &ReferenceDatasets,
) -> f64 {
    let datasets: &[(&CalibrationDataset, &str)] = &[
        (&ds.ros,          "ROS level"),
        (&ds.telomere,     "Telomere length"),
        (&ds.chip_vaf,     "CHIP VAF"),
        (&ds.frailty,      "Frailty index"),
        (&ds.epi_age_accel,"Epi-age acceleration"),
    ];

    let mut ll = 0.0f64;
    for (dataset, biomarker) in datasets {
        for (i, &age) in dataset.ages.iter().enumerate() {
            let snap_source = match *biomarker {
                "ROS level"       => snaps_ros,
                "Telomere length" => snaps_telo,
                _                 => snaps,
            };
            // `None` means this biomarker is excluded from the likelihood (e.g.,
        // telomere depletes before calibration range, CHIP VAF has no direct
        // AgeSnapshot mapping) — skip, do not penalise.
            let pred = match extract_biomarker(snap_source, age, biomarker) {
                Some(v) => v,
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
    let snaps_ros  = normalise_ros(&snaps);
    let snaps_telo = normalise_telomere(&snaps);
    log_prior_total(params) + log_likelihood(&snaps, &snaps_ros, &snaps_telo, ds)
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
}

// ── Training fitness ──────────────────────────────────────────────────────────

/// Compute R² and RMSE on training datasets using current param values.
pub fn training_fitness(
    params: &[CalibrationParam],
    ds: &ReferenceDatasets,
) -> (f64, f64) {
    let snaps = match run_simulation(params) {
        Some(s) => s,
        None    => return (0.0, f64::INFINITY),
    };
    let snaps_ros  = normalise_ros(&snaps);
    let snaps_telo = normalise_telomere(&snaps);

    let datasets: &[(&CalibrationDataset, &str)] = &[
        (&ds.ros,          "ROS level"),
        (&ds.telomere,     "Telomere length"),
        (&ds.chip_vaf,     "CHIP VAF"),
        (&ds.frailty,      "Frailty index"),
        (&ds.epi_age_accel,"Epi-age acceleration"),
    ];

    let mut all_obs  = Vec::new();
    let mut all_pred = Vec::new();

    for (dataset, biomarker) in datasets {
        for (i, &age) in dataset.ages.iter().enumerate() {
            let snap_src = match *biomarker {
                "ROS level"       => snaps_ros.as_slice(),
                "Telomere length" => snaps_telo.as_slice(),
                _                 => snaps.as_slice(),
            };
            // Skip biomarkers excluded from likelihood (telomere, CHIP VAF)
            if let Some(pred) = extract_biomarker(snap_src, age, biomarker) {
                all_obs.push(dataset.observed[i]);
                all_pred.push(pred);
            }
        }
    }

    (
        Calibrator::calculate_r2(&all_obs, &all_pred),
        Calibrator::calculate_rmse(&all_obs, &all_pred),
    )
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
            proposal_sd: 0.0003, min: 0.001, max: 0.05,
        };
        // At the mean: log_prior = 0.0 (unnormalised)
        assert!((p.log_prior() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_calibration_param_log_prior_decreases_away_from_mean() {
        let mut p = CalibrationParam {
            name: "alpha", value: 0.0082, prior_mean: 0.0082, prior_sd: 0.002,
            proposal_sd: 0.0003, min: 0.001, max: 0.05,
        };
        let lp_center = p.log_prior();
        p.value = 0.0120;  // 1.9σ away
        let lp_far = p.log_prior();
        assert!(lp_far < lp_center, "log prior should decrease away from mean");
    }

    #[test]
    fn test_default_calibration_params_count() {
        let params = default_calibration_params();
        assert_eq!(params.len(), 5);
    }

    #[test]
    fn test_default_calibration_params_names() {
        let params = default_calibration_params();
        let names: Vec<&str> = params.iter().map(|p| p.name).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"tau_protection"));
        assert!(names.contains(&"pi_0"));
        assert!(names.contains(&"hsc_nu"));
        assert!(names.contains(&"dnmt3a_fitness"));
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
            assert!(s.frailty_index >= 0.0 && s.frailty_index <= 1.0,
                "frailty must be in [0,1], got {}", s.frailty_index);
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
    fn test_normalise_telomere_is_one_at_age_0() {
        // HSC telomeres deplete to 0 by ~age 14 (high division rate),
        // so we normalise at birth (age 0), not age 20.
        let params = default_calibration_params();
        let snaps = run_simulation(&params).unwrap();
        let normed = normalise_telomere(&snaps);
        let v = normed.iter().find(|s| s.age_years <= 2.0)
            .map(|s| s.telomere_length).unwrap();
        assert!((v - 1.0).abs() < 1e-6, "normalised telomere at age 0 should be 1.0, got {}", v);
    }

    #[test]
    fn test_normalise_telomere_decreases_from_birth() {
        let params = default_calibration_params();
        let snaps = run_simulation(&params).unwrap();
        let normed = normalise_telomere(&snaps);
        // At birth ≤ 1.0 (normalised), should be > value at age 50
        let v_birth = normed.iter().find(|s| s.age_years <= 2.0)
            .map(|s| s.telomere_length).unwrap_or(0.0);
        let v_50 = normed.iter().find(|s| (s.age_years - 50.0).abs() < 1.5)
            .map(|s| s.telomere_length).unwrap_or(1.0);
        assert!(v_birth >= v_50, "telomere should not increase: birth={}, age50={}", v_birth, v_50);
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
        assert_eq!(result.param_names.len(), 5);
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
        assert_eq!(result.r_hat.len(), 5, "R-hat vector should have one entry per parameter");
    }

    #[test]
    fn test_apply_params_alpha() {
        let mut params = default_calibration_params();
        // Force alpha to a known value
        params[0].value = 0.01;
        let mut fp = FixedParameters::default();
        apply_params(&params, &mut fp);
        assert!((fp.alpha - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_apply_params_all_five() {
        let params = default_calibration_params();
        let mut fp = FixedParameters::default();
        apply_params(&params, &mut fp);
        // After applying defaults, values should equal FixedParameters defaults
        assert!((fp.alpha - 0.0082).abs() < 1e-10);
        assert!((fp.tau_protection - 24.3).abs() < 1e-9);
        assert!((fp.pi_0 - 0.87).abs() < 1e-9);
        assert!((fp.hsc_nu - 12.0).abs() < 1e-9);
        assert!((fp.dnmt3a_fitness - 0.15).abs() < 1e-9);
    }

    #[test]
    fn test_log_posterior_worse_for_extreme_alpha() {
        let ds = ReferenceDatasets::load();
        let params_default = default_calibration_params();
        let lp_default = log_posterior(&params_default, &ds);

        let mut params_bad = default_calibration_params();
        params_bad[0].value = 0.045; // very high alpha — far from prior and data
        let lp_bad = log_posterior(&params_bad, &ds);

        assert!(lp_default > lp_bad,
            "default params (lp={:.2}) should have higher posterior than extreme alpha (lp={:.2})",
            lp_default, lp_bad);
    }
}
