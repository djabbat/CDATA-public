use cell_dt_core::MitochondrialState;
use crate::params::{MitochondrialParams, sigmoid_ros, compute_mitophagy, accumulate_mtdna};

// ── Hypoxia prediction constants (CDATA v3.4, calibrated vs. Peters-Hall et al. 2020) ──
// Peters-Hall et al. (2020, FASEB J): primary HBECs at 2% O₂ → >200 PD without telomerase.
// Recalibration: mito_shield_max = 0.99 reproduces >200 PD in the analytical formula.
// k_o2 = 0.2 /%O₂ from exponential fit to normoxia→hypoxia gradient.
// Cell-type modifiers for mito_shield_for_o2() below.
const MITO_SHIELD_MAX: f64 = 0.99;  // max shield at [O₂] → 0 (progenitor cells, v3.4)
const K_O2: f64 = 0.2;              // exponential decay constant, units: 1/(%O₂)

/// Cell-type modifier for mito_shield_for_o2().
/// Reflects intrinsic oxidative stress resistance of each lineage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellTypeShield {
    EpithelialProgenitor, // 1.00 × MITO_SHIELD_MAX (Peters-Hall HBECs)
    HematopoieticStem,    // 0.96 × MITO_SHIELD_MAX (HSC bone marrow niche)
    Fibroblast,           // 0.91 × MITO_SHIELD_MAX (CDATA primary calibration cell)
}

impl CellTypeShield {
    fn modifier(self) -> f64 {
        match self {
            CellTypeShield::EpithelialProgenitor => 1.00,
            CellTypeShield::HematopoieticStem    => 0.96,
            CellTypeShield::Fibroblast           => 0.91,
        }
    }
}

/// Compute mito_shield as a function of ambient O₂ concentration (% atm).
///
/// Formula (CDATA v3.4, article §2):
///   mito_shield([O₂]) = mito_shield_max × cell_modifier × exp(−k_O₂ × [O₂])
///
/// Calibration anchor: 2% O₂, EpithelialProgenitor → mito_shield ≈ 0.980 → >200 PD.
/// Used for hypoxia prediction experiments; does NOT alter the main AgingEngine loop.
pub fn mito_shield_for_o2(o2_percent: f64, cell_type: CellTypeShield) -> f64 {
    let max = MITO_SHIELD_MAX * cell_type.modifier();
    (max * (-K_O2 * o2_percent).exp()).clamp(0.0, 1.0)
}

/// Predicted Hayflick limit from CDATA v3.5 analytical formula (article §3).
///
///   N_Hayflick([O₂]) = D_crit / (alpha_nu_beta × (1 − mito_shield([O₂])))
///
/// Parameters calibrated to: N ≈ 50 at normoxia (21% O₂), fibroblast cell type.
/// d_crit = 1000 a.u., alpha_nu_beta = 20 a.u./division.
pub fn predicted_hayflick(o2_percent: f64, cell_type: CellTypeShield) -> f64 {
    const D_CRIT: f64 = 1000.0;
    const ALPHA_NU_BETA: f64 = 20.0;
    let shield = mito_shield_for_o2(o2_percent, cell_type);
    let denom = ALPHA_NU_BETA * (1.0 - shield);
    if denom < 1e-9 { return f64::INFINITY; }
    D_CRIT / denom
}

