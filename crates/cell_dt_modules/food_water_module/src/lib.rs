//! Food & Water module — Ecosphere Level (+5) of CDATA hierarchy.
//!
//! ## Biological basis
//!
//! Nutrition and hydration are key environmental modulators of the aging program:
//!
//! - **Caloric intake** activates mTOR → accelerates cell division → speeds up
//!   telomere shortening and centriolar damage accumulation.
//! - **Caloric restriction (CR)** activates AMPK/SIRT1/FOXO3 → autophagy ↑,
//!   ROS↓, mito_shield ↑, Hayflick limit extension.
//! - **Protein quality** (antioxidants, phytochemicals) → proteostasis ↑,
//!   aggregation↓.
//! - **Hydration** → lysosomal function ↑ → autophagy flux ↑ → aggregate
//!   clearance ↑.
//!
//! ## Communication with CDATA core
//!
//! This module writes to ECS components from `cell_dt_core`:
//! - `MitochondrialState`: ros_production, mito_shield_contribution, mitophagy_flux
//! - `CentriolarDamageState`: protein_aggregates (via hydration/proteostasis)
//! - `InflammagingState`: niche_impairment (mTOR excess)

use cell_dt_core::{
    SimulationModule, SimulationResult,
    hecs::World,
    components::{CentriolarDamageState, MitochondrialState, InflammagingState},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use log::trace;

// ---------------------------------------------------------------------------
// ECS Component
// ---------------------------------------------------------------------------

/// ECS component: nutritional and hydration state.
/// Ecosphere level — represents the organism's dietary environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodWaterState {
    /// Caloric balance relative to maintenance: negative = restriction,
    /// positive = excess. Range: [-0.5 .. +0.5].
    pub caloric_balance: f32,

    /// Diet quality index [0..1]: 1.0 = Mediterranean / whole-foods,
    /// 0.0 = ultra-processed / high-AGE.
    pub diet_quality: f32,

    /// Hydration level [0..1]: 1.0 = optimal, 0.5 = mild dehydration.
    pub hydration: f32,

    /// Protein intake quality [0..1]: amino acid completeness + antioxidant load.
    pub protein_quality: f32,

    /// Accumulated mTOR modulation from diet this step.
    pub mtor_modulation: f32,

    /// Accumulated ROS modulation from diet this step.
    pub ros_modulation: f32,
}

