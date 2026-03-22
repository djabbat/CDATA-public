//! CDATA Stochastic Analysis — 10 симуляций с разными seed.
//!
//! Запускает полную 100-летнюю симуляцию развития человека 10 раз
//! с разными начальными состояниями (`noise_scale = 0.05`).
//! Воспроизводит тот же модульный стек, что и `human_development_example`
//! (без MyeloidShiftModule), чтобы сравнимо с калибровкой 78.4 лет.
//!
//! ## Что выводится
//! - Lifespan каждого прогона (год когда все ниши мертвы)
//! - Damage score @ 40 лет (mean ± SD)
//! - Frailty index @ 60 лет (mean ± SD) — у тех прогонов, где кто-то ещё жив
//! - Мин/макс/среднее/SD lifespan + 95% CI

use cell_dt_core::{SimulationManager, SimulationConfig};
use cell_dt_core::components::{CentriolePair, CellCycleStateExtended};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams,
    HumanDevelopmentComponent,
};
use std::io::Write;

const NUM_RUNS: u64 = 10;
const NOISE_SCALE: f32 = 0.05;
const CHECKPOINT_A: usize = 40;   // damage checkpoint
const CHECKPOINT_B: usize = 60;   // frailty checkpoint

struct RunResult {
    lifespan_years: f64,
    damage_at_40: f64,
    frailty_at_60: f64,   // NaN if organism died before year 60
}

fn run_simulation(seed: u64) -> Result<RunResult, Box<dyn std::error::Error>> {
    let config = SimulationConfig {
        max_steps: 40_000,
        dt: 1.0,
        checkpoint_interval: 3650,
        num_threads: Some(2),
        seed: Some(seed),
        parallel_modules: false,
        cleanup_dead_interval: None,
    };

    let mut sim = SimulationManager::new(config);

    sim.register_module(Box::new(CentrioleModule::with_parallel(false)))?;

    let cell_cycle_params = CellCycleParams {
        base_cycle_time:           24.0,
        growth_factor_sensitivity: 0.3,
        stress_sensitivity:        0.2,
        checkpoint_strictness:     0.1,
        enable_apoptosis:          true,
        nutrient_availability:     0.9,
        growth_factor_level:       0.8,
        random_variation:          0.2,
    };
    sim.register_module(Box::new(CellCycleModule::with_params(cell_cycle_params)))?;

    let dev_params = HumanDevelopmentParams {
        time_acceleration:            1.0,
        enable_aging:                 true,
        enable_morphogenesis:         true,
        tissue_detail_level:          3,
        mother_inducer_count:         10,
        daughter_inducer_count:       8,
        base_detach_probability:      0.0003,
        mother_bias:                  0.5,
        age_bias_coefficient:         0.0,
        ptm_exhaustion_scale:         0.001,
        de_novo_centriole_division:   4,
        meiotic_elimination_enabled:  true,
        noise_scale:                  NOISE_SCALE,
    };
    sim.register_module(Box::new(HumanDevelopmentModule::with_params(dev_params)))?;

    {
        let world = sim.world_mut();
        for _ in 0..5 {
            let _ = world.spawn((
                CentriolePair::default(),
                CellCycleStateExtended::new(),
            ));
        }
    }

    sim.initialize()?;

    let mut lifespan_years: f64 = 100.0;
    let mut damage_at_40: f64 = 0.0;
    let mut frailty_at_60: f64 = f64::NAN;

    'outer: for year in 0usize..100 {
        for _ in 0..365 {
            sim.step()?;
        }

        let world = sim.world();

        if year == CHECKPOINT_A {
            let mut q = world.query::<&HumanDevelopmentComponent>();
            let alive: Vec<f64> = q.iter()
                .filter(|(_, c)| c.is_alive)
                .map(|(_, c)| c.damage_score() as f64)
                .collect();
            if !alive.is_empty() {
                damage_at_40 = alive.iter().sum::<f64>() / alive.len() as f64;
            }
        }

        if year == CHECKPOINT_B {
            let mut q = world.query::<&HumanDevelopmentComponent>();
            let alive: Vec<f64> = q.iter()
                .filter(|(_, c)| c.is_alive)
                .map(|(_, c)| c.frailty() as f64)
                .collect();
            if !alive.is_empty() {
                frailty_at_60 = alive.iter().sum::<f64>() / alive.len() as f64;
            }
        }

        {
            let mut q = world.query::<&HumanDevelopmentComponent>();
            let any_alive = q.iter().any(|(_, c)| c.is_alive);
            if !any_alive {
                lifespan_years = year as f64;
                break 'outer;
            }
        }
    }

    Ok(RunResult { lifespan_years, damage_at_40, frailty_at_60 })
}

