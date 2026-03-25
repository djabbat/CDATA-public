//! CDATA Patient Simulation — CLI binary for AIM integration.
//!
//! Reads patient parameters from stdin as JSON, runs CDATA simulation,
//! outputs results as JSON to stdout.
//!
//! ## Usage (from AIM)
//! ```bash
//! echo '{"age":45,"tissue":"Blood","damage_scale":1.0,"steps":36500}' | ./cdata_patient_sim
//! ```
//!
//! ## Input JSON
//! ```json
//! {
//!   "age": 45,               // patient age (informational; future: warm-start)
//!   "tissue": "Blood",       // "Blood" | "Neural" | "Muscle" | "Skin" | "Liver"
//!   "damage_scale": 1.0,     // 1.0=normal, >1.0=accelerated aging, <1.0=longevity
//!   "steps": 36500,          // simulation steps (days), default = 36500 (100 years)
//!   "seed": 42               // optional RNG seed
//! }
//! ```
//!
//! ## Output JSON
//! ```json
//! {
//!   "lifespan_estimate": 78.4,
//!   "healthspan_estimate": 61.2,
//!   "death_cause": "frailty",
//!   "damage_at_60": 0.42,
//!   "damage_at_70": 0.59,
//!   "damage_at_80": 0.78,
//!   "myeloid_bias_at_70": 0.45,
//!   "spindle_fidelity_at_70": 0.61,
//!   "ciliary_function_at_70": 0.58,
//!   "stem_pool_at_70": 0.72,
//!   "methylation_age_at_70": 74.2,
//!   "interventions_recommended": ["NAD+", "Antioxidant"],
//!   "ok": true,
//!   "error": null
//! }
//! ```

use cell_dt_core::{SimulationManager, SimulationConfig};
use cell_dt_core::components::{CentriolePair, CellCycleStateExtended};
use cell_dt_core::EpigeneticClockState;
use centriole_module::CentrioleModule;
use cell_cycle_module::CellCycleModule;
use mitochondrial_module::MitochondrialModule;
use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams,
    HumanDevelopmentComponent, HumanTissueType,
    DamageParams,
};
use myeloid_shift_module::{MyeloidShiftModule, MyeloidShiftComponent};
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

// ── Input / Output structs ────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct PatientInput {
    #[serde(default = "default_tissue")]   tissue: String,
    #[serde(default = "default_damage_scale")] damage_scale: f32,
    #[serde(default = "default_steps")]   steps: u64,
    seed: Option<u64>,
    // age is informational only in this version
    #[allow(dead_code)]
    #[serde(default)]
    age: f64,
}

fn default_tissue() -> String { "Blood".to_string() }
fn default_damage_scale() -> f32 { 1.0 }
fn default_steps() -> u64 { 36_500 }

