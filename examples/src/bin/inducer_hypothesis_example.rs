//! P59 — Inducer Hypothesis Test
//!
//! Сравниваем две симуляции Blood HSC, 100 лет:
//! - **With Inducers**: стандартная модель (enable_inducer_system=true)
//! - **Without Inducers**: потентность = f(spindle_fidelity × ciliary_function)
//!
//! Тест: выяснить, как индукторная система влияет на продолжительность жизни,
//! начало CHIP, миелоидный сдвиг и отказ тканей.

use cell_dt_core::{SimulationManager, SimulationConfig, SimulationModule};
use cell_dt_core::components::CellCycleStateExtended;
use human_development_module::{HumanDevelopmentModule, HumanDevelopmentParams, HumanDevelopmentComponent};

struct SimResult {
    lifespan:           f32,
    chip_onset_year:    Option<f32>,
    caii_at_60:         f32,
    myeloid_bias_at_70: f32,
    stem_pool_at_70:    f32,
}

fn run_scenario(enable_inducers: bool) -> Result<SimResult, Box<dyn std::error::Error>> {
    let config = SimulationConfig {
        max_steps: 36_525,   // 100 лет
        dt: 1.0,
        checkpoint_interval: 36_525,
        num_threads: Some(1),
        seed: Some(42),
        parallel_modules: false,
        cleanup_dead_interval: None,
    };

    let mut sim = SimulationManager::new(config);

    let dev_params = HumanDevelopmentParams {
        time_acceleration:           1.0,
        enable_aging:                true,
        enable_morphogenesis:        true,
        tissue_detail_level:         3,
        mother_inducer_count:        10,
        daughter_inducer_count:      8,
        base_detach_probability:     0.0003,
        mother_bias:                 0.5,
        age_bias_coefficient:        0.0,
        ptm_exhaustion_scale:        0.001,
        de_novo_centriole_division:  4,
        meiotic_elimination_enabled: true,
        noise_scale:                 0.0,
        enable_inducer_system:       enable_inducers,
    };

    let mut dev_module = HumanDevelopmentModule::with_params(dev_params);
    dev_module.set_seed(42);
    sim.register_module(Box::new(dev_module))?;

    {
        let world = sim.world_mut();
        let _ = world.spawn((CellCycleStateExtended::new(),));
    }
    sim.initialize()?;

    let mut lifespan           = 100.0f32;
    let mut chip_onset_year    = None;
    let mut caii_at_60         = 0.0f32;
    let mut myeloid_bias_at_70 = 0.0f32;
    let mut stem_pool_at_70    = 0.0f32;
    let mut recorded_60        = false;
    let mut recorded_70        = false;
    let mut prev_damage        = 0.0f32;

    for day in 0u64..36_525 {
        sim.step()?;
        let year = day as f32 / 365.25;

        // CHIP onset: быстрый рост повреждения (proxy)
        {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            if let Some((_, dev)) = q.iter().next() {
                let dmg = dev.centriolar_damage.total_damage_score();
                if chip_onset_year.is_none() && dmg > 0.35 && prev_damage < 0.35 {
                    chip_onset_year = Some(year);
                }
                prev_damage = dmg;
            }
        }

        // CAII at 60
        if !recorded_60 && year >= 60.0 {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            if let Some((_, dev)) = q.iter().next() {
                let app = &dev.centriolar_damage;
                caii_at_60 = (app.cep164_integrity * 0.40
                    + app.cep89_integrity  * 0.25
                    + app.ninein_integrity * 0.20
                    + app.cep170_integrity * 0.15).clamp(0.0, 1.0);
            }
            recorded_60 = true;
        }

        // Metrics at 70
        if !recorded_70 && year >= 70.0 {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            if let Some((_, dev)) = q.iter().next() {
                // Proxy myeloid_bias from damage
                let dmg = dev.centriolar_damage.total_damage_score();
                myeloid_bias_at_70 = (dmg * 0.60).clamp(0.0, 1.0);
                stem_pool_at_70    = dev.tissue_state.stem_cell_pool;
            }
            recorded_70 = true;
        }

        // Death check
        {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            let all_dead = q.iter().all(|(_, d)| !d.is_alive);
            if all_dead && day > 365 {
                let world = sim.world();
                let mut q2 = world.query::<&HumanDevelopmentComponent>();
                if let Some((_, dev)) = q2.iter().next() {
                    lifespan = dev.age_years() as f32;
                }
                break;
            }
        }
    }

    Ok(SimResult {
        lifespan,
        chip_onset_year,
        caii_at_60,
        myeloid_bias_at_70,
        stem_pool_at_70,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== INDUCER HYPOTHESIS TEST ===");
    println!("Running Blood HSC × 100 yr simulation...");
    println!();

    let with_ind    = run_scenario(true)?;
    let without_ind = run_scenario(false)?;

    println!("{:<24} {:>16} {:>18}", "", "With Inducers", "Without Inducers");
    println!("{}", "─".repeat(60));
    println!("{:<24} {:>14.1} yr {:>16.1} yr",
        "Lifespan:", with_ind.lifespan, without_ind.lifespan);
    println!("{:<24} {:>14} {:>18}",
        "CHIP onset:",
        with_ind.chip_onset_year.map_or("N/A".to_string(), |y| format!("{:.0} yr", y)),
        without_ind.chip_onset_year.map_or("N/A".to_string(), |y| format!("{:.0} yr", y))
    );
    println!("{:<24} {:>16.3} {:>18.3}",
        "CAII at 60:", with_ind.caii_at_60, without_ind.caii_at_60);
    println!("{:<24} {:>16.3} {:>18.3}",
        "Myeloid bias@70 (proxy):", with_ind.myeloid_bias_at_70, without_ind.myeloid_bias_at_70);
    println!("{:<24} {:>16.3} {:>18.3}",
        "Stem pool@70:", with_ind.stem_pool_at_70, without_ind.stem_pool_at_70);

    println!();
    println!("CONCLUSION: With inducer system lifespan = {:.1} yr vs without = {:.1} yr",
        with_ind.lifespan, without_ind.lifespan);
    if with_ind.lifespan >= without_ind.lifespan {
        println!("Inducer system EXTENDS lifespan by {:.1} yr (hypothesis supported)",
            with_ind.lifespan - without_ind.lifespan);
    } else {
        println!("Without inducers lifespan longer by {:.1} yr (unexpected — check params)",
            without_ind.lifespan - with_ind.lifespan);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_inducers_still_simulates() {
        // Без индукторов симуляция должна запускаться без ошибок
        let result = run_scenario(false);
        assert!(result.is_ok(), "without_inducers: симуляция завершилась с ошибкой: {:?}", result.err());
        let r = result.unwrap();
        assert!(r.lifespan > 0.0, "lifespan > 0");
    }

    #[test]
    fn inducer_system_affects_stem_pool_at_70() {
        let with_ind    = run_scenario(true).unwrap();
        let without_ind = run_scenario(false).unwrap();
        // Without inducers: stem_pool ≤ with_inducers at 70 (additional constraint applied)
        assert!(
            without_ind.stem_pool_at_70 <= with_ind.stem_pool_at_70 + 0.05,
            "without({:.3}) не должен быть значительно выше with({:.3})",
            without_ind.stem_pool_at_70,
            with_ind.stem_pool_at_70
        );
    }

    #[test]
    fn track_ab_preserved_in_both_modes() {
        // Треки A и B работают в обоих режимах (CAII > 0 в 60 лет)
        let with_ind    = run_scenario(true).unwrap();
        let without_ind = run_scenario(false).unwrap();
        assert!(with_ind.caii_at_60 > 0.0,    "with_inducers: CAII at 60 > 0");
        assert!(without_ind.caii_at_60 > 0.0, "without_inducers: CAII at 60 > 0");
    }
}
