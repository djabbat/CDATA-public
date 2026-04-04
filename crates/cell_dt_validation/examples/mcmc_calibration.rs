/// CDATA v3.0 — MCMC Calibration example (Task 17 / Task 20)
///
/// Runs Metropolis-Hastings calibration against reference datasets
/// (ROS, telomere, CHIP VAF, MCAI, epigenetic age) and reports:
///  - Posterior mean ± SD for 5 key parameters
///  - Acceptance rate and R-hat convergence diagnostics
///  - Training R² and RMSE

use cell_dt_validation::{
    Metropolis, default_calibration_params, training_fitness, ReferenceDatasets,
};

fn main() {
    println!("=== CDATA v3.0 — MCMC Calibration (Metropolis-Hastings) ===\n");

    // ── Load reference datasets ───────────────────────────────────────────────
    let ds = ReferenceDatasets::load();
    println!("Reference datasets loaded:");
    for (name, n) in [
        (&ds.ros.name,          ds.ros.ages.len()),
        (&ds.telomere.name,     ds.telomere.ages.len()),
        (&ds.chip_vaf.name,     ds.chip_vaf.ages.len()),
        (&ds.mcai.name,      ds.mcai.ages.len()),
        (&ds.epi_age_accel.name,ds.epi_age_accel.ages.len()),
    ] {
        println!("  {:<25} {} data points", name, n);
    }

    // ── Prior fitness (default params) ────────────────────────────────────────
    let params = default_calibration_params();
    let (r2_prior, rmse_prior) = training_fitness(&params, &ds);
    println!("\nPrior (default) fitness:");
    println!("  R²   = {:.4}", r2_prior);
    println!("  RMSE = {:.6}", rmse_prior);

    // ── MCMC run ─────────────────────────────────────────────────────────────
    // 500 burn-in + 1000 samples (fast enough for CI; increase for production)
    let mcmc = Metropolis::new(500, 1000, 42);
    println!("\nRunning MCMC: {} burn-in + {} samples ...",
        mcmc.burn_in, mcmc.n_samples);

    let result = mcmc.run(params, &ds);

    // ── Results ───────────────────────────────────────────────────────────────
    println!("\n{:<20} {:>12} {:>10} {:>8}",
        "Parameter", "Post.mean", "Post.SD", "R-hat");
    println!("{}", "-".repeat(52));
    for (i, name) in result.param_names.iter().enumerate() {
        let rh = result.r_hat[i];
        let rh_str = if rh.is_nan()      { "  n/a".to_string() }
                     else if rh.is_infinite() { "  >10".to_string() }
                     else { format!("{:8.4}", rh) };
        println!("{:<20} {:>12.6} {:>10.6} {}",
            name,
            result.posterior_mean[i],
            result.posterior_sd[i],
            rh_str,
        );
    }

    println!("\nAcceptance rate : {:.1}%", result.acceptance_rate * 100.0);
    println!("Training R²     : {:.4}", result.r2_training);
    println!("Training RMSE   : {:.6}", result.rmse_training);

    // ── Convergence assessment ────────────────────────────────────────────────
    let converged = result.r_hat.iter().all(|&r| r.is_nan() || r < 1.05);
    let r2_ok     = result.r2_training > 0.80;

    println!("\n=== Summary ===");
    println!("Convergence (R-hat < 1.05) : {}", if converged { "✅ YES" } else { "⚠️  NOT YET" });
    println!("Validation (R² > 0.80)     : {}", if r2_ok { "✅ YES" } else { "⚠️  NO" });

    if !converged {
        println!("\nTip: increase n_samples (try 5000) for full convergence.");
    }
}