#[derive(Serialize, Debug, Default)]
struct SimOutput {
    lifespan_estimate:          f64,
    healthspan_estimate:        f64,
    death_cause:                String,
    damage_at_60:               f32,
    damage_at_70:               f32,
    damage_at_80:               f32,
    myeloid_bias_at_70:         f32,
    spindle_fidelity_at_70:     f32,
    ciliary_function_at_70:     f32,
    stem_pool_at_70:            f32,
    methylation_age_at_70:      f32,
    interventions_recommended:  Vec<String>,
    ok:                         bool,
    error:                      Option<String>,
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

#[derive(Default)]
struct Snapshot {
    damage:           f32,
    myeloid_bias:     f32,
    spindle_fidelity: f32,
    ciliary_function: f32,
    stem_pool:        f32,
    methylation_age:  f32,
}

fn capture(sim: &SimulationManager) -> Snapshot {
    let world = sim.world();
    let mut s = Snapshot::default();

    if let Some((_, comp)) = world.query::<&HumanDevelopmentComponent>().iter().next() {
        s.damage           = comp.centriolar_damage.total_damage_score();
        s.spindle_fidelity = comp.centriolar_damage.spindle_fidelity;
        s.ciliary_function = comp.centriolar_damage.ciliary_function;
        s.stem_pool        = comp.tissue_state.stem_cell_pool;
    }
    if let Some((_, m)) = world.query::<&MyeloidShiftComponent>().iter().next() {
        s.myeloid_bias = m.myeloid_bias;
    }
    if let Some((_, e)) = world.query::<&EpigeneticClockState>().iter().next() {
        s.methylation_age = e.methylation_age;
    }
    s
}

// ── Interventions recommender ─────────────────────────────────────────────────

fn recommend(s70: &Snapshot, s_final: &Snapshot) -> Vec<String> {
    let mut r = vec![];
    if s70.damage        > 0.55 { r.push("NAD+".into()); }
    if s70.myeloid_bias  > 0.40 { r.push("Senolytics".into()); }
    if s70.spindle_fidelity < 0.65 { r.push("Antioxidant".into()); }
    if s70.ciliary_function < 0.60 { r.push("CafdRetainer".into()); }
    if s70.methylation_age  > 75.0 { r.push("CaloricRestriction".into()); }
    if s_final.damage    > 0.88 { r.push("Tert".into()); }
    r
}

// ── Tissue mapping ────────────────────────────────────────────────────────────

fn tissue(s: &str) -> HumanTissueType {
    match s {
        "Blood"   => HumanTissueType::Blood,
        "Neural"  => HumanTissueType::Neural,
        "Muscle"  => HumanTissueType::Muscle,
        "Skin"    => HumanTissueType::Skin,
        "Liver"   => HumanTissueType::Liver,
        "Kidney"  => HumanTissueType::Kidney,
        "Lung"    => HumanTissueType::Lung,
        "Heart"   => HumanTissueType::Heart,
        _         => HumanTissueType::Blood,
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).ok();

    let patient: PatientInput = match serde_json::from_str(&raw) {
        Ok(p)  => p,
        Err(e) => {
            let out = SimOutput { error: Some(format!("Bad JSON: {e}")), ..Default::default() };
            println!("{}", serde_json::to_string(&out).unwrap());
            return;
        }
    };

    match run(patient) {
        Ok(out) => println!("{}", serde_json::to_string(&out).unwrap()),
        Err(e)  => {
            let out = SimOutput { error: Some(format!("Sim error: {e}")), ..Default::default() };
            println!("{}", serde_json::to_string(&out).unwrap());
        }
    }
}

fn run(p: PatientInput) -> Result<SimOutput, Box<dyn std::error::Error>> {
    let config = SimulationConfig {
        max_steps: p.steps,
        dt: 1.0,
        checkpoint_interval: 3650,
        num_threads: Some(2),
        seed: p.seed.or(Some(42)),
        parallel_modules: false,
        cleanup_dead_interval: Some(500),
    };

    let mut sim = SimulationManager::new(config);

    // Modules in required order
    sim.register_module(Box::new(CentrioleModule::new()))?;
    sim.register_module(Box::new(CellCycleModule::new()))?;
    sim.register_module(Box::new(MitochondrialModule::new()))?;

    let mut hdm = HumanDevelopmentModule::with_params(HumanDevelopmentParams {
        base_detach_probability: HumanDevelopmentParams::default().base_detach_probability
            * p.damage_scale,
        ptm_exhaustion_scale: HumanDevelopmentParams::default().ptm_exhaustion_scale
            * p.damage_scale,
        ..HumanDevelopmentParams::default()
    });
    // Scale all molecular damage rates via DamageParams::scaled()
    hdm.set_damage_rates(DamageParams::scaled(p.damage_scale));
    sim.register_module(Box::new(hdm))?;
    sim.register_module(Box::new(MyeloidShiftModule::new()))?;
    // AsymmetricDivisionModule is for CHIP-drift population studies only:
    // it activates D-IDI detachment per M-phase division which depletes daughter
    // inducers in ~42 years instead of the calibrated 78-year lifespan.
    // StemCellHierarchyModule: not needed for patient-level output metrics.

    // Spawn 5 niche entities — one per tissue type.
    // Use the same minimal spawn pattern as human_development_example (calibrated to 78.4 yr):
    // only CentriolePair + CellCycleStateExtended. All modules add their own components
    // during initialize(): HumanDevelopmentModule assigns tissue types via tissue_cycle,
    // MyeloidShiftModule adds MyeloidShiftComponent, AsymmetricDivisionModule adds
    // AsymmetricDivisionComponent + DivisionExhaustionState, etc.
    {
        let world = sim.world_mut();
        for _ in 0..5 {
            world.spawn((
                CentriolePair::default(),
                CellCycleStateExtended::new(),
            ));
        }
    }
    let _ = tissue(&p.tissue); // tissue mapping available for future warm-start use

    sim.initialize()?;

    let day_60 = (60.0 * 365.25) as u64;
    let day_70 = (70.0 * 365.25) as u64;
    let day_80 = (80.0 * 365.25) as u64;

    let mut snap60 = Snapshot::default();
    let mut snap70 = Snapshot::default();
    let mut snap80 = Snapshot::default();
    let mut lifespan: f64 = 0.0;
    let mut healthspan_days: u64 = 0;

    loop {
        let step = sim.current_step();
        if step == day_60 { snap60 = capture(&sim); }
        if step == day_70 { snap70 = capture(&sim); }
        if step == day_80 { snap80 = capture(&sim); }


        // Use organism_is_alive from module params (aggregates all 5 niches)
        let dead = {
            let params = sim.get_module_params("human_development_module").ok();
            let org_alive = params.as_ref()
                .and_then(|v| v["organism_is_alive"].as_bool())
                .unwrap_or(true);
            let org_age = params.as_ref()
                .and_then(|v| v["organism_age_years"].as_f64())
                .unwrap_or(0.0);

            if org_alive {
                lifespan = org_age;
                // healthspan: track while damage < 0.5 on any live niche
                let world = sim.world();
                if let Some((_, comp)) = world.query::<&HumanDevelopmentComponent>().iter()
                    .find(|(_, c)| c.is_alive) {
                    if comp.centriolar_damage.total_damage_score() < 0.5 {
                        healthspan_days += 1;
                    }
                }
                false
            } else {
                lifespan = org_age;
                true
            }
        };

        if dead || step >= p.steps { break; }
        sim.step()?;
    }

    // Get death cause from module params
    let death_cause = sim
        .get_module_params("human_development_module")
        .ok()
        .and_then(|v| v["organism_death_cause"].as_str().map(String::from))
        .unwrap_or_else(|| "max_age".to_string());

    let snap_final = capture(&sim);
    let recs = recommend(&snap70, &snap_final);

    Ok(SimOutput {
        lifespan_estimate:         lifespan,
        healthspan_estimate:       healthspan_days as f64 / 365.25,
        death_cause,
        damage_at_60:              snap60.damage,
        damage_at_70:              snap70.damage,
        damage_at_80:              snap80.damage,
        myeloid_bias_at_70:        snap70.myeloid_bias,
        spindle_fidelity_at_70:    snap70.spindle_fidelity,
        ciliary_function_at_70:    snap70.ciliary_function,
        stem_pool_at_70:           snap70.stem_pool,
        methylation_age_at_70:     snap70.methylation_age,
        interventions_recommended: recs,
        ok:    true,
        error: None,
    })
}