fn mean(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}

fn std_dev(data: &[f64]) -> f64 {
    let m = mean(data);
    let variance = data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / data.len() as f64;
    variance.sqrt()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CDATA Stochastic Analysis — {} runs, noise_scale = {:.2} ===\n",
        NUM_RUNS, NOISE_SCALE);
    println!("Theory: Centriolar Damage Accumulation (Jaba Tkemaladze, 2007–2023)");
    println!("Modules: CentrioleModule + CellCycleModule + HumanDevelopmentModule");
    println!("(same stack as human_development_example; calibration target: ~78.4 yr)\n");

    println!("{:<6} {:>14} {:>14} {:>14}",
        "Run", "Lifespan(yr)", "Damage@40yr", "Frailty@60yr");
    println!("{}", "-".repeat(52));

    let mut results: Vec<RunResult> = Vec::new();

    for seed in 0..NUM_RUNS {
        print!("  [{}] seed {:>2}... ", seed + 1, seed);
        std::io::stdout().flush()?;

        match run_simulation(seed) {
            Ok(r) => {
                let frailty_str = if r.frailty_at_60.is_nan() {
                    "  (died<60)".to_string()
                } else {
                    format!("{:>10.3}", r.frailty_at_60)
                };
                println!("{:>10.1} yr  {:>10.3}  {}",
                    r.lifespan_years, r.damage_at_40, frailty_str);
                results.push(r);
            }
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }
    }

    if results.is_empty() {
        println!("No results collected.");
        return Ok(());
    }

    let lifespans: Vec<f64> = results.iter().map(|r| r.lifespan_years).collect();
    let damages: Vec<f64>   = results.iter().map(|r| r.damage_at_40).collect();
    let frailties: Vec<f64> = results.iter()
        .filter(|r| !r.frailty_at_60.is_nan())
        .map(|r| r.frailty_at_60)
        .collect();

    let ls_mean = mean(&lifespans);
    let ls_sd   = std_dev(&lifespans);
    let ls_min  = lifespans.iter().cloned().fold(f64::INFINITY, f64::min);
    let ls_max  = lifespans.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let ls_ci   = 1.96 * ls_sd / (results.len() as f64).sqrt();

    println!("\n{}", "=".repeat(52));
    println!("=== STATISTICAL SUMMARY ({} simulations) ===", results.len());
    println!("{}", "=".repeat(52));
    println!();
    println!("Lifespan (years):");
    println!("  Mean ± SD :  {:.1} ± {:.1}", ls_mean, ls_sd);
    println!("  Min / Max :  {:.1} / {:.1}", ls_min, ls_max);
    println!("  Range     :  {:.1} years", ls_max - ls_min);
    println!("  95% CI    :  ({:.1}, {:.1})", ls_mean - ls_ci, ls_mean + ls_ci);
    println!();
    println!("Centriolar damage @ year {}:", CHECKPOINT_A);
    println!("  Mean ± SD :  {:.3} ± {:.3}", mean(&damages), std_dev(&damages));
    println!();
    if frailties.is_empty() {
        println!("Frailty @ year {}: N/A (all organisms died before year {})",
            CHECKPOINT_B, CHECKPOINT_B);
    } else {
        println!("Frailty @ year {} ({}/{} survived to this point):",
            CHECKPOINT_B, frailties.len(), results.len());
        println!("  Mean ± SD :  {:.3} ± {:.3}", mean(&frailties), std_dev(&frailties));
    }
    println!();

    let cv = ls_sd / ls_mean * 100.0;
    println!("Coefficient of variation (lifespan): {:.1}%", cv);
    if cv < 5.0 {
        println!("  → LOW variance — deterministic (check noise_scale)");
    } else if cv < 15.0 {
        println!("  → MODERATE variance — biologically plausible stochasticity");
    } else {
        println!("  → HIGH variance — consider reducing noise_scale");
    }

    println!();
    println!("Calibration target: ~78.4 yr");
    println!("Observed mean:       {:.1} yr  (delta: {:.1} yr)",
        ls_mean, ls_mean - 78.4);
    if (ls_mean - 78.4).abs() < 5.0 {
        println!("  ✓ Within ±5 yr of calibration — noise does not break calibration");
    } else {
        println!("  ! More than 5 yr from calibration — noise_scale may be too high");
    }

    Ok(())
}
