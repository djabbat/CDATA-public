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
        // DAMPs
        let damps_prod = self.params.damps_rate * (state.senescent_cell_fraction + dna_damage * 0.5);
        state.damps_level = (state.damps_level + damps_prod * dt - 0.1 * state.damps_level * dt).clamp(0.0, 1.0);

        // cGAS-STING
        state.cgas_sting_activity = (state.damps_level * self.params.cgas_sensitivity + mtdna_release * 0.5).min(1.0);

        // NF-κB: dynamic activation (FIXED Round 6: was static 0.1 → SASP was always zero)
        // Activated by cGAS-STING signal, DAMPs, and SASP positive feedback loop
        let nfkb_input = state.cgas_sting_activity * 0.6
            + state.sasp_level * 0.3
            + state.damps_level * 0.1;
        state.nfkb_activity = (0.05 + nfkb_input * 0.9).clamp(0.05, 1.0);

        // SASP production: requires cGAS-STING × NF-κB × senescent cell burden
        let sasp_prod = state.cgas_sting_activity * state.nfkb_activity * state.senescent_cell_fraction;
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
