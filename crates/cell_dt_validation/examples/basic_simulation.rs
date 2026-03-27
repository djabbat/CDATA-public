use cell_dt_core::{FixedParameters, TissueState, MitochondrialState, InflammagingState};
use cell_dt_mitochondrial::MitochondrialSystem;
use cell_dt_inflammaging::InflammagingSystem;
use cell_dt_tissue_specific::{TissueSpecificParams, TissueType};
use cell_dt_asymmetric_division::ChipSystem;

fn main() {
    println!("=== CDATA v3.0 — Basic Simulation (Round 7: all fixes complete) ===\n");

    // Validate parameters before use
    let params = FixedParameters::default();
    params.validate().expect("FixedParameters validation failed");

    let mut tissue = TissueState::new(0.0);
    let mut mito = MitochondrialState::default();
    let mut inflamm = InflammagingState::default();

    let mito_sys = MitochondrialSystem::new();
    let inflamm_sys = InflammagingSystem::new();
    let hsc = TissueSpecificParams::for_tissue(TissueType::Hematopoietic);
    // L1: CHIP system — DNMT3A/TET2 clones amplify SASP (PMID: 29507339)
    let mut chip_sys = ChipSystem::new(42);

    // Telomere loss per division (kb): HSC lose ~30-50 bp/division (Lansdorp 2005)
    // Normalized: 1.0 = full young length, ~0.3 at age 80 in HSC
    const TELOMERE_LOSS_PER_DIVISION: f64 = 0.012; // ~1.2% per division

    // Epigenetic clock acceleration factor with damage
    const EPI_STRESS_COEFF: f64 = 0.15;

    println!("{:<8} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10}",
        "Age", "Damage", "StemPool", "ROS", "SASP", "Frailty", "Telomere", "EpiAge");
    println!("{}", "-".repeat(88));

    let dt = 1.0_f64;
    for year in 0usize..=100 {
        let age = year as f64;
        tissue.age_years = age;

        // === Damage accumulation (CDATA main equation) ===
        // d(Damage)/dt = α × ν(t) × (1 - Π(t)) × β × (1 - tolerance) × ROS_factor
        //
        // Round 7 additions:
        // L2: damage → quiescence: high centriole damage suppresses division rate
        //     Biological basis: damaged HSC arrested by p21/p53 pathway (PMID: 20357022)
        // L3: fibrosis → regenerative_potential reduction
        let protection = params.youth_protection(age);
        let age_factor = 1.0 - (age / 120.0_f64).min(0.5);
        let sasp_factor = params.sasp_hormetic_response(inflamm.sasp_level);

        // L2: quiescence suppression at high centriole damage
        let quiescence_factor = (1.0 - tissue.centriole_damage * 0.5).max(0.2);
        // L3: fibrosis reduces regenerative potential
        let regen_factor = (1.0 - inflamm.fibrosis_level * 0.4).max(0.3);

        let division_rate = hsc.base_division_rate
            * age_factor
            * sasp_factor
            * hsc.regenerative_potential
            * quiescence_factor  // L2: ADDED
            * regen_factor;      // L3: ADDED

        // ROS amplifies centriole damage (oxidative PTM modifications)
        let ros_damage_factor = 1.0 + mito.ros_level * 0.5;

        let damage_rate = params.alpha
            * division_rate
            * (1.0 - protection)
            * hsc.damage_per_division_multiplier
            * (1.0 - hsc.tolerance)
            * ros_damage_factor;
        tissue.centriole_damage = (tissue.centriole_damage + damage_rate * dt).min(1.0);
        tissue.stem_cell_pool = (1.0 - tissue.centriole_damage * 0.8).max(0.0);

        // === M1: Telomere shortening ===
        // Telomere length normalized to [0, 1]. Shortens with each division.
        // Calibrated: HSC ~30-50 bp/division; at 12 divisions/year, ~50% loss over 100 years
        let telomere_loss = TELOMERE_LOSS_PER_DIVISION * division_rate * dt;
        tissue.telomere_length = (tissue.telomere_length - telomere_loss).max(0.0);

        // === M2: Epigenetic clock acceleration ===
        // Base: epigenetic age drifts toward chronological age
        // Stress: centriole damage and SASP accelerate epigenetic aging
        let epi_base_drift = (age - tissue.epigenetic_age) * 0.1 * dt;
        let epi_stress = EPI_STRESS_COEFF * (tissue.centriole_damage + inflamm.sasp_level * 0.5) * dt;
        tissue.epigenetic_age = (tissue.epigenetic_age + epi_base_drift + epi_stress)
            .clamp(0.0, age + 30.0); // epigenetic age can run ahead of chronological

        // === Mitochondrial dynamics ===
        mito_sys.update(&mut mito, dt, age, inflamm.sasp_level);

        // === Inflammaging dynamics ===
        // Round 6 fix: differential update for senescence
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

        // === L1: CHIP → SASP amplification (Round 7 fix) ===
        // DNMT3A/TET2 mutant clones produce excess IL-6 and TNF-α (PMID: 29507339)
        chip_sys.update(division_rate, inflamm.sasp_level, age, dt);
        let sasp_chip_boost = (chip_sys.sasp_amplification() - 1.0) * 0.1 * dt;
        inflamm.sasp_level = (inflamm.sasp_level + sasp_chip_boost).min(1.0);

        // === M3: Circadian amplitude → repair efficiency modulation ===
        // circadian_amplitude=0.2 modulates effective repair over 24-hr cycle.
        // In annual simulation we use mean ± half-amplitude as aging penalty:
        // older organisms lose circadian coherence → lower mean repair.
        // Here approximated as: circadian_penalty += 0.001 * (1 - circadian_amplitude) * dt
        // (placeholder — full implementation needs sub-annual timestep)
        let _circadian_repair_factor = 1.0 - (1.0 - params.circadian_amplitude) * (age / 100.0) * 0.2;

        // === Frailty index ===
        // Round 7: added telomere component (short telomeres → frailty in elderly)
        tissue.frailty_index = (tissue.centriole_damage * 0.4
            + inflamm.sasp_level * 0.3
            + (1.0 - tissue.stem_cell_pool) * 0.2
            + (1.0 - tissue.telomere_length) * 0.1).min(1.0);

        if year % 10 == 0 {
            println!("{:<8.0} {:<10.4} {:<10.4} {:<10.4} {:<10.4} {:<10.4} {:<10.4} {:<10.1}",
                age, tissue.centriole_damage, tissue.stem_cell_pool,
                mito.ros_level, inflamm.sasp_level, tissue.frailty_index,
                tissue.telomere_length, tissue.epigenetic_age);
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

    println!("\n=== Round 7 Fix Summary ===");
    println!("  B2: NF-κB formula: removed *0.9, clamp adjusted to 0.95");
    println!("  B3: CHIP VAF recalibrated per Jaiswal 2017 (PMID: 28792876)");
    println!("  B4: NK decay 0.005→0.010 per PMID: 12803352");
    println!("  M1: Telomere shortening per division added");
    println!("  M2: Epigenetic clock drift + stress acceleration added");
    println!("  M3: circadian_amplitude placeholder added");
    println!("  L2: Quiescence suppression at high centriole damage");
    println!("  L3: Fibrosis reduces regenerative_potential");
    println!("  L1: CHIP→SASP amplification via ChipSystem (sasp_boost * 0.1/yr, dampened)");
}
