use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TissueState {
    pub age_years: f64,
    pub stem_cell_pool: f64,
    pub centriole_damage: f64,
    pub division_count: u64,
    pub frailty_index: f64,
    pub epigenetic_age: f64,
    pub telomere_length: f64,
}

impl Default for TissueState {
    fn default() -> Self {
        Self {
            age_years: 0.0,
            stem_cell_pool: 1.0,
            centriole_damage: 0.0,
            division_count: 0,
            frailty_index: 0.0,
            epigenetic_age: 0.0,
            telomere_length: 1.0,
        }
    }
}

impl TissueState {
    pub fn new(age_years: f64) -> Self {
        Self { age_years, epigenetic_age: age_years, ..Default::default() }
    }

    pub fn is_viable(&self) -> bool {
        self.stem_cell_pool > 0.05 && self.frailty_index < 0.95
    }
}
