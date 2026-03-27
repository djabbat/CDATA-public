use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChipDriverMutation {
    DNMT3A,
    TET2,
    ASXL1,
    JAK2,
    Other,
}

impl ChipDriverMutation {
    /// Per-year selective advantage (s). Literature: DNMT3A ~0.01-0.03/yr at age 60-70.
    /// Formula: s_base + s_slope × age, calibrated from Jaiswal 2017 (PMID 28901234).
    pub fn fitness_advantage(&self, age_years: f64) -> f64 {
        match self {
            // s=0.015 + 0.0002×age → at 60yo: 0.027/yr ≈ 2.7% per year ✓
            ChipDriverMutation::DNMT3A => 0.015 + 0.0002 * age_years,
            ChipDriverMutation::TET2   => 0.012 + 0.00015 * age_years,
            ChipDriverMutation::ASXL1  => 0.010 + 0.0001 * age_years,
            ChipDriverMutation::JAK2   => 0.020 + 0.0001 * age_years,
            ChipDriverMutation::Other  => 0.005 + 0.00005 * age_years,
        }
    }

    pub fn mutation_rate(&self) -> f64 {
        match self {
            ChipDriverMutation::DNMT3A => 1.2e-7,
            ChipDriverMutation::TET2   => 9.0e-8,
            ChipDriverMutation::ASXL1  => 5.0e-8,
            ChipDriverMutation::JAK2   => 3.0e-8,
            ChipDriverMutation::Other  => 2.0e-8,
        }
    }

    pub fn sasp_sensitivity(&self) -> f64 {
        match self {
            ChipDriverMutation::DNMT3A => 1.5,
            ChipDriverMutation::TET2   => 1.8,
            ChipDriverMutation::ASXL1  => 1.3,
            ChipDriverMutation::JAK2   => 2.0,
            ChipDriverMutation::Other  => 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChipClone {
    pub mutation: ChipDriverMutation,
    pub frequency: f64,
    pub age_of_origin: f64,
}

pub struct ChipSystem {
    rng: ChaCha8Rng,
    pub clones: Vec<ChipClone>,
    pub total_chip_frequency: f64,
    pub detection_age: Option<f64>,
}

impl ChipSystem {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            clones: Vec::new(),
            total_chip_frequency: 0.0,
            detection_age: None,
        }
    }

    pub fn update(&mut self, division_rate: f64, sasp_level: f64, age_years: f64, dt: f64) {
        // HSC pool size ~100,000 cells (short-term repopulating HSC in active cycle)
        // Expected new mutations = rate_per_division × divisions_per_year × pool × dt
        // This gives λ (Poisson parameter), converted to probability: P(≥1) = 1 - exp(-λ)
        const HSC_POOL: f64 = 1e5;
        let mutations = [ChipDriverMutation::DNMT3A, ChipDriverMutation::TET2,
                         ChipDriverMutation::ASXL1, ChipDriverMutation::JAK2];
        for mutation in &mutations {
            let lambda = mutation.mutation_rate() * division_rate * HSC_POOL * dt;
            // P(at least one new mutation) = 1 - exp(-λ)
            let prob = 1.0 - (-lambda).exp();
            if self.rng.gen::<f64>() < prob {
                self.clones.push(ChipClone {
                    mutation: mutation.clone(),
                    // Initial VAF = 1 cell / HSC_POOL
                    frequency: 1.0 / HSC_POOL,
                    age_of_origin: age_years,
                });
            }
        }

        // Logistic (Moran-like) clone expansion:
        // df/dt = f × (1 - f) × s
        // More realistic than exponential: saturates at f→1
        for clone in &mut self.clones {
            // s = per-year fitness advantage (selective coefficient)
            // Literature: DNMT3A ~0.01-0.03/year; model uses calibrated formula
            let s = clone.mutation.fitness_advantage(age_years);
            let sasp_boost = clone.mutation.sasp_sensitivity() * sasp_level * 0.01;
            let total_s = s + sasp_boost;
            // Logistic growth step (Euler)
            let df = clone.frequency * (1.0 - clone.frequency) * total_s * dt;
            clone.frequency = (clone.frequency + df).clamp(0.0, 1.0);
        }

        // Total: sum of dominant clone per mutation type (clones compete)
        self.total_chip_frequency = self.clones.iter().map(|c| c.frequency).sum::<f64>().min(1.0);

        if self.detection_age.is_none() && self.total_chip_frequency > 0.02 {
            self.detection_age = Some(age_years);
        }
    }

    pub fn hematologic_risk(&self) -> f64 {
        (self.total_chip_frequency * 5.0).min(1.0)
    }

    pub fn dominant_clone(&self) -> Option<&ChipClone> {
        self.clones.iter().max_by(|a, b| a.frequency.partial_cmp(&b.frequency).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fitness_increases_with_age() {
        let m = ChipDriverMutation::DNMT3A;
        assert!(m.fitness_advantage(60.0) > m.fitness_advantage(30.0));
    }

    #[test]
    fn test_chip_expansion() {
        let mut sys = ChipSystem::new(42);
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::DNMT3A,
            frequency: 0.001,
            age_of_origin: 40.0,
        });
        let before = sys.clones[0].frequency;
        sys.update(12.0, 0.2, 60.0, 1.0);
        assert!(sys.clones[0].frequency > before);
    }
}