impl Default for FoodWaterState {
    fn default() -> Self {
        Self {
            caloric_balance: 0.0,
            diet_quality:    0.8,
            hydration:       0.9,
            protein_quality: 0.7,
            mtor_modulation: 0.0,
            ros_modulation:  0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

/// Parameters controlling the food/water → aging relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodWaterParams {
    /// mTOR sensitivity to caloric balance [0..1].
    pub mtor_caloric_sensitivity: f32,

    /// ROS reduction per unit of diet quality above 0.5.
    pub diet_quality_ros_scale: f32,

    /// Autophagy boost from optimal hydration [0..0.2].
    pub hydration_autophagy_scale: f32,

    /// Proteostasis improvement per unit of protein_quality [0..0.15].
    pub protein_proteostasis_scale: f32,
}

impl Default for FoodWaterParams {
    fn default() -> Self {
        Self {
            mtor_caloric_sensitivity:   0.5,
            diet_quality_ros_scale:     0.10,
            hydration_autophagy_scale:  0.12,
            protein_proteostasis_scale: 0.08,
        }
    }
}

// ---------------------------------------------------------------------------
// Core update function (pure, testable)
// ---------------------------------------------------------------------------

/// Compute FoodWaterState outputs for one simulation step.
pub fn update_food_water(state: &mut FoodWaterState, params: &FoodWaterParams, dt: f32) {
    state.mtor_modulation = state.caloric_balance * params.mtor_caloric_sensitivity * dt;
    state.ros_modulation  = -(state.diet_quality - 0.5) * params.diet_quality_ros_scale * dt;
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

pub struct FoodWaterModule {
    pub params: FoodWaterParams,
}

impl Default for FoodWaterModule {
    fn default() -> Self { Self { params: FoodWaterParams::default() } }
}

impl FoodWaterModule {
    pub fn new() -> Self { Self::default() }
    pub fn with_params(params: FoodWaterParams) -> Self { Self { params } }
}

impl SimulationModule for FoodWaterModule {
    fn name(&self) -> &str { "food_water_module" }

    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        let entities: Vec<_> = world
            .query::<&CentriolarDamageState>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for e in entities {
            if world.get::<&FoodWaterState>(e).is_err() {
                let _ = world.insert_one(e, FoodWaterState::default());
            }
        }
        Ok(())
    }

    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()> {
        let dt32 = dt as f32;

        let entities: Vec<_> = world
            .query::<&FoodWaterState>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        for e in entities {
            let (mtor_mod, ros_mod, hydration, protein_q) = {
                let Ok(mut fw) = world.get::<&mut FoodWaterState>(e) else { continue };
                update_food_water(&mut fw, &self.params, dt32);
                (fw.mtor_modulation, fw.ros_modulation, fw.hydration, fw.protein_quality)
            };

            if let Ok(mut mito) = world.get::<&mut MitochondrialState>(e) {
                let hydration_boost = (hydration - 0.5).max(0.0)
                    * self.params.hydration_autophagy_scale * dt32;
                mito.mitophagy_flux = (mito.mitophagy_flux + hydration_boost).clamp(0.0, 1.0);
                mito.ros_production = (mito.ros_production + ros_mod).clamp(0.0, 1.0);

                let prot_boost = (protein_q - 0.5).max(0.0)
                    * self.params.protein_proteostasis_scale * dt32;
                mito.mito_shield_contribution =
                    (mito.mito_shield_contribution + prot_boost).clamp(0.0, 1.0);
            }

            // mTOR excess → niche_impairment (via InflammagingState)
            if mtor_mod > 0.0 {
                if let Ok(mut inf) = world.get::<&mut InflammagingState>(e) {
                    inf.niche_impairment = (inf.niche_impairment + mtor_mod * 0.3).clamp(0.0, 1.0);
                }
            }

            // Hydration → reduces protein_aggregates directly
            if let Ok(mut cda) = world.get::<&mut CentriolarDamageState>(e) {
                let agg_clear = (hydration - 0.5).max(0.0) * 0.05 * dt32;
                cda.protein_aggregates = (cda.protein_aggregates - agg_clear).clamp(0.0, 1.0);
            }

            trace!(
                "FoodWater: mtor_mod={:.4} ros_mod={:.4} hydration={:.2} protein_q={:.2}",
                mtor_mod, ros_mod, hydration, protein_q
            );
        }
        Ok(())
    }

    fn get_params(&self) -> serde_json::Value {
        json!({
            "mtor_caloric_sensitivity":   self.params.mtor_caloric_sensitivity,
            "diet_quality_ros_scale":     self.params.diet_quality_ros_scale,
            "hydration_autophagy_scale":  self.params.hydration_autophagy_scale,
            "protein_proteostasis_scale": self.params.protein_proteostasis_scale,
        })
    }

    fn set_params(&mut self, params: &serde_json::Value) -> SimulationResult<()> {
        if let Some(v) = params.get("mtor_caloric_sensitivity").and_then(|v| v.as_f64()) {
            self.params.mtor_caloric_sensitivity = v as f32;
        }
        if let Some(v) = params.get("diet_quality_ros_scale").and_then(|v| v.as_f64()) {
            self.params.diet_quality_ros_scale = v as f32;
        }
        if let Some(v) = params.get("hydration_autophagy_scale").and_then(|v| v.as_f64()) {
            self.params.hydration_autophagy_scale = v as f32;
        }
        if let Some(v) = params.get("protein_proteostasis_scale").and_then(|v| v.as_f64()) {
            self.params.protein_proteostasis_scale = v as f32;
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
    fn caloric_excess_activates_mtor() {
        let mut state = FoodWaterState { caloric_balance: 0.3, ..Default::default() };
        update_food_water(&mut state, &FoodWaterParams::default(), 1.0);
        assert!(state.mtor_modulation > 0.0);
    }

    #[test]
    fn caloric_restriction_suppresses_mtor() {
        let mut state = FoodWaterState { caloric_balance: -0.3, ..Default::default() };
        update_food_water(&mut state, &FoodWaterParams::default(), 1.0);
        assert!(state.mtor_modulation < 0.0);
    }

    #[test]
    fn high_diet_quality_reduces_ros() {
        let mut state = FoodWaterState { diet_quality: 1.0, ..Default::default() };
        update_food_water(&mut state, &FoodWaterParams::default(), 1.0);
        assert!(state.ros_modulation < 0.0);
    }

    #[test]
    fn normal_diet_neutral_mtor() {
        let mut state = FoodWaterState::default();
        update_food_water(&mut state, &FoodWaterParams::default(), 1.0);
        assert_eq!(state.mtor_modulation, 0.0);
    }
}
