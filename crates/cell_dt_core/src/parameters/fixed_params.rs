use serde::{Deserialize, Serialize};

/// 32 параметра модели CDATA v3.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedParameters {
    // Базовые
    pub alpha: f64,
    pub hayflick_limit: f64,
    pub base_ros_young: f64,
    // Защита молодости
    pub pi_0: f64,
    pub tau_protection: f64,
    pub pi_baseline: f64,
    // Асимметрия деления
    pub p0_inheritance: f64,
    pub age_decline_rate: f64,
    pub fidelity_loss: f64,
    // Тканевые — HSC
    pub hsc_nu: f64,
    pub hsc_beta: f64,
    pub hsc_tau: f64,
    // Тканевые — ISC
    pub isc_nu: f64,
    pub isc_beta: f64,
    pub isc_tau: f64,
    // Тканевые — Muscle
    pub muscle_nu: f64,
    pub muscle_beta: f64,
    pub muscle_tau: f64,
    // Тканевые — Neural
    pub neural_nu: f64,
    pub neural_beta: f64,
    pub neural_tau: f64,
    // SASP
    pub stim_threshold: f64,
    pub inhib_threshold: f64,
    pub max_stimulation: f64,
    pub max_inhibition: f64,
    // CHIP
    pub dnmt3a_fitness: f64,
    pub dnmt3a_age_slope: f64,
    pub tet2_fitness: f64,
    pub tet2_age_slope: f64,
    // Прочие
    pub mtor_activity: f64,
    pub circadian_amplitude: f64,
    pub meiotic_reset: f64,
    pub yap_taz_sensitivity: f64,
}

impl Default for FixedParameters {
    fn default() -> Self {
        Self {
            alpha: 0.0082,
            hayflick_limit: 50.0,
            base_ros_young: 0.12,
            pi_0: 0.87,
            tau_protection: 24.3,
            pi_baseline: 0.10,
            p0_inheritance: 0.94,
            age_decline_rate: 0.15,
            fidelity_loss: 0.10,
            hsc_nu: 12.0,
            hsc_beta: 1.0,
            hsc_tau: 0.3,
            isc_nu: 70.0,
            isc_beta: 0.3,
            isc_tau: 0.8,
            muscle_nu: 4.0,
            muscle_beta: 1.2,
            muscle_tau: 0.5,
            neural_nu: 2.0,
            neural_beta: 1.5,
            neural_tau: 0.2,
            stim_threshold: 0.3,
            inhib_threshold: 0.8,
            max_stimulation: 1.5,
            max_inhibition: 0.3,
            dnmt3a_fitness: 0.15,
            dnmt3a_age_slope: 0.002,
            tet2_fitness: 0.12,
            tet2_age_slope: 0.0015,
            mtor_activity: 0.7,
            circadian_amplitude: 0.2,
            meiotic_reset: 0.8,
            yap_taz_sensitivity: 0.5,
        }
    }
}

impl FixedParameters {
    /// Validates internal consistency of parameters.
    /// Must pass before use in simulations.
    pub fn validate(&self) -> Result<(), String> {
        if self.pi_0 + self.pi_baseline > 1.0 {
            return Err(format!(
                "pi_0 ({}) + pi_baseline ({}) > 1.0: protection at t=0 would exceed 100%",
                self.pi_0, self.pi_baseline
            ));
        }
        if self.alpha <= 0.0 || self.alpha > 0.1 {
            return Err(format!("alpha ({}) out of plausible range (0, 0.1]", self.alpha));
        }
        if self.stim_threshold >= self.inhib_threshold {
            return Err(format!(
                "stim_threshold ({}) must be < inhib_threshold ({})",
                self.stim_threshold, self.inhib_threshold
            ));
        }
        for (name, val) in [("hsc_tau", self.hsc_tau), ("isc_tau", self.isc_tau),
                             ("muscle_tau", self.muscle_tau), ("neural_tau", self.neural_tau)] {
            if val <= 0.0 || val > 1.0 {
                return Err(format!("{} ({}) must be in (0, 1]", name, val));
            }
        }
        Ok(())
    }

    pub fn youth_protection(&self, age_years: f64) -> f64 {
        self.pi_0 * (-age_years / self.tau_protection).exp() + self.pi_baseline
    }

    pub fn inheritance_probability(&self, age_years: f64, spindle_fidelity: f64) -> f64 {
        let p = self.p0_inheritance
            - self.age_decline_rate * (age_years / 100.0)
            - self.fidelity_loss * (1.0 - spindle_fidelity);
        p.clamp(0.60, 0.98)
    }

    pub fn sasp_hormetic_response(&self, sasp: f64) -> f64 {
        if sasp < self.stim_threshold {
            1.0 + (self.max_stimulation - 1.0) / self.stim_threshold * sasp
        } else if sasp <= self.inhib_threshold {
            let range = self.inhib_threshold - self.stim_threshold;
            let t = (sasp - self.stim_threshold) / range;
            self.max_stimulation - (self.max_stimulation - 1.0) * t
        } else {
            1.0 / (1.0 + 3.0 * (sasp - self.inhib_threshold))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_32_parameters() {
        let p = FixedParameters::default();
        assert!((p.alpha - 0.0082).abs() < 1e-6);
        assert!((p.pi_0 - 0.87).abs() < 1e-6);
        assert!((p.hsc_nu - 12.0).abs() < 1e-6);
        assert!((p.isc_nu - 70.0).abs() < 1e-6);
        assert!((p.dnmt3a_fitness - 0.15).abs() < 1e-6);
    }

    #[test]
    fn test_youth_protection_decay() {
        let p = FixedParameters::default();
        assert!(p.youth_protection(0.0) > p.youth_protection(25.0));
        assert!(p.youth_protection(25.0) > p.youth_protection(100.0));
        assert!(p.youth_protection(100.0) >= p.pi_baseline);
    }

    #[test]
    fn test_inheritance_probability_bounds() {
        let p = FixedParameters::default();
        let prob = p.inheritance_probability(50.0, 0.8);
        assert!(prob >= 0.60, "prob={}", prob);
        assert!(prob <= 0.98, "prob={}", prob);
        // Молодой > Старый
        assert!(p.inheritance_probability(20.0, 1.0) > p.inheritance_probability(80.0, 0.5));
    }

    #[test]
    fn test_sasp_hormesis() {
        let p = FixedParameters::default();
        assert!(p.sasp_hormetic_response(0.1) > 1.0, "Low SASP should stimulate");
        assert!(p.sasp_hormetic_response(0.95) < 1.0, "High SASP should inhibit");
        // Пик где-то в районе stim_threshold
        assert!(p.sasp_hormetic_response(0.0).abs() <= p.max_stimulation + 0.01);
    }
}
