//! ZeHealth — связь CAII с Ze Vector Theory (уровень -5).
//!
//! Обновляет ZeHealthState из текущего CAII и энтропийной оценки из ThermodynamicState.
//!
//! # Биологический смысл
//!
//! Ze Vector Theory (Tkemaladze): каждый биологический процесс имеет
//! критическую «скорость» v* = 0.456 (T/S-квантовая точка). Отклонение
//! от v* характеризует «биологический возраст» клетки.
//!
//! Связь с CDATA: CAII (Centriolar Appendage Integrity Index) — прямой
//! структурный биомаркер, отражающий потерю функции придатков центриоли.
//! При CAII=1.0 (молодая клетка): v = v* = 0.456 (оптимум).
//! При CAII→0 (старая клетка): v → 1.0 (коллапс).
//!
//! # Формула
//!
//! ```text
//! v = v* + (1 − v*) × (1 − CAII)
//!   = 0.456 + 0.544 × (1 − CAII)
//! ```

use cell_dt_core::components::ZeHealthState;

// ─────────────────────────────────────────────────────────────────────────────
// Основная функция обновления
// ─────────────────────────────────────────────────────────────────────────────

/// Обновить ZeHealthState из текущего CAII и опционально из энтропийной оценки.
///
/// Вся логика преобразования CAII → v реализована в `ZeHealthState::update()`.
/// Эта функция — тонкая обёртка для единообразия с остальными модулями.
///
/// # Аргументы
/// * `ze`        — изменяемый ZeHealthState.
/// * `caii`      — текущий CAII из AppendageProteinState [0..1].
/// * `entropy_v` — термодинамическая оценка v (ze_velocity_analog из ThermodynamicState).
///                 При отсутствии ThermodynamicState: `None` → v_consensus = v.
pub fn update_ze_health_state(
    ze: &mut ZeHealthState,
    caii: f32,
    entropy_v: Option<f32>,
) {
    ze.update(caii, entropy_v);
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// CAII=1.0 → v = v* (нет отклонения, оптимальное здоровье)
    #[test]
    fn test_pristine_caii_gives_v_optimal() {
        let mut ze = ZeHealthState::default();
        update_ze_health_state(&mut ze, 1.0, None);
        assert!((ze.v - ZeHealthState::V_OPTIMAL).abs() < 1e-5,
            "CAII=1.0 → v должно быть v*={}: {:.6}", ZeHealthState::V_OPTIMAL, ze.v);
        assert!(ze.deviation_from_optimal < 1e-5,
            "Отклонение при CAII=1.0: {:.6}", ze.deviation_from_optimal);
        assert!((ze.ze_health_index - 1.0).abs() < 1e-5);
    }

    /// CAII=0.0 → v = 1.0 (максимальное отклонение, коллапс)
    #[test]
    fn test_zero_caii_gives_v_max() {
        let mut ze = ZeHealthState::default();
        update_ze_health_state(&mut ze, 0.0, None);
        assert!((ze.v - 1.0).abs() < 1e-5, "CAII=0 → v=1.0: {:.6}", ze.v);
        assert!((ze.deviation_from_optimal - ZeHealthState::MAX_DEVIATION).abs() < 1e-5,
            "Максимальное отклонение: {:.6}", ze.deviation_from_optimal);
        assert!(ze.ze_health_index < 1e-5);
    }

    /// CAII=0.5 → v = v* + MAX_DEV×0.5 (линейная формула)
    #[test]
    fn test_half_caii_linear() {
        let mut ze = ZeHealthState::default();
        update_ze_health_state(&mut ze, 0.5, None);
        let expected_v = ZeHealthState::V_OPTIMAL + ZeHealthState::MAX_DEVIATION * 0.5;
        assert!((ze.v - expected_v).abs() < 1e-5,
            "CAII=0.5: ожидается v={:.6}, получено {:.6}", expected_v, ze.v);
    }

    /// Энтропийная оценка обновляет v_entropy и v_consensus
    #[test]
    fn test_entropy_updates_consensus() {
        let mut ze = ZeHealthState::default();
        let entropy_v = 0.55_f32;
        update_ze_health_state(&mut ze, 1.0, Some(entropy_v));
        // v = v* = 0.456, entropy_v = 0.55
        // consensus = (0.456 + 0.55) / 2 = 0.503
        let expected_consensus = (ZeHealthState::V_OPTIMAL + entropy_v) / 2.0;
        assert!((ze.v_consensus - expected_consensus).abs() < 1e-5,
            "Ожидался консенсус {:.6}, получено {:.6}", expected_consensus, ze.v_consensus);
        assert!((ze.v_entropy - entropy_v).abs() < 1e-5);
    }

    /// Без энтропийной оценки: v_consensus = v
    #[test]
    fn test_without_entropy_consensus_equals_v() {
        let mut ze = ZeHealthState::default();
        update_ze_health_state(&mut ze, 0.7, None);
        assert!((ze.v_consensus - ze.v).abs() < 1e-5,
            "Без энтропии: consensus={:.6} должно быть = v={:.6}",
            ze.v_consensus, ze.v);
    }

    /// Интерпретационные пороги: optimal / mild / moderate / severe / near_collapse
    #[test]
    fn test_interpretation_thresholds() {
        let mut ze = ZeHealthState::default();

        // deviation < 0.05 → optimal
        // CAII=0.95: v = 0.456 + 0.544×0.05 = 0.456 + 0.027 = 0.483, dev=0.027
        update_ze_health_state(&mut ze, 0.95, None);
        assert_eq!(ze.interpretation(), "optimal",
            "CAII=0.95, dev={:.4} → ожидается 'optimal'", ze.deviation_from_optimal);

        // deviation 0.05–0.15 → mild_aging
        // CAII=0.85: v = 0.456 + 0.544×0.15 = 0.456 + 0.082 = 0.538, dev=0.082
        update_ze_health_state(&mut ze, 0.85, None);
        assert_eq!(ze.interpretation(), "mild_aging",
            "CAII=0.85, dev={:.4} → ожидается 'mild_aging'", ze.deviation_from_optimal);

        // deviation 0.15–0.30 → moderate_aging
        // CAII=0.60: v = 0.456 + 0.544×0.40 = 0.456 + 0.218 = 0.674, dev=0.218
        update_ze_health_state(&mut ze, 0.60, None);
        assert_eq!(ze.interpretation(), "moderate_aging",
            "CAII=0.60, dev={:.4} → ожидается 'moderate_aging'", ze.deviation_from_optimal);

        // deviation > 0.45 → near_collapse
        // CAII=0.10: v = 0.456 + 0.544×0.90 = 0.456 + 0.490 = 0.946, dev=0.490
        update_ze_health_state(&mut ze, 0.10, None);
        assert_eq!(ze.interpretation(), "near_collapse",
            "CAII=0.10, dev={:.4} → ожидается 'near_collapse'", ze.deviation_from_optimal);
    }

    /// Монотонность: чем ниже CAII, тем выше v и deviation
    #[test]
    fn test_monotone_in_caii() {
        let caii_values = [1.0_f32, 0.8, 0.6, 0.4, 0.2, 0.0];
        let mut prev_v = -1.0_f32;
        for &caii in &caii_values {
            let mut ze = ZeHealthState::default();
            update_ze_health_state(&mut ze, caii, None);
            assert!(ze.v > prev_v || (ze.v - prev_v).abs() < 1e-6,
                "CAII={}: v={:.4} должно монотонно расти", caii, ze.v);
            prev_v = ze.v;
        }
    }
}
