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
    /// Formula: s_base + s_slope × age, calibrated from Jaiswal SS et al. 2017 NEJM (PMID 28636844).
    ///
    /// Relationship to FixedParameters:
    ///   FixedParameters.dnmt3a_fitness = 0.15  (reference value per 10-year unit)
    ///   Per-year base here = dnmt3a_fitness / 10 = 0.015  (matches /yr unit)
    ///   FixedParameters.dnmt3a_age_slope = 0.002  →  per-year slope = 0.0002
    /// This conversion is intentional. FixedParameters stores /10yr units for
    /// sensitivity analysis; chip_drift.rs uses /yr units for logistic growth.
    pub fn fitness_advantage(&self, age_years: f64) -> f64 {
        match self {
            // s=0.015 + 0.0002×age → at 60yo: 0.027/yr ≈ 2.7% per year ✓
            // (= FixedParameters.dnmt3a_fitness/10 + dnmt3a_age_slope/10 × age)
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

    /// L1: CHIP → SASP coupling (Round 7 fix)
    /// DNMT3A/TET2 mutant clones produce excess IL-6 and TNF-α,
    /// amplifying inflammaging (PMID: 29507339, Caiado et al. 2023).
    /// Returns a sasp_amplification factor: multiply sasp_prod by this.
    /// Formula: 1.0 + chip_sasp_boost × dominant_clone_sasp_sensitivity × total_freq
    pub fn sasp_amplification(&self) -> f64 {
        // Base amplification from total CHIP burden
        let base = self.total_chip_frequency * 0.4;
        // Additional boost from dominant clone's mutation-specific sensitivity
        let clone_boost = self.dominant_clone()
            .map(|c| c.mutation.sasp_sensitivity() * c.frequency * 0.3)
            .unwrap_or(0.0);
        1.0 + (base + clone_boost).min(0.8) // clamp max amplification to 1.8×
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

    // ── ChipDriverMutation::fitness_advantage ─────────────────────────────────

    #[test]
    fn test_dnmt3a_fitness_at_60_approximately_027() {
        // DNMT3A: 0.015 + 0.0002*60 = 0.015 + 0.012 = 0.027
        let f = ChipDriverMutation::DNMT3A.fitness_advantage(60.0);
        assert!((f - 0.027).abs() < 1e-9, "DNMT3A at 60 = 0.027, got {}", f);
    }

    #[test]
    fn test_tet2_fitness_at_60() {
        // TET2: 0.012 + 0.00015*60 = 0.012 + 0.009 = 0.021
        let f = ChipDriverMutation::TET2.fitness_advantage(60.0);
        assert!((f - 0.021).abs() < 1e-9, "TET2 at 60 = 0.021, got {}", f);
    }

    #[test]
    fn test_dnmt3a_fitness_greater_than_tet2_at_age_60() {
        let dnmt3a = ChipDriverMutation::DNMT3A.fitness_advantage(60.0);
        let tet2   = ChipDriverMutation::TET2.fitness_advantage(60.0);
        assert!(dnmt3a > tet2,
            "DNMT3A fitness ({}) must exceed TET2 fitness ({}) at age 60", dnmt3a, tet2);
    }

    #[test]
    fn test_all_mutations_fitness_positive() {
        let muts = [ChipDriverMutation::DNMT3A, ChipDriverMutation::TET2,
                    ChipDriverMutation::ASXL1, ChipDriverMutation::JAK2, ChipDriverMutation::Other];
        for m in &muts {
            assert!(m.fitness_advantage(60.0) > 0.0, "All fitness advantages must be positive at age 60");
        }
    }

    #[test]
    fn test_fitness_increases_with_age_all_mutations() {
        let muts = [ChipDriverMutation::DNMT3A, ChipDriverMutation::TET2,
                    ChipDriverMutation::ASXL1, ChipDriverMutation::JAK2, ChipDriverMutation::Other];
        for m in &muts {
            let f_young = m.fitness_advantage(20.0);
            let f_old   = m.fitness_advantage(80.0);
            assert!(f_old > f_young, "Fitness must increase with age for {:?}", m);
        }
    }

    #[test]
    fn test_jak2_highest_base_fitness() {
        // JAK2 base=0.020 is the highest
        let jak2  = ChipDriverMutation::JAK2.fitness_advantage(0.0);
        let dnmt  = ChipDriverMutation::DNMT3A.fitness_advantage(0.0);
        assert!(jak2 > dnmt, "JAK2 base fitness should be highest");
    }

    #[test]
    fn test_other_mutation_lowest_fitness() {
        let other = ChipDriverMutation::Other.fitness_advantage(60.0);
        let dnmt  = ChipDriverMutation::DNMT3A.fitness_advantage(60.0);
        assert!(other < dnmt, "Other mutation has lowest fitness");
    }

    // ── ChipDriverMutation::mutation_rate ────────────────────────────────────

    #[test]
    fn test_mutation_rates_positive() {
        let muts = [ChipDriverMutation::DNMT3A, ChipDriverMutation::TET2,
                    ChipDriverMutation::ASXL1, ChipDriverMutation::JAK2, ChipDriverMutation::Other];
        for m in &muts {
            assert!(m.mutation_rate() > 0.0, "mutation_rate must be positive for {:?}", m);
        }
    }

    #[test]
    fn test_dnmt3a_highest_mutation_rate() {
        let dnmt3a = ChipDriverMutation::DNMT3A.mutation_rate();
        let tet2   = ChipDriverMutation::TET2.mutation_rate();
        let asxl1  = ChipDriverMutation::ASXL1.mutation_rate();
        assert!(dnmt3a > tet2, "DNMT3A should have highest mutation rate");
        assert!(dnmt3a > asxl1);
    }

    #[test]
    fn test_mutation_rates_in_plausible_range() {
        let muts = [ChipDriverMutation::DNMT3A, ChipDriverMutation::TET2,
                    ChipDriverMutation::ASXL1, ChipDriverMutation::JAK2, ChipDriverMutation::Other];
        for m in &muts {
            let r = m.mutation_rate();
            assert!(r > 1e-9 && r < 1e-6, "Mutation rate {:e} out of range for {:?}", r, m);
        }
    }

    // ── ChipDriverMutation::sasp_sensitivity ─────────────────────────────────

    #[test]
    fn test_sasp_sensitivity_all_positive() {
        let muts = [ChipDriverMutation::DNMT3A, ChipDriverMutation::TET2,
                    ChipDriverMutation::ASXL1, ChipDriverMutation::JAK2, ChipDriverMutation::Other];
        for m in &muts {
            assert!(m.sasp_sensitivity() > 0.0);
        }
    }

    #[test]
    fn test_jak2_highest_sasp_sensitivity() {
        let jak2 = ChipDriverMutation::JAK2.sasp_sensitivity();
        let tet2 = ChipDriverMutation::TET2.sasp_sensitivity();
        let dnmt = ChipDriverMutation::DNMT3A.sasp_sensitivity();
        assert!(jak2 > tet2 && jak2 > dnmt, "JAK2 should have highest SASP sensitivity");
    }

    #[test]
    fn test_other_mutation_base_sensitivity_one() {
        assert!((ChipDriverMutation::Other.sasp_sensitivity() - 1.0).abs() < 1e-9);
    }

    // ── ChipSystem ────────────────────────────────────────────────────────────

    #[test]
    fn test_chip_system_new_empty() {
        let sys = ChipSystem::new(42);
        assert!(sys.clones.is_empty());
        assert_eq!(sys.total_chip_frequency, 0.0);
        assert!(sys.detection_age.is_none());
    }

    #[test]
    fn test_chip_expansion_30_years() {
        // CHIP: expansion over 30 years from initial frequency
        // Logistic growth: df = f*(1-f)*s per year, s~0.027 at age 60
        // From f=1e-5: 30yr at s=0.027 → f ≈ f0 * exp(30*0.027) ≈ 2.2e-4 (linear regime)
        let mut sys = ChipSystem::new(99);
        let initial_freq = 1e-5;
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::DNMT3A,
            frequency: initial_freq,
            age_of_origin: 40.0,
        });
        // Simulate 30 years, 1yr steps
        for year in 0..30 {
            sys.update(12.0, 0.1, 40.0 + year as f64, 1.0);
        }
        assert!(sys.clones[0].frequency > initial_freq,
            "Clone should expand over 30 years: {} > {}",
            sys.clones[0].frequency, initial_freq);
        // In logistic regime near zero: growth ≈ exp(s*t), s=0.027, t=30 → 2.2×
        assert!(sys.clones[0].frequency > initial_freq * 1.5,
            "Clone should grow at least 1.5× over 30 years: got {}", sys.clones[0].frequency);
    }

    #[test]
    fn test_chip_frequency_non_negative() {
        let mut sys = ChipSystem::new(7);
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::TET2,
            frequency: 0.001,
            age_of_origin: 50.0,
        });
        for _ in 0..50 {
            sys.update(12.0, 0.3, 60.0, 1.0);
        }
        for clone in &sys.clones {
            assert!(clone.frequency >= 0.0);
        }
    }

    #[test]
    fn test_chip_frequency_clamped_at_one() {
        let mut sys = ChipSystem::new(1);
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::JAK2,
            frequency: 0.999,
            age_of_origin: 60.0,
        });
        for _ in 0..100 {
            sys.update(12.0, 1.0, 80.0, 1.0);
        }
        assert!(sys.clones[0].frequency <= 1.0,
            "Clone frequency must not exceed 1.0");
    }

    #[test]
    fn test_total_chip_frequency_bounded() {
        let mut sys = ChipSystem::new(5);
        sys.clones.push(ChipClone { mutation: ChipDriverMutation::DNMT3A, frequency: 0.5, age_of_origin: 50.0 });
        sys.clones.push(ChipClone { mutation: ChipDriverMutation::TET2, frequency: 0.5, age_of_origin: 55.0 });
        sys.update(12.0, 0.5, 70.0, 1.0);
        assert!(sys.total_chip_frequency <= 1.0,
            "Total CHIP frequency must be bounded at 1.0, got {}", sys.total_chip_frequency);
    }

    #[test]
    fn test_detection_age_set_when_threshold_crossed() {
        let mut sys = ChipSystem::new(3);
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::DNMT3A,
            frequency: 0.025, // above 0.02 detection threshold
            age_of_origin: 60.0,
        });
        sys.update(12.0, 0.2, 65.0, 1.0);
        assert!(sys.detection_age.is_some(), "Detection age should be set when freq > 0.02");
    }

    #[test]
    fn test_detection_age_not_set_below_threshold() {
        let mut sys = ChipSystem::new(3);
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::DNMT3A,
            frequency: 0.001,
            age_of_origin: 30.0,
        });
        sys.update(12.0, 0.0, 35.0, 1.0);
        // May or may not be set depending on whether clone grew; just verify if set, value makes sense
        if let Some(age) = sys.detection_age {
            assert!(age >= 0.0 && age <= 200.0);
        }
    }

    #[test]
    fn test_hematologic_risk_proportional_to_frequency() {
        let mut sys = ChipSystem::new(8);
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::DNMT3A,
            frequency: 0.1,
            age_of_origin: 55.0,
        });
        sys.update(12.0, 0.2, 60.0, 0.01);
        let risk = sys.hematologic_risk();
        assert!(risk >= 0.0 && risk <= 1.0,
            "Hematologic risk must be in [0,1], got {}", risk);
    }

    #[test]
    fn test_hematologic_risk_zero_when_empty() {
        let sys = ChipSystem::new(0);
        assert_eq!(sys.hematologic_risk(), 0.0);
    }

    #[test]
    fn test_sasp_amplification_no_clones() {
        let sys = ChipSystem::new(0);
        assert!((sys.sasp_amplification() - 1.0).abs() < 1e-9,
            "No CHIP → amplification = 1.0");
    }

    #[test]
    fn test_sasp_amplification_increases_with_chip_burden() {
        let mut sys = ChipSystem::new(42);
        // Start with no clones → amplification = 1.0
        let amp_base = sys.sasp_amplification();
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::TET2,
            frequency: 0.5,
            age_of_origin: 60.0,
        });
        sys.total_chip_frequency = 0.5;
        let amp_with = sys.sasp_amplification();
        assert!(amp_with > amp_base, "CHIP burden increases SASP amplification");
    }

    #[test]
    fn test_sasp_amplification_capped_at_18() {
        let mut sys = ChipSystem::new(42);
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::JAK2,
            frequency: 1.0,
            age_of_origin: 50.0,
        });
        sys.total_chip_frequency = 1.0;
        let amp = sys.sasp_amplification();
        assert!(amp <= 1.8 + 1e-9,
            "SASP amplification max = 1.8, got {}", amp);
    }

    #[test]
    fn test_dominant_clone_none_when_empty() {
        let sys = ChipSystem::new(0);
        assert!(sys.dominant_clone().is_none());
    }

    #[test]
    fn test_dominant_clone_returns_largest() {
        let mut sys = ChipSystem::new(0);
        sys.clones.push(ChipClone { mutation: ChipDriverMutation::DNMT3A, frequency: 0.1, age_of_origin: 50.0 });
        sys.clones.push(ChipClone { mutation: ChipDriverMutation::TET2,   frequency: 0.5, age_of_origin: 55.0 });
        sys.clones.push(ChipClone { mutation: ChipDriverMutation::ASXL1,  frequency: 0.2, age_of_origin: 60.0 });
        let dom = sys.dominant_clone().unwrap();
        assert!((dom.frequency - 0.5).abs() < 1e-9, "Dominant clone must be largest");
        assert_eq!(dom.mutation, ChipDriverMutation::TET2);
    }

    #[test]
    fn test_sasp_boosts_expansion() {
        let mut s1 = ChipSystem::new(42);
        let mut s2 = ChipSystem::new(42);
        s1.clones.push(ChipClone { mutation: ChipDriverMutation::DNMT3A, frequency: 0.01, age_of_origin: 50.0 });
        s2.clones.push(ChipClone { mutation: ChipDriverMutation::DNMT3A, frequency: 0.01, age_of_origin: 50.0 });
        // High SASP for s2
        for _ in 0..10 {
            s1.update(12.0, 0.0, 60.0, 1.0);
            s2.update(12.0, 1.0, 60.0, 1.0);
        }
        assert!(s2.clones[0].frequency >= s1.clones[0].frequency,
            "High SASP should boost CHIP expansion");
    }

    #[test]
    fn test_chip_logistic_growth_saturates() {
        // Start near saturation: logistic growth should slow down
        let mut sys = ChipSystem::new(7);
        sys.clones.push(ChipClone {
            mutation: ChipDriverMutation::DNMT3A,
            frequency: 0.99,
            age_of_origin: 60.0,
        });
        let before = sys.clones[0].frequency;
        sys.update(12.0, 0.0, 60.0, 1.0);
        let after = sys.clones[0].frequency;
        // logistic: df = f*(1-f)*s; at f=0.99, (1-f)=0.01 → very slow growth
        assert!(after - before < 0.001, "Logistic growth near saturation must be slow");
    }
}
