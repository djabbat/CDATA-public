//! ЭПС-стресс и UPR (уровень -1: органоиды).
//!
//! Снижение протеасомной активности + агрегаты → ЭПС перегружен
//! неправильно свёрнутыми белками → UPR активируется → Ca²⁺ выходит.
//! Хронический UPR (> 0.7) → CHOP → апоптоз.
//!
//! # Формулы
//!
//! ```text
//! upr_input = aggregates × 0.60 + (1 − proteasome_activity) × 0.40
//! d(upr)/dt = (upr_input − upr × repair_rate) × dt
//! ca2_buffer = (1 − upr × 0.60).clamp(0.10, 1.0)
//! chaperone_saturation = upr × 0.90
//! ```

use cell_dt_core::components::ERStressState;

/// Параметры ER-стресса.
#[derive(Debug, Clone)]
pub struct ERStressParams {
    /// Вклад агрегатов в UPR [0..1].
    pub aggregates_upr_weight:  f32,
    /// Вклад снижения протеасомы в UPR.
    pub proteasome_upr_weight:  f32,
    /// Скорость разрешения UPR при адаптации (BiP-экспансия) [/год].
    pub upr_repair_rate:        f32,
}

impl Default for ERStressParams {
    fn default() -> Self {
        Self {
            aggregates_upr_weight: 0.60,
            proteasome_upr_weight: 0.40,
            upr_repair_rate:       0.50,
        }
    }
}

/// Обновить ERStressState на один шаг.
///
/// # Аргументы
/// * `er`                  — состояние (in/out).
/// * `aggregates`          — уровень агрегатов в клетке [0..1].
/// * `proteasome_activity` — нормированная активность протеасомы [0..1].
/// * `params`              — параметры.
/// * `dt_years`            — шаг (лет).
pub fn update_er_stress_state(
    er:                  &mut ERStressState,
    aggregates:          f32,
    proteasome_activity: f32,
    params:              &ERStressParams,
    dt_years:            f32,
) {
    let upr_input = aggregates * params.aggregates_upr_weight
        + (1.0 - proteasome_activity) * params.proteasome_upr_weight;

    let dupr = upr_input - er.unfolded_protein_response * params.upr_repair_rate;
    er.unfolded_protein_response =
        (er.unfolded_protein_response + dupr * dt_years).clamp(0.0, 1.0);

    er.chaperone_saturation =
        (er.unfolded_protein_response * 0.90).clamp(0.0, 1.0);

    er.update_derived(); // ca2_buffer_capacity
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> ERStressParams { ERStressParams::default() }

    #[test]
    fn test_pristine_no_stress() {
        let s = ERStressState::pristine();
        assert!((s.unfolded_protein_response - 0.0).abs() < 1e-5);
        assert!((s.ca2_buffer_capacity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_aggregates_trigger_upr() {
        let mut s = ERStressState::pristine();
        update_er_stress_state(&mut s, 0.8, 1.0, &p(), 1.0);
        assert!(s.unfolded_protein_response > 0.0,
            "Агрегаты → UPR активируется");
    }

    #[test]
    fn test_low_proteasome_triggers_upr() {
        let mut s = ERStressState::pristine();
        update_er_stress_state(&mut s, 0.0, 0.2, &p(), 1.0);
        assert!(s.unfolded_protein_response > 0.0,
            "Протеасома↓ → UPR↑");
    }

    #[test]
    fn test_high_upr_reduces_ca2_buffer() {
        let mut s = ERStressState::pristine();
        s.unfolded_protein_response = 0.8;
        s.update_derived();
        assert!(s.ca2_buffer_capacity < 0.55,
            "Высокий UPR → Ca²⁺ буфер истощается");
    }

    #[test]
    fn test_repair_resolves_upr_without_stress() {
        let mut s = ERStressState::pristine();
        s.unfolded_protein_response = 0.5;
        // Без агрегатов + полная протеасома → UPR снижается
        update_er_stress_state(&mut s, 0.0, 1.0, &p(), 2.0);
        assert!(s.unfolded_protein_response < 0.5,
            "Без стресса + репарация → UPR снижается");
    }

    #[test]
    fn test_chaperone_saturation_proportional_to_upr() {
        let mut s = ERStressState::pristine();
        update_er_stress_state(&mut s, 0.5, 0.5, &p(), 1.0);
        let expected = (s.unfolded_protein_response * 0.90).min(1.0);
        assert!((s.chaperone_saturation - expected).abs() < 1e-4,
            "chaperone_saturation = upr × 0.90");
    }
}
