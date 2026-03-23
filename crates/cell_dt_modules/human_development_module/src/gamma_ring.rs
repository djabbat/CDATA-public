//! γ-тубулиновые кольцевые комплексы (γ-TuRC) — уровень -3: нуклеация MT.
//!
//! γ-TuRC нуклеируют микротрубочки на центриолях. Ninein-integrity из
//! AppendageProteinState определяет плотность γ-TuRC на субдистальных придатках.
//!
//! # Механизм
//!
//! ```text
//! Ninein (субдистальный придаток)
//!     → якорение γ-TuRC на центриоли
//!     → pericentriolar_density = ninein × ninein_coupling + (1 − ninein) × 0.20
//!
//! OH· + агрегаты
//!     → карбонилирование γ-TuRC → ring_integrity ↓
//!     → PCM-репарация: ring_integrity частично восстанавливается
//!
//! nucleation_efficiency = ring_integrity × pericentriolar_density
//! ```

use serde::{Deserialize, Serialize};
use cell_dt_core::components::GammaRingState;

// ─────────────────────────────────────────────────────────────────────────────
// Параметры
// ─────────────────────────────────────────────────────────────────────────────

/// Параметры γ-TuRC нуклеационного комплекса.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaRingParams {
    /// Базовая эффективность нуклеации [0..1].
    pub base_nucleation: f32,
    /// Коэффициент сцепления Ninein с γ-TuRC [0..1].
    pub ninein_coupling: f32,
    /// Чувствительность γ-TuRC к OH·-радикалу [0..1].
    pub ros_sensitivity: f32,
    /// Скорость PCM-репарации γ-TuRC кольца [/шаг].
    pub pcm_repair_rate: f32,
}

