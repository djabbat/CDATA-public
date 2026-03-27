use cell_dt_core::MitochondrialState;
use crate::params::{MitochondrialParams, sigmoid_ros, compute_mitophagy, accumulate_mtdna};

pub struct MitochondrialSystem {
    pub params: MitochondrialParams,
}

impl MitochondrialSystem {
    pub fn new() -> Self {
        Self { params: MitochondrialParams::default() }
    }

    pub fn update(&self, state: &mut MitochondrialState, dt: f64, age_years: f64, inflammation_level: f64) {
        state.mtdna_mutations = accumulate_mtdna(state.mtdna_mutations, state.ros_level, dt);
        let oxidative_input = inflammation_level * 0.3;
        state.ros_level = sigmoid_ros(
            state.mtdna_mutations, oxidative_input,
            self.params.ros_steepness, self.params.mitophagy_threshold,
        );
        state.mitophagy_efficiency = compute_mitophagy(
            state.ros_level, age_years, self.params.mitophagy_threshold,
        );
        state.mito_shield = (1.0 - age_years / 120.0).max(0.1);
        state.membrane_potential = (1.0 - state.mtdna_mutations * 0.5).max(0.2);
    }

    pub fn calculate_oxygen_delivery(&self, state: &MitochondrialState, age_years: f64) -> f64 {
        let base = 1.0 - age_years / 200.0;
        (base * state.membrane_potential).max(0.1)
    }

    pub fn check_mitochondrial_collapse(&self, state: &MitochondrialState) -> bool {
        state.mtdna_mutations > 0.9 || state.membrane_potential < 0.15
    }
}

impl Default for MitochondrialSystem {
    fn default() -> Self { Self::new() }
}
