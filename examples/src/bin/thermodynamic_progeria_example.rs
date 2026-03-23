//! P62 — Thermodynamic Aging: Normal vs Progeria
//!
//! Два сценария: нормальное старение и прогерия (DamageParams::progeria(), ×5).
//! Отслеживает: Ze-velocity (v_consensus), локальную температуру, энтропию.
//! Демонстрирует термодинамическую модель (ThermodynamicState, Аррениус, Ze Theory).
//!
//! CSV → sensitivity_output/thermodynamic_progeria.csv

use cell_dt_core::components::{
    CentriolarDamageState, ThermodynamicState, ZeHealthState,
};
use human_development_module::{
    DamageParams, accumulate_damage,
    ThermodynamicParams, update_thermodynamic_state,
    update_ze_health_state,
};
use std::fs;
use std::io::Write;

/// Симулировать один сценарий до смерти или max_age.
/// Возвращает: вектор (age_years, v_consensus, local_temp_celsius, entropy_production)
fn simulate_scenario(
    damage_params: &DamageParams,
    thermo_params: &ThermodynamicParams,
    max_age: f32,
    dt: f32,
) -> (Vec<(f32, f32, f32, f32)>, f32) {
    let mut damage = CentriolarDamageState::pristine();
    let mut thermo = ThermodynamicState::pristine();
    let mut ze = ZeHealthState::default();

    let mut trajectory: Vec<(f32, f32, f32, f32)> = Vec::new();
    let mut age = 0.0f32;
    let mut death_age = max_age;

    while age <= max_age {
        // Обновить повреждения
        accumulate_damage(&mut damage, damage_params, age, dt, 0.0);

        // Обновить термодинамику
        update_thermodynamic_state(&mut thermo, &damage, damage.ros_level * 0.3, thermo_params, dt);

        // Обновить Ze-Health (caii approximation = ciliary_function как прокси)
        update_ze_health_state(&mut ze, damage.ciliary_function, Some(thermo.ze_velocity_analog));

        trajectory.push((
            age,
            ze.v_consensus,
            thermo.local_temp_celsius,
            thermo.entropy_production,
        ));

        // Проверить смерть: сенесценция достигнута
        if damage.is_senescent {
            death_age = age;
            break;
        }

        age += dt;
    }

    (trajectory, death_age)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dt = 1.0f32; // 1 год на шаг
    let max_age = 120.0f32;

    let normal_params  = DamageParams::default();
    let progeria_params = DamageParams::progeria();
    let thermo_params  = ThermodynamicParams::with_arrhenius();

    println!("=== THERMODYNAMIC AGING: NORMAL vs PROGERIA ===");
    println!();
    println!("{:<6} {:>11} {:>13} {:>14} {:>16} {:>11} {:>11}",
        "Age", "Normal_v", "Progeria_v", "Normal_Temp", "Progeria_Temp",
        "Entropy_N", "Entropy_P");
    println!("{}", "─".repeat(90));

    let (normal_traj,   normal_death)   = simulate_scenario(&normal_params,   &thermo_params, max_age, dt);
    let (progeria_traj, progeria_death) = simulate_scenario(&progeria_params, &thermo_params, max_age, dt);

    let mut csv_rows: Vec<String> = Vec::new();
    csv_rows.push("age,normal_v,progeria_v,normal_temp,progeria_temp,entropy_normal,entropy_progeria".to_string());

    // Печать каждые 10 лет
    let max_idx = normal_traj.len().max(progeria_traj.len());
    for i in 0..max_idx {
        let age = i as f32 * dt;
        if i % 10 != 0 { continue; }

        let (nv, nt, ne) = normal_traj.get(i)
            .map(|r| (r.1, r.2, r.3))
            .unwrap_or((f32::NAN, f32::NAN, f32::NAN));
        let (pv, pt, pe) = progeria_traj.get(i)
            .map(|r| (r.1, r.2, r.3))
            .unwrap_or((f32::NAN, f32::NAN, f32::NAN));

        let nv_str = if nv.is_nan() { "n/a (died)".to_string() } else { format!("{:.3}", nv) };
        let pv_str = if pv.is_nan() { "n/a (died)".to_string() } else { format!("{:.3}", pv) };
        let nt_str = if nt.is_nan() { "n/a".to_string() } else { format!("{:.2}°C", nt) };
        let pt_str = if pt.is_nan() { "n/a".to_string() } else { format!("{:.2}°C", pt) };
        let ne_str = if ne.is_nan() { "n/a".to_string() } else { format!("{:.3}", ne) };
        let pe_str = if pe.is_nan() { "n/a".to_string() } else { format!("{:.3}", pe) };

        println!("{:<6} {:>11} {:>13} {:>14} {:>16} {:>11} {:>11}",
            age as u32, nv_str, pv_str, nt_str, pt_str, ne_str, pe_str);

        csv_rows.push(format!(
            "{:.1},{},{},{},{},{},{}",
            age,
            if nv.is_nan() { "".to_string() } else { format!("{:.4}", nv) },
            if pv.is_nan() { "".to_string() } else { format!("{:.4}", pv) },
            if nt.is_nan() { "".to_string() } else { format!("{:.4}", nt) },
            if pt.is_nan() { "".to_string() } else { format!("{:.4}", pt) },
            if ne.is_nan() { "".to_string() } else { format!("{:.6}", ne) },
            if pe.is_nan() { "".to_string() } else { format!("{:.6}", pe) },
        ));
    }

    // CSV: все строки
    for i in 0..max_idx {
        let age = i as f32 * dt;
        if i % 10 == 0 { continue; } // уже добавлены
        let (nv, nt, ne) = normal_traj.get(i).map(|r| (r.1, r.2, r.3)).unwrap_or((f32::NAN, f32::NAN, f32::NAN));
        let (pv, pt, pe) = progeria_traj.get(i).map(|r| (r.1, r.2, r.3)).unwrap_or((f32::NAN, f32::NAN, f32::NAN));
        csv_rows.push(format!(
            "{:.1},{},{},{},{},{},{}",
            age,
            if nv.is_nan() { "".to_string() } else { format!("{:.4}", nv) },
            if pv.is_nan() { "".to_string() } else { format!("{:.4}", pv) },
            if nt.is_nan() { "".to_string() } else { format!("{:.4}", nt) },
            if pt.is_nan() { "".to_string() } else { format!("{:.4}", pt) },
            if ne.is_nan() { "".to_string() } else { format!("{:.6}", ne) },
            if pe.is_nan() { "".to_string() } else { format!("{:.6}", pe) },
        ));
    }

    println!();
    println!("Death: Normal={:.1}yr  Progeria={:.1}yr", normal_death, progeria_death);

    // Ze Theory check
    println!();
    println!("Ze Theory check:");
    if let Some((_, v20, _, _)) = normal_traj.get(20) {
        let expected = ZeHealthState::V_OPTIMAL;
        let deviation = (v20 - expected).abs() / expected * 100.0;
        let ok = if deviation < 15.0 { "✓" } else { "✗ (deviation > 15%)" };
        println!("  v* at age 20 (Normal):  {:.3}  (expected ≈ {:.3}, deviation = {:.1}%)  {}",
            v20, expected, deviation, ok);
    }
    if let Some(drop_age) = progeria_traj.iter().find(|&&(_, v, _, _)| v < 0.35).map(|r| r.0) {
        println!("  Progeria × 5 damage:    v drops below 0.35 at age {:.0}  ✓", drop_age);
    } else {
        println!("  Progeria × 5 damage:    v did not drop below 0.35 before death");
    }

    // --- Сохранить CSV ---
    let out_dir = "sensitivity_output";
    fs::create_dir_all(out_dir)?;
    let path = format!("{}/thermodynamic_progeria.csv", out_dir);
    let mut f = fs::File::create(&path)?;
    // Сортируем строки по возрасту
    let mut data_rows: Vec<&String> = csv_rows[1..].iter().collect();
    data_rows.sort_by(|a, b| {
        let av: f32 = a.split(',').next().unwrap_or("0").parse().unwrap_or(0.0);
        let bv: f32 = b.split(',').next().unwrap_or("0").parse().unwrap_or(0.0);
        av.partial_cmp(&bv).unwrap()
    });
    writeln!(f, "{}", csv_rows[0])?;
    for row in data_rows {
        writeln!(f, "{}", row)?;
    }
    println!();
    println!("CSV saved: {}", path);

    Ok(())
}
