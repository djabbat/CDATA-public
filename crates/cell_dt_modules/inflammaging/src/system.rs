use cell_dt_core::InflammagingState;
use crate::params::InflammagingParams;

pub struct InflammagingSystem {
    pub params: InflammagingParams,
}

impl InflammagingSystem {
    pub fn new() -> Self {
        Self { params: InflammagingParams::default() }
    }

    pub fn update(
        &self,
        state: &mut InflammagingState,
        dt: f64,
        age_years: f64,
        dna_damage: f64,
        mtdna_release: f64,
    ) {
        self.update_with_chip(state, dt, age_years, dna_damage, mtdna_release, 0.0);
    }

    /// Full update with explicit CHIP VAF (CDATA v3.5, L1 link).
    ///
    /// `chip_vaf`: CHIP variant allele frequency [0.0, 1.0].
    ///   At age 70: typical VAF ≈ 0.07 (Jaiswal SS et al. 2017 NEJM 377(2):111-121, PMID: 28636844).
    ///   CHIP amplifies SASP: sasp_prod *= (1 + chip_vaf × chip_sasp_strength)
    ///   Prior on chip_sasp_strength: Normal(0.5, 0.15) — Wu et al. 2023.
    pub fn update_with_chip(
        &self,
        state: &mut InflammagingState,
        dt: f64,
        age_years: f64,
        dna_damage: f64,
        mtdna_release: f64,
        chip_vaf: f64,
    ) {
        // DAMPs
        let damps_prod = self.params.damps_rate * (state.senescent_cell_fraction + dna_damage * 0.5);
        // FIX C4: use named damps_decay_rate instead of hardcoded 0.1
        state.damps_level = (state.damps_level + damps_prod * dt - self.params.damps_decay_rate * state.damps_level * dt).clamp(0.0, 1.0);

        // cGAS-STING
        state.cgas_sting_activity = (state.damps_level * self.params.cgas_sensitivity + mtdna_release * 0.5).min(1.0);

        // NF-κB: dynamic activation
        // FIX Round 7 (B2): removed spurious *0.9; weights sum to 1.0; clamp to 0.95
        let nfkb_input = state.cgas_sting_activity * 0.6
            + state.sasp_level * 0.3
            + state.damps_level * 0.1;
        state.nfkb_activity = (0.05 + nfkb_input).clamp(0.05, 0.95);

        // SASP production with CHIP amplifier (L1 link, CDATA v3.5)
        let chip_amplifier = 1.0 + chip_vaf.clamp(0.0, 1.0) * self.params.chip_sasp_strength;
        let sasp_prod = state.cgas_sting_activity * state.nfkb_activity
            * state.senescent_cell_fraction * chip_amplifier;
        state.sasp_level = (state.sasp_level + sasp_prod * dt - self.params.sasp_decay * state.sasp_level * dt).clamp(0.0, 1.0);

        // NK эффективность
        let base_nk = (1.0 - age_years * self.params.nk_age_decay).max(0.1);
        state.nk_efficiency = (base_nk * (1.0 - state.sasp_level * 0.3)).max(0.05);

        // NK элиминация сенесцентных клеток
        let eliminated = state.nk_efficiency * 0.1 * state.senescent_cell_fraction * dt;
        state.senescent_cell_fraction = (state.senescent_cell_fraction - eliminated).max(0.0);

        // Фиброз
        state.fibrosis_level = (state.fibrosis_level + self.params.fibrosis_rate * state.sasp_level * dt).min(1.0);
    }
}

impl Default for InflammagingSystem {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::InflammagingState;

    fn default_state() -> InflammagingState {
        InflammagingState::default()
    }

    fn sys() -> InflammagingSystem {
        InflammagingSystem::new()
    }

    // ── Construction ──────────────────────────────────────────────────────────

