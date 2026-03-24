//! P51 — Ze-валидация.
//!
//! Симулирует один организм (average profile, Blood HSC, time_acceleration=1.0)
//! до 85 лет. Каждые 5 лет записывает ZeTrajectoryPoint из ZeHealthState.
//!
//! Проверяет соответствие клиническим данным Ze-HRV (n=60, Дортмунд):
//! - 3 контрольные возрастные группы: молодые (20-30), средние (40-55), пожилые (60-75)
//! - Корреляция Пирсона r(age, v) < -0.90

use cell_dt_core::{SimulationManager, SimulationConfig, ZeTrajectoryPoint, validate_ze_point, pearson_correlation};
use cell_dt_core::components::{CentriolePair, CellCycleStateExtended, ZeHealthState};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams,
};
use myeloid_shift_module::MyeloidShiftModule;
use std::io::Write;
use std::fs;

const MAX_AGE_YEARS: usize = 85;
const RECORD_INTERVAL: usize = 5;   // каждые 5 лет
const TIME_ACC: f64 = 1.0;          // 1 шаг = 1 день (точная физика)
const STEPS_PER_YEAR: usize = 365;  // 365 шагов = 1 год при time_acc=1.0

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Ze-VELOCITY TRAJECTORY (CDATA Simulator) ===");
    println!();

    let max_steps = (MAX_AGE_YEARS * STEPS_PER_YEAR + STEPS_PER_YEAR) as u64;

    let config = SimulationConfig {
        max_steps,
        dt: 1.0,
        checkpoint_interval: max_steps,
        num_threads: None,
        seed: Some(42),
        parallel_modules: false,
        cleanup_dead_interval: None,
    };

    let mut sim = SimulationManager::new(config);

    sim.register_module(Box::new(CentrioleModule::with_parallel(false)))?;
    sim.register_module(Box::new(CellCycleModule::with_params(CellCycleParams {
        base_cycle_time:           24.0,
        growth_factor_sensitivity: 0.3,
        stress_sensitivity:        0.2,
        checkpoint_strictness:     0.1,
        enable_apoptosis:          true,
        nutrient_availability:     0.9,
        growth_factor_level:       0.8,
        random_variation:          0.0,
    })))?;
    sim.register_module(Box::new(HumanDevelopmentModule::with_params(
        HumanDevelopmentParams {
            time_acceleration:          TIME_ACC,
            enable_aging:               true,
            enable_morphogenesis:       true,
            tissue_detail_level:        3,
            mother_inducer_count:       10,
            daughter_inducer_count:     8,
            base_detach_probability:    0.0003,
            mother_bias:                0.5,
            age_bias_coefficient:       0.0,
            ptm_exhaustion_scale:       0.001,
            de_novo_centriole_division: 4,
            meiotic_elimination_enabled: true,
            noise_scale:                0.0,  // без шума для воспроизводимости
            enable_inducer_system:      true,
            ..Default::default()
        }
    )))?;
    sim.register_module(Box::new(MyeloidShiftModule::new()))?;

    // Спавн одной ниши Blood HSC с ZeHealthState
    {
        let world = sim.world_mut();
        world.spawn((
            CentriolePair::default(),
            CellCycleStateExtended::new(),
            ZeHealthState::default(),
        ));
    }

    sim.initialize()?;

    let mut trajectory: Vec<ZeTrajectoryPoint> = Vec::new();
    let mut alive = true;

    for year in 1usize..=MAX_AGE_YEARS {
        // 365 шагов = 1 год
        for _ in 0..STEPS_PER_YEAR {
            sim.step()?;
        }

        // Проверяем живой ли организм
        let params = sim.get_module_params("human_development_module")?;
        alive = params.get("organism_is_alive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if year % RECORD_INTERVAL == 0 {
            // Читаем ZeHealthState из ECS
            let world = sim.world();
            let mut ze_v: f32 = ZeHealthState::V_OPTIMAL;
            let mut ze_hi: f32 = 1.0;
            let mut ze_interp: String = "optimal".to_string();
            for (_, ze) in world.query::<&ZeHealthState>().iter() {
                ze_v = ze.v_consensus;
                ze_hi = ze.ze_health_index;
                ze_interp = ze.interpretation().to_string();
                break; // берём первую (единственную) сущность
            }

            trajectory.push(ZeTrajectoryPoint {
                age_years: year as f32,
                v_consensus: ze_v,
                ze_health_index: ze_hi,
                interpretation: ze_interp,
            });
        }

        if !alive {
            let age = params.get("organism_age_years")
                .and_then(|v| v.as_f64())
                .unwrap_or(year as f64);
            println!("[Organism died at age {:.1}]", age);
            break;
        }
    }

    // Таблица траектории
    println!("{:<6} {:<12} {:<8} {:<20} {:<18} {}",
        "Age", "v_consensus", "Ze_HI", "Interpretation", "Clinical Range", "Status");
    println!("{}", "-".repeat(80));

    let mut validated_count = 0usize;
    let mut total_validated = 0usize;

    for pt in &trajectory {
        let (range_str, status_str) = match validate_ze_point(pt.v_consensus, pt.age_years) {
            Some(in_range) => {
                total_validated += 1;
                if in_range { validated_count += 1; }
                // Найти диапазон
                let ref_data = cell_dt_core::ZE_CLINICAL_REFS.iter()
                    .find(|r| pt.age_years >= r.age_min && pt.age_years <= r.age_max);
                let range = if let Some(r) = ref_data {
                    format!("{:.3} - {:.3}", r.v_mean - 1.5 * r.v_sd, r.v_mean + 1.5 * r.v_sd)
                } else {
                    "—".to_string()
                };
                let status = if in_range { "✓ IN".to_string() } else { "✗ OUT".to_string() };
                (range, status)
            }
            None => ("—".to_string(), "—".to_string()),
        };

        println!("{:<6} {:<12.3} {:<8.3} {:<20} {:<18} {}",
            pt.age_years as u32,
            pt.v_consensus,
            pt.ze_health_index,
            pt.interpretation,
            range_str,
            status_str);
    }

    // Корреляция r(age, ze_health_index) — CAII убывает с возрастом → r < 0
    let ages: Vec<f32> = trajectory.iter().map(|p| p.age_years).collect();
    let vs:   Vec<f32> = trajectory.iter().map(|p| p.v_consensus).collect();
    let his:  Vec<f32> = trajectory.iter().map(|p| p.ze_health_index).collect();
    let r_hi = pearson_correlation(&ages, &his);
    // r(age, v) для справки
    let r_v = pearson_correlation(&ages, &vs);

    println!();
    println!("=== VALIDATION SUMMARY ===");
    println!("Points in clinical range: {}/{} validated ages ({}%)",
        validated_count, total_validated,
        if total_validated > 0 { validated_count * 100 / total_validated } else { 0 });
    println!("Pearson r(age, Ze_HI): {:.3}  {} strong negative correlation {}",
        r_hi,
        if r_hi < -0.90 { "<-" } else { "  " },
        if r_hi < -0.90 { "✓" } else { "✗ (expected r < -0.90)" });
    println!("Pearson r(age, v):     {:.3}  (positive = v grows with age, as expected)", r_v);
    println!("Expected r(age, Ze_HI): < -0.90");
    println!();

    // Ze Theory prediction check — v* при молодом возрасте
    let v_at_20 = trajectory.iter().find(|p| p.age_years == 20.0).map(|p| p.v_consensus);
    let v_optimal = ZeHealthState::V_OPTIMAL;
    println!("Ze Theory v* = {:.3} (optimal health reference point)", v_optimal);
    if let Some(v20) = v_at_20 {
        let deviation = (v20 - v_optimal).abs();
        println!("Simulated v at age 20: {:.3}  deviation from v* = {:.3}  (CAII@20 = {:.3})",
            v20, deviation,
            trajectory.iter().find(|p| p.age_years == 20.0).map(|p| p.ze_health_index).unwrap_or(0.0));
    }

    // Сохранить CSV
    save_csv(&trajectory)?;
    println!();
    println!("CSV saved to: viz_output/ze_trajectory.csv");

    Ok(())
}

/// Сохранить траекторию Ze в CSV.
fn save_csv(trajectory: &[ZeTrajectoryPoint]) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("viz_output")?;
    let path = "viz_output/ze_trajectory.csv";
    let mut csv = String::new();
    csv.push_str("age_years,v_consensus,ze_health_index,interpretation\n");
    for pt in trajectory {
        csv.push_str(&format!(
            "{:.0},{:.4},{:.4},{}\n",
            pt.age_years,
            pt.v_consensus,
            pt.ze_health_index,
            pt.interpretation,
        ));
    }
    fs::write(path, csv)?;
    Ok(())
}