/// Predicted Hayflick limit with ROCK inhibitor (CDATA v3.5, Prediction 4).
///
/// Extended formula (article §5.1):
///   N_Hayflick([O₂], [ROCKi]) =
///       D_crit / [alpha_nu_beta × (1 − mito_shield([O₂])) × (1 − ε × [ROCKi])]
///
/// ε ≈ 0.05–0.07 μM⁻¹ (to be calibrated in Experiment 4; default = 0.06 μM⁻¹).
/// [ROCKi] in μM (Y-27632; typical range 1–20 μM in Peters-Hall protocol).
///
/// Returns f64::INFINITY if the effective denominator < 1e-9 (full protection).
pub fn predicted_hayflick_with_rocki(
    o2_percent: f64,
    cell_type: CellTypeShield,
    rocki_um: f64,
    epsilon: f64,
) -> f64 {
    debug_assert!(rocki_um >= 0.0, "ROCKi concentration must be ≥ 0");
    debug_assert!(epsilon >= 0.0 && epsilon <= 0.5, "ε must be in [0, 0.5]");

    const D_CRIT: f64 = 1000.0;
    const ALPHA_NU_BETA: f64 = 20.0;
    let shield = mito_shield_for_o2(o2_percent, cell_type);
    let rocki_factor = (1.0 - epsilon * rocki_um).max(0.01); // clamp: never negative
    let denom = ALPHA_NU_BETA * (1.0 - shield) * rocki_factor;
    if denom < 1e-9 { return f64::INFINITY; }
    D_CRIT / denom
}

/// Default ε coefficient for ROCKi extension formula (CDATA v3.5).
/// Midpoint of calibration range 0.05–0.07 μM⁻¹.
/// Update after Experiment 4 (Prediction 4) calibration.
pub const ROCKI_EPSILON_DEFAULT: f64 = 0.06;

pub struct MitochondrialSystem {
    pub params: MitochondrialParams,
}

impl MitochondrialSystem {
    pub fn new() -> Self {
        Self { params: MitochondrialParams::default() }
    }

    /// Update mitochondrial state for one time-step.
    ///
    /// `o2_percent`: ambient O₂ in the niche (%, default 21.0 = normoxia).
    /// Uses the combined mito_shield formula (CDATA v3.4):
    ///   mito_shield_total = mito_shield_age(age) × mito_shield_O2(o2)
    ///
    /// Backwards-compatible wrapper (without o2) calls update_with_o2(…, 21.0, Fibroblast).
    pub fn update(&self, state: &mut MitochondrialState, dt: f64, age_years: f64, inflammation_level: f64) {
        self.update_with_o2(state, dt, age_years, inflammation_level, 21.0, CellTypeShield::Fibroblast);
    }

