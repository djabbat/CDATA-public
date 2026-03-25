//! Breathing module — Ecosphere Level (+5) of CDATA hierarchy.
//!
//! ## Biological basis
//!
//! Breathing controls O₂ delivery to tissues — the core CDATA damage vector:
//!
//! - **Hyperoxia** (supplemental O₂, low altitude) → more O₂ reaches centrioles
//!   → faster inducer detachment → shorter Hayflick limit
//! - **Hypoxia** (high altitude, sleep apnea) → HIF-1α → VEGF ↑,
//!   anaerobic shift → mitochondrial uncoupling → more ROS
//! - **Optimal breathing** (slow diaphragmatic) → CO₂ retention → Bohr effect
//!   → better tissue O₂ unloading without centrosomal exposure
//! - **Air pollution** (PM2.5, NOx) → oxidative stress → ROS ↑
//!
//! ## Key CDATA insight
//!
//! `mito_shield_contribution` protects centrioles from O₂. This module modifies
//! that shield based on ambient pO₂ and breathing quality, allowing simulation
//! of altitude training, pranayama, and pollution effects on longevity.
//!
//! ## Communication with CDATA core
//!
//! Writes to:
//! - `MitochondrialState.mito_shield_contribution` (O₂ shield ±)
//! - `MitochondrialState.ros_production` (pollution, hypoxia ROS)

use cell_dt_core::{
    SimulationModule, SimulationResult,
    hecs::World,
    components::{CentriolarDamageState, MitochondrialState},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::trace;

// ---------------------------------------------------------------------------
// ECS Component
// ---------------------------------------------------------------------------

/// ECS component: breathing and gas exchange state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathingState {
    /// Fraction of O₂ in inspired air [0..1]. Sea level = 0.21.
    pub o2_fraction: f32,

    /// Breathing pattern quality [0..1]: 1.0 = optimal diaphragmatic slow.
    pub breathing_quality: f32,

    /// Air pollution index [0..1]: PM2.5, NOx → oxidative stress.
    pub pollution_index: f32,

    /// Computed O₂ modifier at centrosomal zone (positive = more O₂ reaching centrioles).
    pub centrosomal_o2_modifier: f32,

    /// ROS contribution from breathing environment.
    pub ros_contribution: f32,
}

