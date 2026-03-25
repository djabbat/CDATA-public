//! Sleep module — Ecosphere Level (+5) of CDATA hierarchy.
//!
//! ## Biological basis
//!
//! Sleep is the primary window for cellular repair and proteostasis restoration:
//!
//! - **Slow-wave sleep (SWS)** → glymphatic clearance → protein_aggregates ↓
//! - **Growth hormone (GH) pulse** during deep sleep → IGF-1 → stem cell
//!   proliferation ↑, tissue regeneration ↑
//! - **Circadian alignment** → CLOCK/BMAL1 → DNA repair ↑, telomerase ↑
//! - **Sleep restriction** → cortisol ↑ → SASP acceleration, ROS ↑,
//!   inflammaging ↑
//!
//! ## Communication with CDATA core
//!
//! Writes to:
//! - `CentriolarDamageState.protein_aggregates` (glymphatic clearance)
//! - `MitochondrialState.ros_production` (cortisol-driven ROS)
//! - `InflammagingState.sasp_intensity` (sleep deprivation → SASP)

use cell_dt_core::{
    SimulationModule, SimulationResult,
    hecs::World,
    components::{CentriolarDamageState, MitochondrialState, InflammagingState},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::trace;

// ---------------------------------------------------------------------------
// ECS Component
// ---------------------------------------------------------------------------

/// ECS component: sleep quality and circadian state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepState {
    /// Sleep quality index [0..1]: 1.0 = optimal restorative sleep, 0 = none.
    pub sleep_quality: f32,

    /// Circadian alignment [0..1].
    pub circadian_alignment: f32,

    /// Growth hormone pulse strength [0..1] — derived each step.
    pub gh_pulse: f32,

    /// Glymphatic clearance rate [0..1].
    pub glymphatic_clearance: f32,

    /// Cortisol stress from poor sleep [0..1].
    pub cortisol_stress: f32,
}

impl Default for SleepState {
    fn default() -> Self {
        Self {
            sleep_quality:        0.75,
            circadian_alignment:  0.80,
            gh_pulse:             0.60,
            glymphatic_clearance: 0.65,
            cortisol_stress:      0.10,
        }
    }
}

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepParams {
    pub glymphatic_aggregate_scale: f32,
    pub cortisol_ros_scale:         f32,
    pub circadian_dna_repair_scale: f32,
    pub gh_regeneration_scale:      f32,
}

impl Default for SleepParams {
    fn default() -> Self {
        Self {
            glymphatic_aggregate_scale: 0.20,
            cortisol_ros_scale:         0.15,
            circadian_dna_repair_scale: 0.10,
            gh_regeneration_scale:      0.12,
        }
    }
}

// ---------------------------------------------------------------------------
// Core update function (pure, testable)
// ---------------------------------------------------------------------------

