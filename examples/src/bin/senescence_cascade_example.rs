//! P53 — Senescence Cascade Example
//!
//! 100-летняя симуляция Blood HSC с двумя сценариями:
//! - **Control**: без сенолитиков
//! - **Senolytic**: senolytic_clearance=0.30 начиная с года 60
//!
//! Демонстрирует использование SenescenceAccumulationState + update_senescence_accumulation_state.

use cell_dt_core::components::{
    SenescenceAccumulationState, SenescenceAccumulationParams,
    update_senescence_accumulation_state,
};
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SENESCENCE CASCADE ===");
    println!();
    println!("{:<6} {:>7} {:>7} {:>14} {:>9}  {}",
             "Age", "Senes%", "SASP", "NicheCapacity", "DivRate", "Scenario");
    println!("{}", "─".repeat(70));

    // --- Параметры симуляции ---
    let dt = 1.0f32; // 1 год на шаг

    // Базовые CDATA-параметры: молодой Blood HSC
    let base_ros   = 0.05f32;
    let aged_ros   = 0.35f32;  // ROS растёт с возрастом
    let base_caii  = 0.95f32;
    let aged_caii  = 0.55f32;  // CAII падает с возрастом

    let mut csv_rows: Vec<String> = Vec::new();
    csv_rows.push("scenario,age,senescent_fraction,sasp_output,niche_regenerative_capacity,division_rate".to_string());

    // --- Запуск двух сценариев ---
    for scenario in &["Control", "Senolytic"] {
        let mut state = SenescenceAccumulationState {
            senescent_fraction: 0.01,
            ..Default::default()
        };
        let mut params = SenescenceAccumulationParams::default();

        for age in 1u32..=100 {
            // Интерполируем ROS и CAII по возрасту
            let progress = (age as f32 - 1.0) / 99.0;
            let ros_level = base_ros + (aged_ros - base_ros) * progress;
            let caii      = base_caii + (aged_caii - base_caii) * progress;

            // Senolytic сценарий: с года 60 включаем clearance каждые 5 лет
            if *scenario == "Senolytic" {
                if age >= 60 && (age - 60) % 5 == 0 {
                    params.senolytic_clearance = 0.30;
                } else {
                    params.senolytic_clearance = 0.0;
                }
            }

            update_senescence_accumulation_state(
                &mut state, &params, ros_level, 0.8, caii, dt,
            );

            // Приблизительная division rate: снижается с сенесценцией и возрастом
            let division_rate = (1.0 - state.senescent_fraction * 0.5)
                * (1.0 - progress * 0.40)
                * caii;

            // Вывод каждые 10 лет
            if age % 10 == 0 {
                println!("{:<6} {:>6.1}% {:>7.3} {:>14.3} {:>9.2}  {}",
                    age,
                    state.senescent_fraction * 100.0,
                    state.sasp_output,
                    state.niche_regenerative_capacity,
                    division_rate,
                    scenario
                );
                csv_rows.push(format!(
                    "{},{},{:.4},{:.4},{:.4},{:.4}",
                    scenario, age,
                    state.senescent_fraction,
                    state.sasp_output,
                    state.niche_regenerative_capacity,
                    division_rate
                ));
            }
        }
        println!();
    }

    // --- Сохранить CSV ---
    let out_dir = "cell_cycle_output";
    fs::create_dir_all(out_dir)?;
    let path = format!("{}/senescence_cascade.csv", out_dir);
    let mut f = fs::File::create(&path)?;
    for row in &csv_rows {
        writeln!(f, "{}", row)?;
    }
    println!("CSV saved: {}", path);

    Ok(())
}