impl Default for BreathingState {
    fn default() -> Self {
        Self {
            o2_fraction:             0.21,
            breathing_quality:       0.75,
            pollution_index:         0.10,
            centrosomal_o2_modifier: 0.0,
            ros_contribution:        0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathingParams {
    /// Reference O₂ fraction (sea level = 0.21).
    pub reference_o2: f32,

    /// Sensitivity of centrosomal O₂ to o2_fraction deviation.
    pub o2_centrosomal_sensitivity: f32,

    /// How much good breathing quality reduces centrosomal O₂ modifier (Bohr effect).
    pub breathing_quality_shield_scale: f32,

    /// ROS production per unit of pollution_index.
    pub pollution_ros_scale: f32,
}

impl Default for BreathingParams {
    fn default() -> Self {
        Self {
            reference_o2:                    0.21,
            o2_centrosomal_sensitivity:      2.0,
            breathing_quality_shield_scale:  0.15,
            pollution_ros_scale:             0.20,
        }
    }
}

// ---------------------------------------------------------------------------
// Core update function (pure, testable)
// ---------------------------------------------------------------------------

pub fn update_breathing(state: &mut BreathingState, params: &BreathingParams, dt: f32) {
    let o2_dev = state.o2_fraction - params.reference_o2;
    let breathing_shield = (state.breathing_quality - 0.5).max(0.0)
        * params.breathing_quality_shield_scale;
    state.centrosomal_o2_modifier =
        (o2_dev * params.o2_centrosomal_sensitivity - breathing_shield) * dt;
    state.ros_contribution = state.pollution_index * params.pollution_ros_scale * dt;
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

pub struct BreathingModule {
    pub params: BreathingParams,
}

impl Default for BreathingModule {
    fn default() -> Self { Self { params: BreathingParams::default() } }
}

impl BreathingModule {
    pub fn new() -> Self { Self::default() }

    pub fn high_altitude() -> Self {
        Self { params: BreathingParams { reference_o2: 0.21, ..Default::default() } }
    }

    pub fn urban_pollution() -> Self {
        Self { params: BreathingParams { pollution_ros_scale: 0.35, ..Default::default() } }
    }

    pub fn pranayama() -> Self {
        Self { params: BreathingParams {
            breathing_quality_shield_scale: 0.25,
            ..Default::default()
        } }
    }
}

impl SimulationModule for BreathingModule {
    fn name(&self) -> &str { "breathing_module" }

    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        let entities: Vec<_> = world
            .query::<&CentriolarDamageState>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for e in entities {
            if world.get::<&BreathingState>(e).is_err() {
                let _ = world.insert_one(e, BreathingState::default());
            }
        }
        Ok(())
    }

    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()> {
        let dt_years = (dt / 365.25) as f32;

        let entities: Vec<_> = world
            .query::<&BreathingState>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        for e in entities {
            let (o2_modifier, ros_contrib) = {
                let Ok(mut br) = world.get::<&mut BreathingState>(e) else { continue };
                update_breathing(&mut br, &self.params, dt_years);
                (br.centrosomal_o2_modifier, br.ros_contribution)
            };

            if let Ok(mut mito) = world.get::<&mut MitochondrialState>(e) {
                // Hyperoxia (o2_modifier > 0) weakens mito shield
                mito.mito_shield_contribution =
                    (mito.mito_shield_contribution - o2_modifier).clamp(0.0, 1.0);
                mito.ros_production = (mito.ros_production + ros_contrib).clamp(0.0, 1.0);
            }

            trace!(
                "Breathing: o2_mod={:.4} ros_contrib={:.4}",
                o2_modifier, ros_contrib
            );
        }
        Ok(())
    }

    fn get_params(&self) -> Value {
        json!({
            "reference_o2":                    self.params.reference_o2,
            "o2_centrosomal_sensitivity":       self.params.o2_centrosomal_sensitivity,
            "breathing_quality_shield_scale":   self.params.breathing_quality_shield_scale,
            "pollution_ros_scale":              self.params.pollution_ros_scale,
        })
    }

    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(v) = params.get("reference_o2").and_then(|v| v.as_f64()) {
            self.params.reference_o2 = v as f32;
        }
        if let Some(v) = params.get("o2_centrosomal_sensitivity").and_then(|v| v.as_f64()) {
            self.params.o2_centrosomal_sensitivity = v as f32;
        }
        if let Some(v) = params.get("breathing_quality_shield_scale").and_then(|v| v.as_f64()) {
            self.params.breathing_quality_shield_scale = v as f32;
        }
        if let Some(v) = params.get("pollution_ros_scale").and_then(|v| v.as_f64()) {
            self.params.pollution_ros_scale = v as f32;
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
    fn hyperoxia_increases_centrosomal_o2() {
        let mut state = BreathingState { o2_fraction: 0.35, breathing_quality: 0.5, ..Default::default() };
        update_breathing(&mut state, &BreathingParams::default(), 1.0);
        assert!(state.centrosomal_o2_modifier > 0.0);
    }

    #[test]
    fn hypoxia_reduces_centrosomal_o2() {
        let mut state = BreathingState { o2_fraction: 0.14, breathing_quality: 0.5, ..Default::default() };
        update_breathing(&mut state, &BreathingParams::default(), 1.0);
        assert!(state.centrosomal_o2_modifier < 0.0);
    }

    #[test]
    fn sea_level_minimal_modifier() {
        let mut state = BreathingState { o2_fraction: 0.21, breathing_quality: 0.5, ..Default::default() };
        update_breathing(&mut state, &BreathingParams::default(), 1.0);
        assert!(state.centrosomal_o2_modifier.abs() < 0.05);
    }

    #[test]
    fn pollution_raises_ros() {
        let mut state = BreathingState { pollution_index: 0.8, ..Default::default() };
        update_breathing(&mut state, &BreathingParams::default(), 1.0);
        assert!(state.ros_contribution > 0.10);
    }

    #[test]
    fn pranayama_shields_centrioles() {
        let mut s_normal = BreathingState { o2_fraction: 0.21, breathing_quality: 0.5, ..Default::default() };
        update_breathing(&mut s_normal, &BreathingParams::default(), 1.0);

        let mut s_prana = BreathingState { o2_fraction: 0.21, breathing_quality: 1.0, ..Default::default() };
        update_breathing(&mut s_prana, &BreathingParams { breathing_quality_shield_scale: 0.25, ..Default::default() }, 1.0);

        assert!(s_prana.centrosomal_o2_modifier < s_normal.centrosomal_o2_modifier);
    }
}
