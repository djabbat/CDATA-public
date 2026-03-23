//! P60 — InterSpecies CDATA Validation
//!
//! Сравнивает известные продолжительности жизни с предсказаниями CDATA-модели
//! для мыши, человека, летучей мыши и голого землекопа.

use cell_dt_core::components::{SPECIES_PROFILES, predicted_lifespan_from_cdata};

fn main() {
    println!("=== INTERSPECIES CDATA VALIDATION ===");
    println!();
    println!("{:<18} {:>9} {:>11} {:>8}", "Species", "Known", "Predicted", "Error%");
    println!("{}", "─".repeat(52));

    for sp in SPECIES_PROFILES {
        let predicted = predicted_lifespan_from_cdata(sp);
        let error_pct = (predicted - sp.lifespan_years).abs() / sp.lifespan_years * 100.0;
        println!("{:<18} {:>7.1} yr {:>9.1} yr {:>7.1}%",
            sp.name, sp.lifespan_years, predicted, error_pct);
    }

    println!();
    println!("Model: predicted = 78 / sqrt(base_detach_scale × ros_scale)");
    println!("Theory: CDATA (Tkemaladze 2025) — detachment×ROS drives aging rate");

    // Validate key predictions
    println!();
    println!("=== KEY PREDICTIONS ===");

    let mouse = SPECIES_PROFILES.iter().find(|s| s.name == "mouse").unwrap();
    let human = SPECIES_PROFILES.iter().find(|s| s.name == "human").unwrap();
    let bat   = SPECIES_PROFILES.iter().find(|s| s.name == "bat").unwrap();
    let nmr   = SPECIES_PROFILES.iter().find(|s| s.name == "naked_mole_rat").unwrap();

    let pred_mouse = predicted_lifespan_from_cdata(mouse);
    let pred_human = predicted_lifespan_from_cdata(human);
    let pred_bat   = predicted_lifespan_from_cdata(bat);
    let pred_nmr   = predicted_lifespan_from_cdata(nmr);

    println!("Mouse shorter than human:          {} (pred {:.1} < {:.1})",
        pred_mouse < pred_human, pred_mouse, pred_human);
    println!("Bat longer than mouse (low ROS):   {} (bat {:.1} > mouse {:.1})",
        pred_bat > pred_mouse, pred_bat, pred_mouse);
    println!("NMR longevity (very low ROS):      {} (nmr {:.1} > mouse {:.1})",
        pred_nmr > pred_mouse, pred_nmr, pred_mouse);
}
