use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use cell_dt_core::FixedParameters;

#[derive(Debug, Default)]
pub struct AsymmetryStatistics {
    pub total_divisions: u64,
    pub maternal_inheritances: u64,
}

impl AsymmetryStatistics {
    pub fn record_division(&mut self, inherited_maternal: bool) {
        self.total_divisions += 1;
        if inherited_maternal {
            self.maternal_inheritances += 1;
        }
    }

    pub fn asymmetry_fraction(&self) -> f64 {
        if self.total_divisions == 0 { return 0.0; }
        self.maternal_inheritances as f64 / self.total_divisions as f64
    }
}

pub struct AsymmetricDivisionSystem {
    rng: ChaCha8Rng,
    pub stats: AsymmetryStatistics,
}

impl AsymmetricDivisionSystem {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            stats: AsymmetryStatistics::default(),
        }
    }

    pub fn calculate_probability(params: &FixedParameters, age_years: f64, spindle_fidelity: f64) -> f64 {
        params.inheritance_probability(age_years, spindle_fidelity)
    }

    pub fn roll_division(&mut self, params: &FixedParameters, age_years: f64, spindle_fidelity: f64) -> bool {
        let prob = Self::calculate_probability(params, age_years, spindle_fidelity);
        let inherited = self.rng.gen::<f64>() < prob;
        self.stats.record_division(inherited);
        inherited
    }

    pub fn damage_multiplier(inherited_maternal: bool) -> f64 {
        if inherited_maternal { 1.2 } else { 0.3 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probability_bounds() {
        let params = FixedParameters::default();
        let p_young = AsymmetricDivisionSystem::calculate_probability(&params, 20.0, 1.0);
        let p_old = AsymmetricDivisionSystem::calculate_probability(&params, 80.0, 0.5);
        assert!(p_young >= 0.60 && p_young <= 0.98, "p_young={}", p_young);
        assert!(p_old >= 0.60 && p_old <= 0.98, "p_old={}", p_old);
        assert!(p_young > p_old);
    }

    #[test]
    fn test_stochastic_distribution() {
        let params = FixedParameters::default();
        let mut sys = AsymmetricDivisionSystem::new(42);
        for _ in 0..1000 {
            sys.roll_division(&params, 50.0, 0.9);
        }
        let fraction = sys.stats.asymmetry_fraction();
        assert!(fraction > 0.7 && fraction < 0.99, "fraction={}", fraction);
    }
}