/// Returns (aggregate_reduction, ros_addition, sasp_boost).
pub fn update_sleep(
    state: &mut SleepState,
    params: &SleepParams,
    age_years: f32,
    dt: f32,
) -> (f32, f32, f32) {
    let age_gh_factor = (1.0 - age_years * 0.005).clamp(0.3, 1.0);
    state.gh_pulse = state.sleep_quality * state.circadian_alignment * age_gh_factor;
    state.glymphatic_clearance = state.sleep_quality * 0.9;
    state.cortisol_stress = (1.0 - state.sleep_quality) * 0.4;

    let aggregate_reduction = state.glymphatic_clearance
        * params.glymphatic_aggregate_scale * dt;
    let ros_addition = (state.cortisol_stress - 0.10).max(0.0)
        * params.cortisol_ros_scale * dt;
    // sasp_boost from chronic deprivation
    let sasp_boost = (state.cortisol_stress - 0.15).max(0.0) * 0.10 * dt;

    (aggregate_reduction, ros_addition, sasp_boost)
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

pub struct SleepModule {
    pub params: SleepParams,
    step_count: u64,
}

impl Default for SleepModule {
    fn default() -> Self { Self { params: SleepParams::default(), step_count: 0 } }
}

impl SleepModule {
    pub fn new() -> Self { Self::default() }
    pub fn sleep_deprived() -> Self {
        Self { params: SleepParams { cortisol_ros_scale: 0.25, ..Default::default() }, step_count: 0 }
    }
    pub fn optimal() -> Self {
        Self { params: SleepParams { gh_regeneration_scale: 0.20, ..Default::default() }, step_count: 0 }
    }
}

impl SimulationModule for SleepModule {
    fn name(&self) -> &str { "sleep_module" }

    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        let entities: Vec<_> = world
            .query::<&CentriolarDamageState>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for e in entities {
            if world.get::<&SleepState>(e).is_err() {
                let _ = world.insert_one(e, SleepState::default());
            }
        }
        Ok(())
    }

    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        let dt_years = (dt / 365.25) as f32;
        // Approximate age from step count (rough; human_dev module has exact age)
        let approx_age = (self.step_count as f32 * dt_years).clamp(0.0, 120.0);

        let entities: Vec<_> = world
            .query::<&SleepState>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        for e in entities {
            let (agg_red, ros_add, sasp_boost) = {
                let Ok(mut sl) = world.get::<&mut SleepState>(e) else { continue };
                update_sleep(&mut sl, &self.params, approx_age, dt_years)
            };

            if let Ok(mut cda) = world.get::<&mut CentriolarDamageState>(e) {
                cda.protein_aggregates = (cda.protein_aggregates - agg_red).clamp(0.0, 1.0);
            }

            if let Ok(mut mito) = world.get::<&mut MitochondrialState>(e) {
                mito.ros_production = (mito.ros_production + ros_add).clamp(0.0, 1.0);
            }

            if sasp_boost > 0.0 {
                if let Ok(mut inf) = world.get::<&mut InflammagingState>(e) {
                    inf.sasp_intensity = (inf.sasp_intensity + sasp_boost).clamp(0.0, 1.0);
                }
            }

            trace!(
                "Sleep step {}: agg_red={:.4} ros_add={:.4} sasp_boost={:.4}",
                self.step_count, agg_red, ros_add, sasp_boost
            );
        }
        Ok(())
    }

    fn get_params(&self) -> Value {
        json!({
            "glymphatic_aggregate_scale": self.params.glymphatic_aggregate_scale,
            "cortisol_ros_scale":         self.params.cortisol_ros_scale,
            "circadian_dna_repair_scale": self.params.circadian_dna_repair_scale,
            "gh_regeneration_scale":      self.params.gh_regeneration_scale,
        })
    }

    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(v) = params.get("glymphatic_aggregate_scale").and_then(|v| v.as_f64()) {
            self.params.glymphatic_aggregate_scale = v as f32;
        }
        if let Some(v) = params.get("cortisol_ros_scale").and_then(|v| v.as_f64()) {
            self.params.cortisol_ros_scale = v as f32;
        }
        if let Some(v) = params.get("circadian_dna_repair_scale").and_then(|v| v.as_f64()) {
            self.params.circadian_dna_repair_scale = v as f32;
        }
        if let Some(v) = params.get("gh_regeneration_scale").and_then(|v| v.as_f64()) {
            self.params.gh_regeneration_scale = v as f32;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_sleep_clears_aggregates() {
        let mut state = SleepState { sleep_quality: 1.0, circadian_alignment: 1.0, ..Default::default() };
        let (agg_red, _, _) = update_sleep(&mut state, &SleepParams::default(), 30.0, 1.0);
        assert!(agg_red > 0.1);
    }

    #[test]
    fn poor_sleep_raises_ros() {
        let mut state = SleepState { sleep_quality: 0.2, ..Default::default() };
        let (_, ros_add, _) = update_sleep(&mut state, &SleepParams::default(), 30.0, 1.0);
        assert!(ros_add > 0.0);
    }

    #[test]
    fn gh_pulse_declines_with_age() {
        let mut young = SleepState { sleep_quality: 0.9, circadian_alignment: 0.9, ..Default::default() };
        let mut old   = SleepState { sleep_quality: 0.9, circadian_alignment: 0.9, ..Default::default() };
        update_sleep(&mut young, &SleepParams::default(), 25.0, 1.0);
        update_sleep(&mut old,   &SleepParams::default(), 75.0, 1.0);
        assert!(young.gh_pulse > old.gh_pulse);
    }

    #[test]
    fn chronic_deprivation_triggers_sasp() {
        let mut state = SleepState { sleep_quality: 0.1, ..Default::default() };
        let (_, _, sasp) = update_sleep(&mut state, &SleepParams::default(), 50.0, 1.0);
        assert!(sasp > 0.0, "severe sleep deprivation should boost SASP");
    }
}
