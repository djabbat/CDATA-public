use cell_dt_core::{FixedParameters, TissueState, MitochondrialState, InflammagingState};
use cell_dt_mitochondrial::MitochondrialSystem;
use cell_dt_inflammaging::InflammagingSystem;
use cell_dt_tissue_specific::{TissueSpecificParams, TissueType};

fn main() {
    println!("=== CDATA v3.0 — Basic Simulation ===\n");

    // Validate parameters before use
    let params = FixedParameters::default();
    params.validate().expect("FixedParameters validation failed");

    let mut tissue = TissueState::new(0.0);
    let mut mito = MitochondrialState::default();
    let mut inflamm = InflammagingState::default();

    let mito_sys = MitochondrialSystem::new();
    let inflamm_sys = InflammagingSystem::new();
    let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);

    println!("{:<8} {:<12} {:<12} {:<12} {:<12} {:<12}",
        "Age", "Damage", "StemPool", "ROS", "SASP", "Frailty");
    println!("{}", "-".repeat(72));

    let dt = 1.0_f64;
    for year in 0usize..=100 {
        let age = year as f64;
        tissue.age_years = age;

        // === Damage accumulation (CDATA main equation) ===
        // d(Damage)/dt = α × ν(t) × (1 - Π(t)) × β × (1 - tolerance) × ROS_factor
        //
        // FIXED (Round 6 peer review):
        // - Was: β / tolerance → tolerance=0.3 gave ×3.33 amplifier → saturation at age 20
        // - Now: β × (1 - tolerance) → tolerance as "protective fraction" [0,1]
        //   tolerance=0.8 (ISC): 80% divisions repaired → small net damage
        //   tolerance=0.3 (HSC): only 30% repaired → faster accumulation
        //
        // - Added ROS coupling: oxidative stress accelerates centriole PTM damage
        let protection = params.youth_protection(age);
        let age_factor = 1.0 - (age / 120.0_f64).min(0.5);
        let sasp_factor = params.sasp_hormetic_response(inflamm.sasp_level);
        let division_rate = hsc.effective_division_rate(age_factor, sasp_factor);

        // ROS amplifies centriole damage (oxidative PTM modifications)
        let ros_damage_factor = 1.0 + mito.ros_level * 0.5;

        let damage_rate = params.alpha
            * division_rate
            * (1.0 - protection)
            * hsc.damage_per_division_multiplier
            * (1.0 - hsc.tolerance)   // FIXED: was / tolerance
            * ros_damage_factor;       // ADDED: ROS coupling
        tissue.centriole_damage = (tissue.centriole_damage + damage_rate * dt).min(1.0);
        tissue.stem_cell_pool = (1.0 - tissue.centriole_damage * 0.8).max(0.0);

        // === Mitochondrial dynamics ===
        mito_sys.update(&mut mito, dt, age, inflamm.sasp_level);

        // === Inflammaging dynamics ===
        // FIXED (Round 6 peer review):
        // - Was: direct assignment inflamm.senescent_cell_fraction = centriole_damage * 0.3
        //   This overwrote NK clearance every year → NK had no cumulative effect (BLOCKER B13)
        // - Now: differential update — cells enter senescence from damage, exit via NK
        let new_senescent_from_damage = tissue.centriole_damage * 0.05 * dt;
        let current_sen = inflamm.senescent_cell_fraction;
        inflamm.senescent_cell_fraction = (current_sen + new_senescent_from_damage).min(1.0);

        inflamm_sys.update(
            &mut inflamm,
            dt,
            age,
            tissue.centriole_damage,
            mito.mtdna_mutations * 0.1,
        );

        // === Frailty index ===
        tissue.frailty_index = (tissue.centriole_damage * 0.5
            + inflamm.sasp_level * 0.3
            + (1.0 - tissue.stem_cell_pool) * 0.2).min(1.0);

        if year % 10 == 0 {
            println!("{:<8.0} {:<12.4} {:<12.4} {:<12.4} {:<12.4} {:<12.4}",
                age, tissue.centriole_damage, tissue.stem_cell_pool,
                mito.ros_level, inflamm.sasp_level, tissue.frailty_index);
        }
    }

    println!("\n=== Tissue Comparison (effective aging rate: ν × β × (1-tolerance)) ===");
    for tissue_type in [TissueType::Hematopoietic, TissueType::Intestinal,
                         TissueType::Muscle, TissueType::Neural] {
        let p = TissueSpecificParams::for_tissue(tissue_type);
        println!("  {:?}: {:.2}", p.tissue_type, p.effective_aging_rate());
    }

    println!("\n=== Parameter validation ===");
    match params.validate() {
        Ok(()) => println!("  All 32 parameters: OK"),
        Err(e) => println!("  VALIDATION ERROR: {}", e),
    }
}
