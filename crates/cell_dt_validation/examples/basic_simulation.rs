/// CDATA v3.0 — Basic Simulation (uses AgingEngine)
///
/// Drives all 6 subsystems through the AgingEngine integrator.
/// Round 7 fixes are embedded in AgingEngine::step().

use cell_dt_aging_engine::{AgingEngine, SimulationConfig, SimulationPreset, InterventionSet};

fn main() {
    println!("=== CDATA v3.0 — Basic Simulation (AgingEngine) ===\n");

    // --- Baseline: Normal HSC ---
    let mut engine = AgingEngine::new(SimulationConfig::default()).expect("param validation");
    let baseline = engine.run(1);

    println!("{:<8} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10}",
        "Age", "Damage", "StemPool", "ROS", "SASP", "MCAI", "Telomere", "EpiAge");
    println!("{}", "-".repeat(88));

    for snap in baseline.iter().filter(|s| s.age_years as usize % 10 == 0) {
        println!("{:<8.0} {:<10.4} {:<10.4} {:<10.4} {:<10.4} {:<10.4} {:<10.4} {:<10.1}",
            snap.age_years, snap.centriole_damage, snap.stem_cell_pool,
            snap.ros_level,  snap.sasp_level,  snap.mcai,
            snap.telomere_length, snap.epigenetic_age);
    }

    // --- Tissue comparison ---
    println!("\n=== Tissue Comparison at age 80 ===");
    for preset in [SimulationPreset::Normal, SimulationPreset::Isc,
                   SimulationPreset::Muscle, SimulationPreset::Neural] {
        let mut e = AgingEngine::new(SimulationConfig {
            preset: preset.clone(), ..Default::default()
        }).unwrap();
        let hist = e.run(1);
        let snap80 = hist.iter().find(|s| (s.age_years - 80.0).abs() < 0.5).unwrap();
        println!("  {:<18} damage={:.4}  frailty={:.4}  telomere={:.4}",
            preset.label(), snap80.centriole_damage,
            snap80.mcai, snap80.telomere_length);
    }

    // --- Intervention comparison at age 80 ---
    println!("\n=== Intervention Comparison at age 80 ===");
    let all_ivs = InterventionSet {
        caloric_restriction: true, senolytics: true, antioxidants: true,
        mtor_inhibition: true,     telomerase: true,  htert: false, nk_boost: true,
        stem_cell_therapy: true,   epigenetic_reprogramming: true,
        strength: 1.0,
    };
    let mut ivs_engine = AgingEngine::new(SimulationConfig {
        interventions: all_ivs, ..Default::default()
    }).unwrap();
    let ivs_hist = ivs_engine.run(1);
    let baseline_80 = baseline.iter().find(|s| (s.age_years - 80.0).abs() < 0.5).unwrap();
    let ivs_80      = ivs_hist.iter().find(|s| (s.age_years - 80.0).abs() < 0.5).unwrap();
    println!("  Baseline   damage={:.4}  frailty={:.4}  telomere={:.4}  ros={:.4}",
        baseline_80.centriole_damage, baseline_80.mcai,
        baseline_80.telomere_length,  baseline_80.ros_level);
    println!("  All ivs    damage={:.4}  frailty={:.4}  telomere={:.4}  ros={:.4}",
        ivs_80.centriole_damage, ivs_80.mcai,
        ivs_80.telomere_length,  ivs_80.ros_level);

    // --- Progeria comparison ---
    println!("\n=== Preset Comparison at age 50 ===");
    for preset in [SimulationPreset::Progeria, SimulationPreset::Normal,
                   SimulationPreset::Longevity] {
        let mut e = AgingEngine::new(SimulationConfig {
            preset: preset.clone(), ..Default::default()
        }).unwrap();
        let hist = e.run(1);
        let snap50 = hist.iter().find(|s| (s.age_years - 50.0).abs() < 0.5).unwrap();
        println!("  {:<18} damage={:.4}  frailty={:.4}",
            preset.label(), snap50.centriole_damage, snap50.mcai);
    }

    println!("\n=== Round 7 fixes (all in AgingEngine::step) ===");
    println!("  B1: senescent_fraction .max(0.0)");
    println!("  B2: NF-κB clamp 0.95");
    println!("  B3: CHIP VAF at 70yr = 0.07 (Jaiswal 2017)");
    println!("  B4: NK decay = 0.010 (PMID 12803352)");
    println!("  B5: SASP continuity tests in fixed_params.rs");
    println!("  C1: mito_shield exp(-k*age), k=0.0099");
    println!("  C4: damps_decay_rate named param in InflammagingParams");
    println!("  M1/M2/M3: telomere, epigenetic clock, circadian placeholder");
    println!("  L1/L2/L3: CHIP→SASP, quiescence, fibrosis→regen");
}
