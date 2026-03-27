use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsymmetricInheritance {
    pub inheritance_probability: f64,
    pub inherited_maternal_last: bool,
    pub total_divisions: u64,
    pub maternal_inheritance_count: u64,
}

impl Default for AsymmetricInheritance {
    fn default() -> Self {
        Self {
            inheritance_probability: 0.94,
            inherited_maternal_last: true,
            total_divisions: 0,
            maternal_inheritance_count: 0,
        }
    }
}

impl AsymmetricInheritance {
    pub fn asymmetry_fraction(&self) -> f64 {
        if self.total_divisions == 0 { return 0.0; }
        self.maternal_inheritance_count as f64 / self.total_divisions as f64
    }
}
