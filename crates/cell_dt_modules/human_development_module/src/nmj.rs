//! Нейромышечный синапс (НМС) — уровень +1: ткань.
//!
//! Денервация нервно-мышечного соединения при старении. Высокий ROS и
//! белковые агрегаты ускоряют денервацию; BDNF-поддержка обеспечивает
//! реиннервацию. Снижение плотности АХ-рецепторов → саркопения.
//!
//! # Механизм
//!
//! ```text
//! denervation_index ↑ ← base_rate + ros × ros_factor
//! denervation_index ↓ ← reinnervation_capacity × reinnervation_rate
//! reinnervation_capacity = bdnf_support × (1 − aggregates × 0.50)
//!
//! ach_receptor_density = (1 − denervation) × (1 − aggregates × 0.30)
//! synaptic_transmission = ach_receptor_density × reinnervation_capacity
//! ```

use serde::{Deserialize, Serialize};
use cell_dt_core::components::NeuromuscularJunctionState;

// ─────────────────────────────────────────────────────────────────────────────
// Параметры
// ─────────────────────────────────────────────────────────────────────────────

/// Параметры нейромышечного соединения.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NMJParams {
    /// Базовая скорость денервации [/год].
    pub base_denervation_rate: f32,
    /// Коэффициент усиления денервации ROS [0..1].
    pub ros_denervation_factor: f32,
    /// Скорость реиннервации [/год].
    pub reinnervation_rate: f32,
    /// Поддержка BDNF (нейротрофический фактор) [0..1].
    pub bdnf_support: f32,
}

impl Default for NMJParams {
    fn default() -> Self {
        Self {
            base_denervation_rate: 0.0002,
            ros_denervation_factor: 0.30,
            reinnervation_rate: 0.10,
            bdnf_support: 0.80,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Основная функция обновления
// ─────────────────────────────────────────────────────────────────────────────

/// Обновить NeuromuscularJunctionState за один шаг.
///
/// # Аргументы
/// * `state`      — изменяемый NeuromuscularJunctionState.
/// * `params`     — параметры НМС.
/// * `ros_level`  — уровень ROS [0..1] из CentriolarDamageState.
/// * `aggregates` — уровень белковых агрегатов [0..1] из CentriolarDamageState.
/// * `dt`         — шаг времени [лет].
pub fn update_nmj_state(
    state: &mut NeuromuscularJunctionState,
    params: &NMJParams,
    ros_level: f32,
    aggregates: f32,
    dt: f32,
) {
    // Способность к реиннервации: BDNF снижается при агрегатах
    state.reinnervation_capacity = params.bdnf_support * (1.0 - aggregates * 0.50);

    // Денервация: нарастает от ROS и базовой скорости
    state.denervation_index +=
        (params.base_denervation_rate + ros_level * params.ros_denervation_factor) * dt;

    // Реиннервация противодействует денервации
    state.denervation_index -=
        state.reinnervation_capacity * params.reinnervation_rate * dt;

    state.denervation_index = state.denervation_index.clamp(0.0, 1.0);

    // Плотность АХ-рецепторов: снижается при денервации и агрегатах
    state.ach_receptor_density =
        (1.0 - state.denervation_index) * (1.0 - aggregates * 0.30);

    // Синаптическая передача
    state.synaptic_transmission = state.ach_receptor_density * state.reinnervation_capacity;
}

/// Штраф к мышечной функции при тяжёлой денервации.
///
/// Если synaptic_transmission < 0.50 → мышечная функция снижена на 20%.
pub fn nmj_muscle_penalty(state: &NeuromuscularJunctionState) -> f32 {
    if state.synaptic_transmission < 0.50 {
        0.80
    } else {
        1.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 365.25;

    /// Здоровая ниша: низкий ROS и агрегаты → денервация остаётся близкой к 0
    #[test]
    fn test_healthy_denervation_stays_low() {
        let mut state = NeuromuscularJunctionState::default();
        let params = NMJParams::default();

        // 50 лет в норме
        for _ in 0..(50 * 365) {
            update_nmj_state(&mut state, &params, 0.02, 0.01, DT);
        }

        assert!(state.denervation_index < 0.20,
            "Здоровая ниша: денервация < 0.20, получено: {:.4}",
            state.denervation_index);
    }

    /// Высокий ROS → денервация нарастает быстрее
    #[test]
    fn test_high_ros_increases_denervation() {
        let mut state_normal = NeuromuscularJunctionState::default();
        let mut state_stress = NeuromuscularJunctionState::default();
        let params = NMJParams::default();

        for _ in 0..(30 * 365) {
            update_nmj_state(&mut state_normal, &params, 0.05, 0.0, DT);
            update_nmj_state(&mut state_stress, &params, 0.80, 0.0, DT);
        }

        assert!(state_stress.denervation_index > state_normal.denervation_index,
            "ROS↑ должен ускорять денервацию: {:.4} vs {:.4}",
            state_stress.denervation_index, state_normal.denervation_index);
    }

    /// Высокий BDNF снижает денервацию по сравнению с низким BDNF
    #[test]
    fn test_bdnf_protection_reduces_denervation() {
        let mut state_low_bdnf  = NeuromuscularJunctionState::default();
        let mut state_high_bdnf = NeuromuscularJunctionState::default();
        let params_low  = NMJParams { bdnf_support: 0.20, ..NMJParams::default() };
        let params_high = NMJParams { bdnf_support: 0.95, ..NMJParams::default() };

        for _ in 0..(40 * 365) {
            update_nmj_state(&mut state_low_bdnf,  &params_low,  0.20, 0.05, DT);
            update_nmj_state(&mut state_high_bdnf, &params_high, 0.20, 0.05, DT);
        }

        assert!(state_high_bdnf.denervation_index < state_low_bdnf.denervation_index,
            "BDNF↑ должен снижать денервацию: {:.4} vs {:.4}",
            state_high_bdnf.denervation_index, state_low_bdnf.denervation_index);
    }

    /// Тяжёлая денервация → низкая плотность АХ-рецепторов
    #[test]
    fn test_severe_denervation_low_ach_density() {
        let mut state = NeuromuscularJunctionState::default();
        state.denervation_index = 0.90;
        let params = NMJParams::default();
        update_nmj_state(&mut state, &params, 0.80, 0.50, DT);

        assert!(state.ach_receptor_density < 0.20,
            "Тяжёлая денервация → низкий АХ: {:.4}",
            state.ach_receptor_density);
    }

    /// synaptic_transmission < 0.50 → штраф = 0.80
    #[test]
    fn test_muscle_penalty_when_low_transmission() {
        let mut state = NeuromuscularJunctionState::default();
        state.synaptic_transmission = 0.30;
        assert!((nmj_muscle_penalty(&state) - 0.80).abs() < 1e-6,
            "Штраф при низкой передаче: {:.4}", nmj_muscle_penalty(&state));

        state.synaptic_transmission = 0.75;
        assert!((nmj_muscle_penalty(&state) - 1.0).abs() < 1e-6,
            "Норма при высокой передаче: {:.4}", nmj_muscle_penalty(&state));
    }
}
