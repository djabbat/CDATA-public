//! Фиброз ниши (уровень +1: ткань).
//!
//! SASP → TGF-β → активация миофибробластов → отложение коллагена →
//! замещение паренхимы → functional_capacity↓.
//!
//! # Формулы
//!
//! ```text
//! fibroblast_activation = (sasp_intensity × 0.70 + existing_fibrosis × 0.30).clamp(0,1)
//! d(functional_replacement)/dt = collagen_deposition_rate × dt
//! fc_penalty = functional_replacement × 0.80
//! ```

use cell_dt_core::components::FibrosisState;

/// Параметры фиброза.
#[derive(Debug, Clone)]
pub struct FibrosisParams {
    /// Вес SASP в активации миофибробластов.
    pub sasp_weight:        f32,
    /// Положительная обратная связь фиброза (TGF-β из матрикса).
    pub fibrosis_feedback:  f32,
}

impl Default for FibrosisParams {
    fn default() -> Self {
        Self {
            sasp_weight:       0.70,
            fibrosis_feedback: 0.30,
        }
    }
}

/// Обновить FibrosisState на один шаг.
pub fn update_fibrosis_state(
    fib:            &mut FibrosisState,
    sasp_intensity: f32,
    params:         &FibrosisParams,
    dt_years:       f32,
) {
    fib.fibroblast_activation =
        (sasp_intensity * params.sasp_weight
         + fib.functional_replacement * params.fibrosis_feedback)
            .clamp(0.0, 1.0);

    fib.update_derived(); // collagen_deposition_rate

    // Фиброз накапливается необратимо (integral)
    fib.functional_replacement =
        (fib.functional_replacement + fib.collagen_deposition_rate * dt_years)
            .clamp(0.0, 1.0);
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> FibrosisParams { FibrosisParams::default() }

    #[test]
    fn test_pristine_no_fibrosis() {
        let s = FibrosisState::pristine();
        assert!((s.functional_replacement - 0.0).abs() < 1e-5);
        assert!((s.fc_penalty() - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_sasp_triggers_fibrosis() {
        let mut s = FibrosisState::pristine();
        update_fibrosis_state(&mut s, 0.8, &p(), 1.0);
        assert!(s.fibroblast_activation > 0.0,
            "SASP → миофибробласты активируются");
    }

    #[test]
    fn test_fibrosis_accumulates_over_time() {
        let mut s = FibrosisState::pristine();
        for _ in 0..10 {
            update_fibrosis_state(&mut s, 0.6, &p(), 1.0);
        }
        assert!(s.functional_replacement > 0.0,
            "Фиброз накапливается со временем");
    }

    #[test]
    fn test_fc_penalty_proportional_to_replacement() {
        let mut s = FibrosisState::pristine();
        s.functional_replacement = 0.5;
        assert!((s.fc_penalty() - 0.5 * 0.8).abs() < 1e-4,
            "fc_penalty = replacement × 0.80");
    }

    #[test]
    fn test_fibrosis_self_amplification() {
        let mut s1 = FibrosisState::pristine();
        let mut s2 = FibrosisState::pristine();
        s2.functional_replacement = 0.3; // уже есть фиброз
        update_fibrosis_state(&mut s1, 0.5, &p(), 1.0);
        update_fibrosis_state(&mut s2, 0.5, &p(), 1.0);
        assert!(s2.fibroblast_activation > s1.fibroblast_activation,
            "Существующий фиброз → положительная обратная связь TGF-β");
    }

    #[test]
    fn test_no_fibrosis_without_sasp() {
        let mut s = FibrosisState::pristine();
        update_fibrosis_state(&mut s, 0.0, &p(), 10.0);
        assert!((s.fibroblast_activation - 0.0).abs() < 1e-4,
            "Без SASP → фиброза нет");
    }
}