    #[test]
    fn test_new_creates_default_params() {
        let s = sys();
        assert!((s.params.damps_rate - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_default_same_as_new() {
        let s1 = InflammagingSystem::new();
        let s2 = InflammagingSystem::default();
        assert!((s1.params.damps_rate - s2.params.damps_rate).abs() < 1e-9);
    }

    // ── update: DAMPs ─────────────────────────────────────────────────────────

    #[test]
    fn test_damps_increase_with_senescent_cells() {
        let sys = sys();
        let mut state = default_state();
        state.senescent_cell_fraction = 0.5;
        let before = state.damps_level;
        sys.update(&mut state, 1.0, 30.0, 0.0, 0.0);
        assert!(state.damps_level >= before, "DAMPs should increase with senescent cells");
    }

    #[test]
    fn test_damps_increase_with_dna_damage() {
        let sys = sys();
        let mut state = default_state();
        let before = state.damps_level;
        sys.update(&mut state, 1.0, 30.0, 0.8, 0.0);
        assert!(state.damps_level >= before, "DAMPs should respond to DNA damage");
    }

    #[test]
    fn test_damps_clamped_to_zero_one() {
        let sys = sys();
        let mut state = default_state();
        state.senescent_cell_fraction = 1.0;
        // Run for many steps
        for _ in 0..100 {
            sys.update(&mut state, 1.0, 50.0, 1.0, 1.0);
        }
        assert!(state.damps_level >= 0.0 && state.damps_level <= 1.0,
            "damps_level must stay in [0,1], got {}", state.damps_level);
    }

    #[test]
    fn test_damps_decay_without_production() {
        // senescent=0, dna_damage=0: production ≈ 0; should decay from initial
        let sys = sys();
        let mut state = default_state();
        state.damps_level = 0.8;
        sys.update(&mut state, 1.0, 30.0, 0.0, 0.0);
        assert!(state.damps_level < 0.8, "DAMPs should decay without input");
    }

    // ── update: cGAS-STING ────────────────────────────────────────────────────

    #[test]
    fn test_cgas_sting_driven_by_damps() {
        let sys = sys();
        let mut state = default_state();
        state.damps_level = 0.5;
        sys.update(&mut state, 0.01, 30.0, 0.0, 0.0);
        assert!(state.cgas_sting_activity > 0.0, "cGAS-STING should respond to DAMPs");
    }

    #[test]
    fn test_cgas_sting_driven_by_mtdna() {
        let sys = sys();
        let mut state = default_state();
        sys.update(&mut state, 0.01, 30.0, 0.0, 0.8);
        assert!(state.cgas_sting_activity > 0.0, "cGAS-STING should respond to mtDNA release");
    }

    #[test]
    fn test_cgas_sting_clamped_to_one() {
        let sys = sys();
        let mut state = default_state();
        state.damps_level = 1.0;
        sys.update(&mut state, 1.0, 30.0, 0.0, 1.0);
        assert!(state.cgas_sting_activity <= 1.0);
    }

    #[test]
    fn test_cgas_sting_non_negative() {
        let sys = sys();
        let mut state = default_state();
        sys.update(&mut state, 1.0, 30.0, 0.0, 0.0);
        assert!(state.cgas_sting_activity >= 0.0);
    }

    // ── update: NF-κB ────────────────────────────────────────────────────────

    #[test]
    fn test_nfkb_minimum_basal() {
        let sys = sys();
        let mut state = default_state();
        sys.update(&mut state, 0.001, 0.0, 0.0, 0.0);
        assert!(state.nfkb_activity >= 0.05,
            "NF-kB must have at least basal 0.05 activity");
    }

    #[test]
    fn test_nfkb_maximum_capped_095() {
        let sys = sys();
        let mut state = default_state();
        state.cgas_sting_activity = 1.0;
        state.sasp_level = 1.0;
        state.damps_level = 1.0;
        sys.update(&mut state, 0.001, 30.0, 0.0, 0.0);
        assert!(state.nfkb_activity <= 0.95,
            "NF-kB must be capped at 0.95, got {}", state.nfkb_activity);
    }

    #[test]
    fn test_nfkb_increases_with_cgas() {
        let sys = sys();
        let mut s1 = default_state();
        let mut s2 = default_state();
        s2.cgas_sting_activity = 0.8;
        sys.update(&mut s1, 0.001, 30.0, 0.0, 0.0);
        sys.update(&mut s2, 0.001, 30.0, 0.0, 0.0);
        assert!(s2.nfkb_activity >= s1.nfkb_activity,
            "Higher cGAS-STING should increase NF-kB");
    }

    // ── update: SASP ─────────────────────────────────────────────────────────

    #[test]
    fn test_sasp_requires_senescent_cells() {
        let sys = sys();
        let mut s1 = default_state();
        let mut s2 = default_state();
        s2.senescent_cell_fraction = 0.5;
        s2.cgas_sting_activity = 0.8;
        s2.nfkb_activity = 0.8;
        sys.update(&mut s1, 1.0, 30.0, 0.0, 0.0);
        sys.update(&mut s2, 1.0, 30.0, 0.0, 0.0);
        assert!(s2.sasp_level >= s1.sasp_level, "SASP needs senescent cells");
    }

    #[test]
    fn test_sasp_clamped_zero_one() {
        let sys = sys();
        let mut state = default_state();
        state.senescent_cell_fraction = 1.0;
        state.cgas_sting_activity = 1.0;
        state.nfkb_activity = 0.95;
        for _ in 0..200 {
            sys.update(&mut state, 1.0, 30.0, 0.0, 0.0);
        }
        assert!(state.sasp_level >= 0.0 && state.sasp_level <= 1.0);
    }

    #[test]
    fn test_sasp_decays_without_senescent_cells() {
        let sys = sys();
        let mut state = default_state();
        state.sasp_level = 0.5;
        state.senescent_cell_fraction = 0.0;
        state.cgas_sting_activity = 0.0;
        state.nfkb_activity = 0.05;
        sys.update(&mut state, 1.0, 30.0, 0.0, 0.0);
        assert!(state.sasp_level < 0.5, "SASP should decay without senescent cells");
    }

    // ── update: NK efficiency ─────────────────────────────────────────────────

    #[test]
    fn test_nk_declines_with_age() {
        let sys = sys();
        let mut s_young = default_state();
        let mut s_old = default_state();
        sys.update(&mut s_young, 0.001, 20.0, 0.0, 0.0);
        sys.update(&mut s_old,   0.001, 80.0, 0.0, 0.0);
        assert!(s_young.nk_efficiency >= s_old.nk_efficiency,
            "NK efficiency should decline with age");
    }

    #[test]
    fn test_nk_minimum_bound() {
        let sys = sys();
        let mut state = default_state();
        state.sasp_level = 1.0;
        sys.update(&mut state, 1.0, 200.0, 0.0, 0.0);
        assert!(state.nk_efficiency >= 0.05,
            "NK efficiency must have minimum 0.05, got {}", state.nk_efficiency);
    }

    #[test]
    fn test_nk_reduced_by_high_sasp() {
        let sys = sys();
        let mut s1 = default_state();
        let mut s2 = default_state();
        s2.sasp_level = 0.9;
        sys.update(&mut s1, 0.001, 30.0, 0.0, 0.0);
        sys.update(&mut s2, 0.001, 30.0, 0.0, 0.0);
        assert!(s1.nk_efficiency >= s2.nk_efficiency,
            "High SASP reduces NK efficiency");
    }

    // ── update: fibrosis ─────────────────────────────────────────────────────

    #[test]
    fn test_fibrosis_increases_with_sasp() {
        let sys = sys();
        let mut state = default_state();
        state.sasp_level = 0.8;
        let before = state.fibrosis_level;
        sys.update(&mut state, 1.0, 30.0, 0.0, 0.0);
        assert!(state.fibrosis_level >= before,
            "Fibrosis should increase when SASP is elevated");
    }

    #[test]
    fn test_fibrosis_no_growth_without_sasp() {
        let sys = sys();
        let mut state = default_state();
        state.sasp_level = 0.0;
        let before = state.fibrosis_level;
        sys.update(&mut state, 1.0, 30.0, 0.0, 0.0);
        assert!((state.fibrosis_level - before).abs() < 1e-9,
            "No SASP → no fibrosis growth");
    }

    #[test]
    fn test_fibrosis_clamped_to_one() {
        let sys = sys();
        let mut state = default_state();
        state.fibrosis_level = 0.99;
        state.sasp_level = 1.0;
        for _ in 0..100 {
            sys.update(&mut state, 1.0, 30.0, 1.0, 1.0);
        }
        assert!(state.fibrosis_level <= 1.0,
            "Fibrosis must not exceed 1.0, got {}", state.fibrosis_level);
    }

    // ── update: NK eliminates senescent cells ──────────────────────────────────

    #[test]
    fn test_senescent_cells_eliminated_by_nk() {
        let sys = sys();
        let mut state = default_state();
        state.senescent_cell_fraction = 0.5;
        state.nk_efficiency = 0.9;
        let before = state.senescent_cell_fraction;
        sys.update(&mut state, 1.0, 30.0, 0.0, 0.0);
        assert!(state.senescent_cell_fraction <= before,
            "NK should eliminate senescent cells");
    }

    #[test]
    fn test_senescent_fraction_non_negative() {
        let sys = sys();
        let mut state = default_state();
        state.senescent_cell_fraction = 0.001;
        state.nk_efficiency = 1.0;
        for _ in 0..100 {
            sys.update(&mut state, 1.0, 30.0, 0.0, 0.0);
        }
        assert!(state.senescent_cell_fraction >= 0.0);
    }

    // ── Biological constraints after update ───────────────────────────────────

    #[test]
    fn test_all_state_vars_in_range_after_update() {
        let sys = sys();
        let mut state = default_state();
        state.senescent_cell_fraction = 0.3;
        sys.update(&mut state, 1.0, 50.0, 0.3, 0.1);
        assert!(state.damps_level >= 0.0 && state.damps_level <= 1.0);
        assert!(state.cgas_sting_activity >= 0.0 && state.cgas_sting_activity <= 1.0);
        assert!(state.nfkb_activity >= 0.05 && state.nfkb_activity <= 0.95);
        assert!(state.sasp_level >= 0.0 && state.sasp_level <= 1.0);
        assert!(state.nk_efficiency >= 0.05);
        assert!(state.fibrosis_level >= 0.0 && state.fibrosis_level <= 1.0);
    }
}
