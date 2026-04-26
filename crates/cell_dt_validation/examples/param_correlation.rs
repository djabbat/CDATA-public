//! Task 23 — Parameter correlation matrix from MCMC posterior samples.
//!
//! Runs MCMC (500 burn-in + 2000 samples) and computes the Pearson correlation
//! matrix of the 5 calibrated parameters.  Off-diagonal |r| > 0.7 indicates
//! strong posterior correlation (potential identifiability concern for EIC review).

use cell_dt_validation::{default_calibration_params, Metropolis, ReferenceDatasets};

fn main() {
    let ds     = ReferenceDatasets::load();
    let params = default_calibration_params();

    println!("Running MCMC (500 burn-in + 2000 samples) ...");
    let mcmc   = Metropolis::new(500, 2000, 42);
    let result = mcmc.run(params, &ds);

    println!("Acceptance rate: {:.1}%", result.acceptance_rate * 100.0);
    println!("R² training (posterior mean): {:.4}\n", result.r2_training);

    let n    = result.param_names.len();
    let corr = result.correlation_matrix();

    // Header
    print!("{:>20}", "");
    for name in &result.param_names {
        print!("{:>14}", name);
    }
    println!();

    // Matrix rows
    for i in 0..n {
        print!("{:>20}", result.param_names[i]);
        for j in 0..n {
            let r = corr[i * n + j];
            let marker = if i != j && r.abs() > 0.7 { " !" } else { "  " };
            print!("{:>12.3}{}", r, marker);
        }
        println!();
    }

    println!("\n! = |r| > 0.7  (strong correlation — potential identifiability issue)");

    // Summary: flag any strong off-diagonal correlations
    let mut issues = Vec::new();
    for i in 0..n {
        for j in (i+1)..n {
            let r = corr[i * n + j];
            if r.abs() > 0.7 {
                issues.push((result.param_names[i], result.param_names[j], r));
            }
        }
    }

    if issues.is_empty() {
        println!("\n✅ No strong parameter correlations (all |r| ≤ 0.70) — parameters are identifiable.");
    } else {
        println!("\n⚠️  Strong correlations detected:");
        for (a, b, r) in &issues {
            println!("   {} ↔ {}  r = {:.3}", a, b, r);
        }
        println!("   Consider reparametrisation or fixing one parameter to prior mean.");
    }

    // R-hat convergence summary
    println!("\nR-hat convergence (< 1.05 = converged):");
    let all_converged = result.r_hat.iter().all(|&r| r < 1.05);
    for (name, &rh) in result.param_names.iter().zip(result.r_hat.iter()) {
        let flag = if rh < 1.05 { "✅" } else { "⚠️ " };
        println!("  {} {:>20}  R-hat = {:.4}", flag, name, rh);
    }
    if all_converged {
        println!("\n✅ All parameters converged (R-hat < 1.05).");
    } else {
        println!("\n⚠️  Some parameters not converged — increase n_samples or tune proposal_sd.");
    }
}
