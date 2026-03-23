//! P61 — Agent Population Model Example
//!
//! 20 агентов-организмов (Blood HSC), симулируемых последовательно.
//! После каждого агента его SASP вносит системный вклад и буст-ирует следующих.
//!
//! Вывод: таблица агентов + сводная статистика.
//! CSV → population_output/agent_population.csv

use cell_dt_core::agent_population::{
    AgentPopulationParams, AgentPopulationStats, simulate_agent_population,
};
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params = AgentPopulationParams::default(); // n=20, transmission=0.05, radius=3

    println!("=== AGENT POPULATION MODEL (n={}) ===", params.n_agents);
    println!();
    println!("{:<10} {:>12} {:>18} {:>18}  {}",
        "Agent", "lifespan_yr", "SASP_received", "senes_frac", "cause");
    println!("{}", "─".repeat(78));

    let results = simulate_agent_population(&params);

    for r in &results {
        println!(
            "Agent {:02}: {:>9.1}y  {:>14.3}  {:>14.3}  {}",
            r.agent_id + 1,
            r.lifespan_years,
            r.received_sasp_burden,
            r.senescent_fraction_final,
            r.death_cause
        );
    }

    let stats = AgentPopulationStats::from_results(&results);

    println!();
    println!("=== POPULATION STATS ===");
    println!("Mean lifespan:        {:.1} ± {:.1} yr", stats.mean_lifespan, stats.sd_lifespan);
    println!("Mean received SASP:   {:.3}", stats.mean_received_sasp);
    println!("SASP-accelerated:     {:.1}% (agents with burden > 0.1)",
        stats.fraction_sasp_accelerated * 100.0);

    // --- Сохранить CSV ---
    let out_dir = "population_output";
    fs::create_dir_all(out_dir)?;
    let path = format!("{}/agent_population.csv", out_dir);
    let mut f = fs::File::create(&path)?;
    writeln!(f, "agent_id,lifespan_years,caii_final,senescent_fraction_final,received_sasp_burden,death_cause")?;
    for r in &results {
        writeln!(f, "{},{:.4},{:.4},{:.4},{:.4},{}",
            r.agent_id,
            r.lifespan_years,
            r.caii_final,
            r.senescent_fraction_final,
            r.received_sasp_burden,
            r.death_cause
        )?;
    }
    println!();
    println!("CSV saved: {}", path);

    Ok(())
}
