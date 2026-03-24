//! P68 — Parametric sweep: initial inducer counts vs lifespan.
//!
//! Sweeps M₀ ∈ {4, 6, 8, 10, 12, 15} and D₀ ∈ {4, 6, 8, 10} → 24 combinations.
//!
//! For each combination:
//! - Runs a 105-year deterministic simulation (Blood HSC, time_acceleration=365, 1 step=1 year)
//! - Records lifespan (when organism dies, or 105.0 if survives)
//! - Records CAII at age 60
//!
//! Results are printed as a table sorted by M₀, D₀ and saved to
//! `inducer_sweep_output/sweep_results.csv`.

use cell_dt_core::{SimulationManager, SimulationConfig};
use cell_dt_core::components::{CentriolePair, CellCycleStateExtended};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams, HumanDevelopmentComponent,
};
use myeloid_shift_module::MyeloidShiftModule;
use std::io::Write;
use std::fs;

const MAX_AGE_YEARS: usize = 105;
const MAX_STEPS: u64 = 105;
const TIME_ACC: f64 = 365.0;

struct SweepResult {
    m0: u32,
    d0: u32,
    lifespan: f32,
    caii_at_60: f32,
}

fn simulate_one(m0: u32, d0: u32) -> Result<SweepResult, Box<dyn std::error::Error>> {
    let config = SimulationConfig {
        max_steps: MAX_STEPS,
        dt: 1.0,
        checkpoint_interval: MAX_STEPS,
        num_threads: None,
        seed: Some(42),
        parallel_modules: false,
        cleanup_dead_interval: None,
    };

    let mut sim = SimulationManager::new(config);

    sim.register_module(Box::new(CentrioleModule::with_parallel(false)))?;
    sim.register_module(Box::new(CellCycleModule::with_params(CellCycleParams {
        checkpoint_strictness: 0.0,
        ..Default::default()
    })))?;
    sim.register_module(Box::new(HumanDevelopmentModule::with_params(HumanDevelopmentParams {
        time_acceleration: TIME_ACC,
        mother_inducer_count: m0,
        daughter_inducer_count: d0,
        base_detach_probability: 0.0003,
        noise_scale: 0.0,
        enable_inducer_system: true,
        ..Default::default()
    })))?;
    sim.register_module(Box::new(MyeloidShiftModule::new()))?;

    {
        let world = sim.world_mut();
        world.spawn((
            CentriolePair::default(),
            CellCycleStateExtended::new(),
        ));
    }

    sim.initialize()?;

    let mut lifespan = MAX_AGE_YEARS as f32;
    let mut caii_at_60 = 0.0f32;

    for year in 0..MAX_AGE_YEARS {
        sim.step()?;

        let params = sim.get_module_params("human_development_module")?;

        let is_alive = params.get("organism_is_alive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if year == 60 {
            caii_at_60 = read_caii(&sim);
        }

        if !is_alive {
            let age = params.get("organism_age_years")
                .and_then(|v| v.as_f64())
                .unwrap_or(year as f64) as f32;
            if caii_at_60 == 0.0 {
                caii_at_60 = read_caii(&sim);
            }
            lifespan = age;
            break;
        }
    }

    // If survived to max age but CAII@60 not recorded (shouldn't happen, year 60 < 105)
    if caii_at_60 == 0.0 {
        caii_at_60 = read_caii(&sim);
    }

    Ok(SweepResult { m0, d0, lifespan, caii_at_60 })
}

fn read_caii(sim: &SimulationManager) -> f32 {
    let world = sim.world();
    for (_, comp) in world.query::<&HumanDevelopmentComponent>().iter() {
        if !comp.is_alive { continue; }
        let d = &comp.centriolar_damage;
        let caii = (d.cep164_integrity
            * d.cep89_integrity
            * d.ninein_integrity
            * d.cep170_integrity).powf(0.25);
        return caii;
    }
    1.0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cell DT Platform — Inducer Sweep (P68) ===");
    println!();
    println!("Sweep: M₀ ∈ {{4,6,8,10,12,15}}, D₀ ∈ {{4,6,8,10}} → 24 combinations");
    println!("Config: time_acceleration={}, max_steps={} (years), noise_scale=0.0 (deterministic)",
        TIME_ACC, MAX_STEPS);
    println!();

    let m0_values: &[u32] = &[4, 6, 8, 10, 12, 15];
    let d0_values: &[u32] = &[4, 6, 8, 10];

    let mut results: Vec<SweepResult> = Vec::new();

    for &m0 in m0_values {
        for &d0 in d0_values {
            print!("  M0={:2}  D0={:2}  ... ", m0, d0);
            std::io::stdout().flush()?;

            let r = simulate_one(m0, d0)?;
            println!("lifespan={:.1}y  CAII@60={:.3}", r.lifespan, r.caii_at_60);
            results.push(r);
        }
    }

    println!();
    println!("{:<4}  {:<4}  {:<10}  {:<8}", "M0", "D0", "Lifespan", "CAII@60");
    println!("{}", "-".repeat(32));
    for r in &results {
        println!("{:<4}  {:<4}  {:<10.1}  {:<8.3}", r.m0, r.d0, r.lifespan, r.caii_at_60);
    }

    // Save CSV
    fs::create_dir_all("inducer_sweep_output")?;
    let path = "inducer_sweep_output/sweep_results.csv";
    let mut csv = String::new();
    csv.push_str("m0,d0,lifespan_years,caii_at_60\n");
    for r in &results {
        csv.push_str(&format!("{},{},{:.2},{:.4}\n",
            r.m0, r.d0, r.lifespan, r.caii_at_60));
    }
    fs::write(path, &csv)?;
    println!();
    println!("CSV saved to: {}", path);

    Ok(())
}