impl Default for GammaRingParams {
    fn default() -> Self {
        Self {
            base_nucleation: 0.90,
            ninein_coupling: 0.70,
            ros_sensitivity: 0.50,
            pcm_repair_rate: 0.05,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Основная функция обновления
// ─────────────────────────────────────────────────────────────────────────────

/// Обновить GammaRingState за один шаг.
///
/// # Аргументы
/// * `state`       — изменяемый GammaRingState.
/// * `params`      — параметры γ-TuRC.
/// * `ninein`      — целостность Ninein [0..1] из AppendageProteinState.
/// * `oh_radical`  — уровень OH·-радикала [0..1] из ROSCascadeState.
/// * `aggregates`  — уровень белковых агрегатов [0..1] из CentriolarDamageState.
/// * `dt`          — шаг времени [лет].
pub fn update_gamma_ring_state(
    state: &mut GammaRingState,
    params: &GammaRingParams,
    ninein: f32,
    oh_radical: f32,
    aggregates: f32,
    dt: f32,
) {
    // Перицентриолярная плотность: Ninein якорит γ-TuRC; без Ninein → остаточный уровень 0.20
    state.pericentriolar_density =
        ninein * params.ninein_coupling + (1.0 - ninein) * 0.20;

    // Повреждение кольца: OH· и агрегаты → карбонилирование γ-TuRC
    let damage_rate = oh_radical * params.ros_sensitivity + aggregates * 0.30;
    state.ring_integrity -= damage_rate * dt;

    // PCM-репарация: восстанавливает ring_integrity (логистический член)
    state.ring_integrity += params.pcm_repair_rate * (1.0 - state.ring_integrity) * dt;
    state.ring_integrity = state.ring_integrity.clamp(0.0, 1.0);

    // Нуклеационная эффективность
    state.nucleation_efficiency = state.ring_integrity * state.pericentriolar_density;
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 365.25;

    /// Здоровая ниша: ninein=1.0 → pericentriolar_density = 0.70, высокая нуклеация
    #[test]
    fn test_healthy_ninein_full() {
        let mut state = GammaRingState::default();
        state.ring_integrity = 1.0;
        let params = GammaRingParams::default();
        update_gamma_ring_state(&mut state, &params, 1.0, 0.0, 0.0, DT);

        // pericentriolar_density = 1.0×0.70 + 0.0×0.20 = 0.70
        assert!((state.pericentriolar_density - 0.70).abs() < 1e-5,
            "pericentriolar_density: {:.4}", state.pericentriolar_density);
        // nucleation = ring_integrity × density ≈ 1.0 × 0.70 = 0.70
        assert!(state.nucleation_efficiency > 0.60,
            "Высокая нуклеация: {:.4}", state.nucleation_efficiency);
    }

    /// Состарившаяся ниша: ninein=0.2 → pericentriolar_density = 0.2×0.7 + 0.8×0.2 = 0.30
    #[test]
    fn test_aged_ninein_low() {
        let mut state = GammaRingState::default();
        state.ring_integrity = 1.0;
        let params = GammaRingParams::default();
        update_gamma_ring_state(&mut state, &params, 0.2, 0.0, 0.0, DT);

        let expected = 0.2_f32 * 0.70 + 0.8_f32 * 0.20; // = 0.30
        assert!((state.pericentriolar_density - expected).abs() < 1e-5,
            "pericentriolar_density: {:.4} vs expected {:.4}",
            state.pericentriolar_density, expected);
    }

    /// Высокий OH·-радикал → ring_integrity снижается
    #[test]
    fn test_ros_damage_reduces_ring_integrity() {
        let mut state_normal = GammaRingState::default();
        state_normal.ring_integrity = 0.90;
        let mut state_stress = GammaRingState::default();
        state_stress.ring_integrity = 0.90;
        let params = GammaRingParams::default();

        // 10 лет с высоким OH·
        for _ in 0..(10 * 365) {
            update_gamma_ring_state(&mut state_normal, &params, 1.0, 0.01, 0.0, DT);
            update_gamma_ring_state(&mut state_stress, &params, 1.0, 0.80, 0.0, DT);
        }

        assert!(state_stress.ring_integrity < state_normal.ring_integrity,
            "ROS↑ должен снижать ring_integrity: {:.4} vs {:.4}",
            state_stress.ring_integrity, state_normal.ring_integrity);
    }

    /// PCM-репарация восстанавливает ring_integrity из повреждённого состояния
    #[test]
    fn test_pcm_repair_restores_ring() {
        let mut state = GammaRingState::default();
        state.ring_integrity = 0.30; // повреждено
        let params = GammaRingParams::default();

        // 20 лет без стресса — репарация должна восстановить
        for _ in 0..(20 * 365) {
            update_gamma_ring_state(&mut state, &params, 1.0, 0.0, 0.0, DT);
        }

        assert!(state.ring_integrity > 0.50,
            "PCM-репарация должна восстанавливать ring_integrity: {:.4}",
            state.ring_integrity);
    }

    /// Ninein=0.0 → pericentriolar_density = 0.20 (минимальный остаточный уровень)
    #[test]
    fn test_zero_ninein_minimal_density() {
        let mut state = GammaRingState::default();
        state.ring_integrity = 1.0;
        let params = GammaRingParams::default();
        update_gamma_ring_state(&mut state, &params, 0.0, 0.0, 0.0, DT);

        assert!((state.pericentriolar_density - 0.20).abs() < 1e-5,
            "Без Ninein: плотность = 0.20, получено: {:.4}",
            state.pericentriolar_density);
    }

    /// nucleation_efficiency = ring_integrity × pericentriolar_density
    #[test]
    fn test_nucleation_equals_ring_times_density() {
        let mut state = GammaRingState::default();
        state.ring_integrity = 0.80;
        let params = GammaRingParams::default();
        update_gamma_ring_state(&mut state, &params, 0.5, 0.1, 0.1, DT);

        let expected = state.ring_integrity * state.pericentriolar_density;
        assert!((state.nucleation_efficiency - expected).abs() < 1e-5,
            "nucleation = ring × density: {:.4} vs {:.4}",
            state.nucleation_efficiency, expected);
    }
}
