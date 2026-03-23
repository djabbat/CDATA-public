//! Актиновое кольцо цитокинеза (уровень -2: цитоскелет).
//!
//! ROS → окисление Cys374 β-актина → деполимеризация.
//! Неполный цитокинез → бинуклеарность → анеуплоидия → PLK4-дисрегуляция↑.
//!
//! # Формулы
//!
//! ```text
//! actin_polymerization_rate = (1 − oh_radical × 0.50).clamp(0, 1)
//! contractile_ring_integrity = actin_polymerization_rate × (1 − phospho × 0.30)
//! incomplete_cytokinesis_prob = (1 − ring_integrity) × 0.30
//! ```

use cell_dt_core::components::ActinRingState;

/// Параметры актин-кольца.
#[derive(Debug, Clone)]
pub struct ActinRingParams {
    /// Чувствительность актина к OH·-радикалу [0..1].
    pub oh_actin_sensitivity:    f32,
    /// Вклад phospho_dysreg в нарушение кольца (ROCK/RHOA путь).
    pub phospho_ring_sensitivity: f32,
}

impl Default for ActinRingParams {
    fn default() -> Self {
        Self {
            oh_actin_sensitivity:     0.50,
            phospho_ring_sensitivity: 0.30,
        }
    }
}

/// Обновить ActinRingState на один шаг.
///
/// # Аргументы
/// * `actin`          — состояние (in/out).
/// * `oh_radical`     — уровень OH·-радикала [0..1].
/// * `phospho_dysreg` — дисрегуляция фосфорилирования (PLK4/ROCK) [0..1].
/// * `params`         — параметры.
pub fn update_actin_ring_state(
    actin:         &mut ActinRingState,
    oh_radical:    f32,
    phospho_dysreg: f32,
    params:        &ActinRingParams,
) {
    actin.actin_polymerization_rate =
        (1.0 - oh_radical * params.oh_actin_sensitivity)
            .clamp(0.0, 1.0);

    actin.contractile_ring_integrity =
        (actin.actin_polymerization_rate * (1.0 - phospho_dysreg * params.phospho_ring_sensitivity))
            .clamp(0.0, 1.0);

    actin.update_derived();
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> ActinRingParams { ActinRingParams::default() }

    #[test]
    fn test_pristine_intact_ring() {
        let s = ActinRingState::pristine();
        assert!((s.contractile_ring_integrity - 1.0).abs() < 1e-5);
        assert!((s.incomplete_cytokinesis_prob - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_oh_radical_depolymerizes_actin() {
        let mut s = ActinRingState::pristine();
        update_actin_ring_state(&mut s, 1.0, 0.0, &p());
        assert!(s.actin_polymerization_rate < 0.55,
            "OH· → актин деполимеризуется");
    }

    #[test]
    fn test_phospho_damages_ring() {
        let mut s = ActinRingState::pristine();
        update_actin_ring_state(&mut s, 0.0, 1.0, &p());
        assert!(s.contractile_ring_integrity < 0.75,
            "phospho_dysreg → кольцо нарушается");
    }

    #[test]
    fn test_incomplete_cytokinesis_prob_nonzero_when_ring_damaged() {
        let mut s = ActinRingState::pristine();
        update_actin_ring_state(&mut s, 0.8, 0.8, &p());
        assert!(s.incomplete_cytokinesis_prob > 0.0,
            "Повреждённое кольцо → риск неполного цитокинеза");
    }

    #[test]
    fn test_combined_damage_additive() {
        let mut s = ActinRingState::pristine();
        update_actin_ring_state(&mut s, 0.5, 0.5, &p());
        // ring = (1 - 0.5×0.5) × (1 - 0.5×0.3) = 0.75 × 0.85 = 0.6375
        let expected_ring = (1.0_f32 - 0.5 * 0.5) * (1.0 - 0.5 * 0.3);
        assert!((s.contractile_ring_integrity - expected_ring).abs() < 1e-4,
            "Суммарный эффект ROS + phospho");
    }
}