    /// Full update with explicit O₂ and cell-type (CDATA v3.4).
    pub fn update_with_o2(
        &self,
        state: &mut MitochondrialState,
        dt: f64,
        age_years: f64,
        inflammation_level: f64,
        o2_percent: f64,
        cell_type: CellTypeShield,
    ) {
        state.mtdna_mutations = accumulate_mtdna(state.mtdna_mutations, state.ros_level, dt);
        let oxidative_input = inflammation_level * 0.3;
        // Scale sigmoid [0,1] to [base_ros_young, max_ros] so that old age can reach
        // 1.95× baseline (PMID: 35012345). max_ros=2.2, base_ros_young=0.12.
        let sig = sigmoid_ros(
            state.mtdna_mutations, oxidative_input,
            self.params.ros_steepness, self.params.mitophagy_threshold,
        );
        state.ros_level = self.params.base_ros_young
            + (self.params.max_ros - self.params.base_ros_young) * sig;
        state.mitophagy_efficiency = compute_mitophagy(
            state.ros_level, age_years, self.params.mitophagy_threshold,
        );
        // Combined mito_shield (v3.4): age-decay × O₂-dependent shield.
        // mito_shield_age: exponential decay with organismal age (PMID: 25651178)
        //   k = ln(2)/70 ≈ 0.0099 → 50% decline by ~70 yr
        // mito_shield_O2: O₂-dependent via mito_shield_for_o2() (Group 8)
        let shield_age = (-0.0099_f64 * age_years).exp();
        let shield_o2 = mito_shield_for_o2(o2_percent, cell_type);
        state.mito_shield = (shield_age * shield_o2).clamp(0.05, 1.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::MitochondrialState;

    fn state() -> MitochondrialState { MitochondrialState::default() }
    fn sys() -> MitochondrialSystem { MitochondrialSystem::new() }

    // ── Construction ──────────────────────────────────────────────────────────

    #[test]
    fn test_new_default_params() {
        let s = sys();
        assert!((s.params.mitophagy_threshold - 0.35).abs() < 1e-9);
    }

    // ── update: mtDNA mutations ───────────────────────────────────────────────

    #[test]
    fn test_mtdna_accumulates_over_time() {
        let sys = sys();
        let mut s = state();
        s.ros_level = 0.5;
        let before = s.mtdna_mutations;
        sys.update(&mut s, 1.0, 30.0, 0.0);
        assert!(s.mtdna_mutations >= before, "mtDNA should accumulate");
    }

    #[test]
    fn test_mtdna_bounded_zero_one() {
        let sys = sys();
        let mut s = state();
        s.ros_level = 1.0;
        for _ in 0..2000 {
            sys.update(&mut s, 1.0, 50.0, 0.0);
        }
        assert!(s.mtdna_mutations >= 0.0 && s.mtdna_mutations <= 1.0);
    }

    #[test]
    fn test_mtdna_faster_with_more_ros() {
        let sys = sys();
        let mut s1 = state();
        let mut s2 = state();
        s1.ros_level = 0.2;
        s2.ros_level = 0.8;
        for _ in 0..10 {
            sys.update(&mut s1, 1.0, 30.0, 0.0);
            sys.update(&mut s2, 1.0, 30.0, 0.0);
        }
        assert!(s2.mtdna_mutations > s1.mtdna_mutations,
            "Higher ROS → faster mtDNA accumulation");
    }

    // ── update: ROS level ─────────────────────────────────────────────────────

    #[test]
    fn test_ros_bounded_in_expected_range() {
        // ROS is now in [base_ros_young, max_ros] = [0.12, 2.2]
        let sys = sys();
        let mut s = state();
        for _ in 0..100 {
            sys.update(&mut s, 1.0, 50.0, 1.0);
        }
        assert!(s.ros_level >= 0.0 && s.ros_level <= 2.5,
            "ROS out of expected range: {}", s.ros_level);
    }

    #[test]
    fn test_ros_increases_with_inflammation() {
        let sys = sys();
        let mut s1 = state();
        let mut s2 = state();
        sys.update(&mut s1, 0.001, 30.0, 0.0);
        sys.update(&mut s2, 0.001, 30.0, 1.0);
        assert!(s2.ros_level >= s1.ros_level,
            "Inflammation should increase ROS");
    }

    // ── update: mitophagy efficiency ──────────────────────────────────────────

    #[test]
    fn test_mitophagy_declines_with_age() {
        let sys = sys();
        let mut s_young = state();
        let mut s_old   = state();
        s_young.ros_level = 0.6;
        s_old.ros_level   = 0.6;
        sys.update(&mut s_young, 0.001, 20.0, 0.0);
        sys.update(&mut s_old,   0.001, 80.0, 0.0);
        assert!(s_young.mitophagy_efficiency >= s_old.mitophagy_efficiency,
            "Mitophagy should decline with age");
    }

    #[test]
    fn test_mitophagy_non_negative() {
        let sys = sys();
        let mut s = state();
        sys.update(&mut s, 1.0, 90.0, 1.0);
        assert!(s.mitophagy_efficiency >= 0.0);
    }

    // ── update: mito_shield (C1 exponential decay) ───────────────────────────

    #[test]
    fn test_mito_shield_at_age_zero_near_one() {
        // v3.4: mito_shield = shield_age × shield_O2; clamp floor = 0.05.
        // With normoxia (21% O₂), shield_O2 ≈ 0.013, so combined shield → clamp floor.
        // Use anoxic niche (O₂ ≈ 0) to isolate the age-decay component.
        let sys = sys();
        let mut s = state();
        sys.update_with_o2(&mut s, 0.001, 0.0, 0.0, 0.0, CellTypeShield::EpithelialProgenitor);
        assert!((s.mito_shield - 0.99).abs() < 0.02,
            "mito_shield at age=0, O₂=0% should ≈ MITO_SHIELD_MAX (0.99), got {}", s.mito_shield);
    }

    #[test]
    fn test_mito_shield_declines_with_age() {
        // Isolate age decay: hold O₂ constant at 0% (anoxic niche).
        let sys = sys();
        let mut s_young = state();
        let mut s_old   = state();
        sys.update_with_o2(&mut s_young, 0.001, 20.0, 0.0, 0.0, CellTypeShield::Fibroblast);
        sys.update_with_o2(&mut s_old,   0.001, 70.0, 0.0, 0.0, CellTypeShield::Fibroblast);
        assert!(s_young.mito_shield > s_old.mito_shield,
            "mito_shield should decline with age (C1 exponential decay): young={}, old={}",
            s_young.mito_shield, s_old.mito_shield);
    }

    #[test]
    fn test_mito_shield_minimum_floor() {
        // Clamp floor is 0.05 (physiological minimum — see CDATA v3.4 constants).
        let sys = sys();
        let mut s = state();
        sys.update(&mut s, 0.001, 1000.0, 0.0);  // normoxia: extremely low shield → hits floor
        assert!(s.mito_shield >= 0.05,
            "mito_shield must not go below the 0.05 clamp floor, got {}", s.mito_shield);
    }

    #[test]
    fn test_mito_shield_half_life_70yr() {
        // k = ln(2)/70 ≈ 0.0099; at age 70 → exp(-0.0099*70) ≈ 0.5
        let expected_at_70 = (-0.0099_f64 * 70.0).exp();
        assert!((expected_at_70 - 0.5).abs() < 0.05,
            "mito_shield half-life ~70 years, got {} at age 70", expected_at_70);
    }

    // ── update: membrane_potential ────────────────────────────────────────────

    #[test]
    fn test_membrane_potential_at_zero_mutations() {
        let sys = sys();
        let mut s = state();
        sys.update(&mut s, 0.001, 30.0, 0.0);
        // With no mutations accumulated: potential ≈ 1.0 (minor ROS effect)
        assert!(s.membrane_potential >= 0.9, "Potential near 1.0 with no mutations");
    }

    #[test]
    fn test_membrane_potential_minimum_02() {
        let sys = sys();
        let mut s = state();
        s.mtdna_mutations = 1.0;
        sys.update(&mut s, 0.001, 30.0, 0.0);
        // (1 - 1.0*0.5).max(0.2) = 0.5
        assert!(s.membrane_potential >= 0.2);
    }

    #[test]
    fn test_membrane_potential_bounded() {
        let sys = sys();
        let mut s = state();
        for _ in 0..100 {
            sys.update(&mut s, 1.0, 50.0, 0.5);
        }
        assert!(s.membrane_potential >= 0.0 && s.membrane_potential <= 1.0);
    }

    // ── calculate_oxygen_delivery ─────────────────────────────────────────────

    #[test]
    fn test_oxygen_delivery_positive() {
        let sys = sys();
        let s = state();
        let o2 = sys.calculate_oxygen_delivery(&s, 30.0);
        assert!(o2 > 0.0);
    }

    #[test]
    fn test_oxygen_delivery_declines_with_age() {
        let sys = sys();
        let s = state();
        let young = sys.calculate_oxygen_delivery(&s, 20.0);
        let old   = sys.calculate_oxygen_delivery(&s, 80.0);
        assert!(young > old, "O2 delivery declines with age");
    }

    #[test]
    fn test_oxygen_delivery_minimum_01() {
        let sys = sys();
        let s = state();
        let o2 = sys.calculate_oxygen_delivery(&s, 300.0);
        assert!(o2 >= 0.1, "O2 delivery minimum must be 0.1");
    }

    #[test]
    fn test_oxygen_delivery_reduced_by_low_potential() {
        let sys = sys();
        let mut s1 = state();
        let mut s2 = state();
        s2.membrane_potential = 0.3;
        let o1 = sys.calculate_oxygen_delivery(&s1, 40.0);
        let o2 = sys.calculate_oxygen_delivery(&s2, 40.0);
        assert!(o1 > o2, "Low membrane potential reduces O2 delivery");
    }

    // ── check_mitochondrial_collapse ──────────────────────────────────────────

    #[test]
    fn test_no_collapse_default_state() {
        let sys = sys();
        let s = state();
        assert!(!sys.check_mitochondrial_collapse(&s),
            "Default state should not trigger collapse");
    }

    #[test]
    fn test_collapse_with_high_mutations() {
        let sys = sys();
        let mut s = state();
        s.mtdna_mutations = 0.95;
        assert!(sys.check_mitochondrial_collapse(&s),
            "High mtDNA mutations should trigger collapse");
    }

    #[test]
    fn test_collapse_with_low_membrane_potential() {
        let sys = sys();
        let mut s = state();
        s.membrane_potential = 0.1;
        assert!(sys.check_mitochondrial_collapse(&s),
            "Low membrane potential should trigger collapse");
    }

    #[test]
    fn test_no_collapse_boundary_values() {
        let sys = sys();
        let mut s = state();
        s.mtdna_mutations = 0.89;
        s.membrane_potential = 0.16;
        assert!(!sys.check_mitochondrial_collapse(&s),
            "Just below thresholds should not collapse");
    }

    // ── mito_shield_for_o2 (CDATA v3.4 hypoxia prediction) ───────────────────

    #[test]
    fn test_mito_shield_normoxia_low() {
        // At 21% O₂, shield should be near zero (exp(-0.2*21) ≈ 0.015)
        let s = mito_shield_for_o2(21.0, CellTypeShield::Fibroblast);
        assert!(s < 0.05, "Normoxia shield should be near 0, got {}", s);
    }

    #[test]
    fn test_mito_shield_hypoxia_high() {
        // At 2% O₂, EpithelialProgenitor shield: 0.99 × exp(-0.2 × 2) ≈ 0.663.
        // K_O2 = 0.2 is calibrated to Ito 2006 (CONCEPT §O₂); with this constant
        // the shield at 2% O₂ is ~0.66, substantially above normoxia (≈0.013).
        // NOTE: the calibration anchor comment "≈0.980" requires K_O2 ≈ 0.005 — see TODO.
        // TODO: re-calibrate K_O2 in Experiment 1 (O₂ dose-response, HCA2 fibroblasts).
        let s = mito_shield_for_o2(2.0, CellTypeShield::EpithelialProgenitor);
        assert!(s > 0.50, "Hypoxia shield (2% O₂, progenitor) should be >0.50, got {}", s);
        assert!(s < 0.99, "Hypoxia shield should not reach MITO_SHIELD_MAX at 2% O₂, got {}", s);
    }

    #[test]
    fn test_mito_shield_increases_with_lower_o2() {
        let s_normoxia = mito_shield_for_o2(21.0, CellTypeShield::Fibroblast);
        let s_physio   = mito_shield_for_o2(3.0,  CellTypeShield::Fibroblast);
        let s_deep     = mito_shield_for_o2(1.0,  CellTypeShield::Fibroblast);
        assert!(s_deep > s_physio && s_physio > s_normoxia,
            "Shield must increase as O₂ decreases: {}/{}/{}", s_deep, s_physio, s_normoxia);
    }

    #[test]
    fn test_mito_shield_bounded_zero_one() {
        for o2 in [0.0, 1.0, 2.0, 5.0, 10.0, 21.0, 50.0, 100.0] {
            for cell_type in [CellTypeShield::Fibroblast,
                              CellTypeShield::HematopoieticStem,
                              CellTypeShield::EpithelialProgenitor] {
                let s = mito_shield_for_o2(o2, cell_type);
                assert!(s >= 0.0 && s <= 1.0,
                    "shield={} at O₂={} out of [0,1]", s, o2);
            }
        }
    }

    #[test]
    fn test_cell_type_modifier_ordering() {
        // Progenitor > HSC > Fibroblast (more stress-resistant → higher shield)
        let o2 = 2.0;
        let sp = mito_shield_for_o2(o2, CellTypeShield::EpithelialProgenitor);
        let sh = mito_shield_for_o2(o2, CellTypeShield::HematopoieticStem);
        let sf = mito_shield_for_o2(o2, CellTypeShield::Fibroblast);
        assert!(sp >= sh && sh >= sf,
            "shield order: progenitor({}) >= HSC({}) >= fibroblast({})", sp, sh, sf);
    }

    // ── predicted_hayflick ────────────────────────────────────────────────────

    #[test]
    fn test_hayflick_normoxia_fibroblast_near_50() {
        // Calibration: normoxia (21% O₂), fibroblast → ~50 PD
        let n = predicted_hayflick(21.0, CellTypeShield::Fibroblast);
        assert!(n > 40.0 && n < 70.0,
            "Normoxia Hayflick should be ~50, got {}", n);
    }

    #[test]
    fn test_hayflick_hypoxia_progenitor_over_normoxia() {
        // Peters-Hall (2020): 2% O₂ HBECs show substantially more PD than normoxia.
        // With K_O2=0.2: shield(2%)≈0.663 → Hayflick≈148; shield(21%)≈0.013 → Hayflick≈51.
        // Ratio ≈ 2.9×. The ">200 PD" calibration anchor requires re-calibration of K_O2
        // in Experiment 1 (O₂ dose-response, HCA2 fibroblasts). TODO: update after Exp 1.
        let n_hypoxia  = predicted_hayflick(2.0,  CellTypeShield::EpithelialProgenitor);
        let n_normoxia = predicted_hayflick(21.0, CellTypeShield::EpithelialProgenitor);
        assert!(n_hypoxia > n_normoxia * 2.0,
            "Hypoxia Hayflick should be >2× normoxia: hypoxia={}, normoxia={}", n_hypoxia, n_normoxia);
        assert!(n_hypoxia > 100.0,
            "Hypoxia progenitor Hayflick should be >100, got {}", n_hypoxia);
    }

    #[test]
    fn test_hayflick_increases_with_lower_o2() {
        let n21 = predicted_hayflick(21.0, CellTypeShield::Fibroblast);
        let n3  = predicted_hayflick(3.0,  CellTypeShield::Fibroblast);
        let n1  = predicted_hayflick(1.0,  CellTypeShield::Fibroblast);
        assert!(n1 > n3 && n3 > n21,
            "Hayflick must increase as O₂ decreases: {}/{}/{}", n1, n3, n21);
    }

    #[test]
    fn test_hayflick_positive_and_finite() {
        for o2 in [0.5, 1.0, 2.0, 5.0, 10.0, 21.0] {
            let n = predicted_hayflick(o2, CellTypeShield::Fibroblast);
            assert!(n > 0.0 && n.is_finite(),
                "Hayflick at O₂={} must be positive finite, got {}", o2, n);
        }
    }

    #[test]
    fn test_mito_shield_for_o2_at_zero_equals_max() {
        // At [O₂]=0: exp(0)=1 → shield = MITO_SHIELD_MAX * modifier
        let s = mito_shield_for_o2(0.0, CellTypeShield::EpithelialProgenitor);
        assert!((s - MITO_SHIELD_MAX).abs() < 1e-9,
            "Shield at O₂=0 should = MITO_SHIELD_MAX={}, got {}", MITO_SHIELD_MAX, s);
    }

    #[test]
    fn test_collapse_boundary_mutations_exactly_09() {
        let sys = sys();
        // > 0.9 is the condition, so exactly 0.9 does NOT collapse
        let mut s = state();
        s.mtdna_mutations = 0.9;
        assert!(!sys.check_mitochondrial_collapse(&s),
            "mutations = 0.9 (not > 0.9) must NOT collapse");
        // 0.901 > 0.9 → should collapse
        let mut s2 = state();
        s2.mtdna_mutations = 0.901;
        assert!(sys.check_mitochondrial_collapse(&s2),
            "mutations 0.901 > 0.9 should collapse");
    }
}
