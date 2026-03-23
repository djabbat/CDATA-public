//! P55 — PTM Trajectory Example
//!
//! Симулирует 3 типа ткани: Blood HSC, Neural NSC, Germline.
//! Каждые 5 лет записывает PTMBurdenProfile.
//! Выводит таблицу возрастов достижения 50% порога для каждого PTM-поля.

use cell_dt_core::components::{
    CentriolarDamageState, PTMBurdenProfile,
};
use std::fs;
use std::io::Write;

/// Параметры скорости старения для разных тканей.
struct TissueAgeParams {
    name: &'static str,
    ros_rate: f32,        // темп накопления ROS/год
    carbonyl_rate: f32,   // темп карбонилирования/год
    acetyl_rate: f32,     // темп гиперацетилирования/год
    aggregate_rate: f32,  // темп агрегации/год
    phospho_rate: f32,    // темп фосфодисрегуляции/год
    appendage_loss_rate: f32, // темп потери придатков/год
}

fn simulate_tissue(p: &TissueAgeParams, max_age: f32, step: f32) -> Vec<PTMBurdenProfile> {
    let mut dam = CentriolarDamageState::pristine();
    let mut trajectory = Vec::new();

    let mut age = 0.0f32;
    while age <= max_age {
        // Снимаем профиль каждые `step` лет
        trajectory.push(PTMBurdenProfile::from_damage_state(age, &dam));

        // Накапливаем повреждения за step лет
        dam.protein_carbonylation = (dam.protein_carbonylation + p.carbonyl_rate * step).clamp(0.0, 1.0);
        dam.tubulin_hyperacetylation = (dam.tubulin_hyperacetylation + p.acetyl_rate * step).clamp(0.0, 1.0);
        dam.protein_aggregates = (dam.protein_aggregates + p.aggregate_rate * step).clamp(0.0, 1.0);
        dam.phosphorylation_dysregulation = (dam.phosphorylation_dysregulation + p.phospho_rate * step).clamp(0.0, 1.0);
        // Потеря придатков: все четыре компонента снижаются
        let loss = p.appendage_loss_rate * step;
        dam.cep164_integrity = (dam.cep164_integrity - loss * 1.2).clamp(0.0, 1.0);
        dam.cep89_integrity  = (dam.cep89_integrity  - loss * 1.0).clamp(0.0, 1.0);
        dam.ninein_integrity = (dam.ninein_integrity - loss * 0.8).clamp(0.0, 1.0);
        dam.cep170_integrity = (dam.cep170_integrity - loss * 0.7).clamp(0.0, 1.0);
        // ROS нарастает (обратная связь)
        dam.ros_level = (dam.ros_level + p.ros_rate * step).clamp(0.0, 1.0);

        age += step;
    }
    trajectory
}

fn fmt_age(a: Option<f32>) -> String {
    match a {
        Some(v) => format!("{:.1} yr", v),
        None    => ">100 yr".to_string(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tissues = vec![
        TissueAgeParams {
            name: "Blood HSC",
            ros_rate: 0.003,
            carbonyl_rate: 0.0055,
            acetyl_rate:   0.0045,
            aggregate_rate: 0.0065,
            phospho_rate:  0.0042,
            appendage_loss_rate: 0.0048,
        },
        TissueAgeParams {
            name: "Neural NSC",
            ros_rate: 0.0045,
            carbonyl_rate: 0.0065,
            acetyl_rate:   0.0055,
            aggregate_rate: 0.0080,
            phospho_rate:  0.0050,
            appendage_loss_rate: 0.0060,
        },
        TissueAgeParams {
            name: "Germline",
            ros_rate: 0.002,
            carbonyl_rate: 0.0042,
            acetyl_rate:   0.0035,
            aggregate_rate: 0.0048,
            phospho_rate:  0.0032,
            appendage_loss_rate: 0.0038,
        },
    ];

    let max_age = 120.0f32;
    let step    = 5.0f32;

    let mut trajectories: Vec<Vec<PTMBurdenProfile>> = Vec::new();
    for t in &tissues {
        trajectories.push(simulate_tissue(t, max_age, step));
    }

    let fields = ["carbonylation", "hyperacetylation", "aggregation", "phospho_dysreg", "appendage_loss"];
    let labels = ["Carbonyl:", "Hyperacet:", "Aggregat:", "Phospho:", "Appendage:"];

    println!("=== PTM TRAJECTORY — AGE 50% DAMAGE THRESHOLD ===");
    println!();
    println!("{:<12} {:>14} {:>14} {:>14}", "", tissues[0].name, tissues[1].name, tissues[2].name);
    println!("{}", "─".repeat(58));

    for (label, field) in labels.iter().zip(fields.iter()) {
        let ages: Vec<Option<f32>> = trajectories.iter()
            .map(|traj| PTMBurdenProfile::age_at_50_percent(traj, field))
            .collect();
        println!("{:<12} {:>14} {:>14} {:>14}",
            label,
            fmt_age(ages[0]),
            fmt_age(ages[1]),
            fmt_age(ages[2])
        );
    }

    // Найти ткань, которая теряет функцию первой (по aggregation)
    let agg_ages: Vec<Option<f32>> = trajectories.iter()
        .map(|traj| PTMBurdenProfile::age_at_50_percent(traj, "aggregation"))
        .collect();

    let earliest = agg_ages.iter().enumerate()
        .filter_map(|(i, a)| a.map(|v| (i, v)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    println!();
    if let Some((idx, age)) = earliest {
        println!("PREDICTION: {} loses function first (aggregation threshold at {:.1} yr)",
            tissues[idx].name, age);
    }

    // --- Сохранить CSV ---
    let out_dir = "viz_output";
    fs::create_dir_all(out_dir)?;
    let path = format!("{}/ptm_trajectory.csv", out_dir);
    let mut f = fs::File::create(&path)?;
    writeln!(f, "tissue,age,carbonylation,hyperacetylation,aggregation,phospho_dysreg,appendage_loss,total")?;
    for (t, traj) in tissues.iter().zip(trajectories.iter()) {
        for p in traj {
            writeln!(f, "{},{:.1},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
                t.name, p.age_years, p.carbonylation, p.hyperacetylation,
                p.aggregation, p.phospho_dysreg, p.appendage_loss, p.total_burden)?;
        }
    }
    println!();
    println!("CSV saved: {}", path);

    Ok(())
}
