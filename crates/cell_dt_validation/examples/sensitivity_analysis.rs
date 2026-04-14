//! Task 24 — One-at-a-time (OAT) sensitivity analysis.
//!
//! Each of the 5 calibrated parameters is perturbed ±10% and ±20% from its
//! default value (others held fixed).  ΔR² measures how sensitive the model
//! fit is to each parameter.  Large |ΔR²| → influential parameter.

use cell_dt_validation::{default_calibration_params, sensitivity_analysis, ReferenceDatasets};

fn main() {
    let ds     = ReferenceDatasets::load();
    let params = default_calibration_params();

    let deltas = [-0.20_f64, -0.10, 0.10, 0.20];
    let rows   = sensitivity_analysis(&params, &ds, &deltas);

    // Baseline R²
    let baseline = rows.iter().map(|r| r.r2_perturbed - r.delta_r2).next().unwrap_or(0.0);
    println!("=== ONE-AT-A-TIME SENSITIVITY ANALYSIS ===");
    println!("Baseline R² (default parameters): {:.4}\n", baseline + rows[0].delta_r2 - rows[0].delta_r2);

    // Print the actual baseline
    use cell_dt_validation::training_fitness;
    let (r2_base, _) = training_fitness(&params, &ds);
    println!("Baseline R²: {:.4}\n", r2_base);

    println!("{:>20}  {:>8}  {:>10}  {:>10}  {:>8}",
        "Parameter", "Δ%", "R²", "ΔR²", "Rank");

    // Rank parameters by max |ΔR²| across all perturbations
    let n_params = params.len();
    let n_delta  = deltas.len();
    let mut max_abs: Vec<(f64, &str)> = (0..n_params).map(|i| {
        let max = rows[i * n_delta .. (i+1) * n_delta]
            .iter()
            .map(|r| r.delta_r2.abs())
            .fold(0.0_f64, f64::max);
        (max, rows[i * n_delta].param_name)
    }).collect();
    max_abs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let rank_of = |name: &str| max_abs.iter().position(|(_, n)| *n == name).unwrap_or(99) + 1;

    let mut current_param = "";
    for row in &rows {
        if row.param_name != current_param {
            if !current_param.is_empty() { println!(); }
            current_param = row.param_name;
        }
        println!("{:>20}  {:>+7.0}%  {:>10.4}  {:>+10.4}  {:>8}",
            row.param_name,
            row.delta_frac * 100.0,
            row.r2_perturbed,
            row.delta_r2,
            rank_of(row.param_name));
    }

    println!("\n=== PARAMETER INFLUENCE RANKING (by max |ΔR²|) ===");
    for (rank, (max_delta, name)) in max_abs.iter().enumerate() {
        let bar: String = "#".repeat((max_delta * 40.0) as usize);
        println!("  {}. {:>20}  max|ΔR²|={:.4}  {}", rank + 1, name, max_delta, bar);
    }

    println!("\nInterpretation:");
    println!("  High |ΔR²| → model output is sensitive to this parameter → calibrate carefully");
    println!("  Low  |ΔR²| → parameter is practically non-identifiable from these biomarkers");
}
