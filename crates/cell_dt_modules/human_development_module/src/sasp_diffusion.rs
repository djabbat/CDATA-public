//! Пространственная диффузия SASP — уровень 0: клетка.
//!
//! SASP (Senescence-Associated Secretory Phenotype) распространяется паракринно
//! на соседние стволовые ниши. Эффективный SASP = local + diffused from neighbors.
//!
//! # Механизм
//!
//! ```text
//! local_sasp    ← устанавливается из InflammagingState.sasp_intensity
//! received_sasp ← mean(neighbor_sasps) × diffusion_coefficient
//! effective_sasp = (local + received).clamp(0, 1)
//!
//! Bystander-эффект: если effective > threshold → ros_boost
//! ```

use serde::{Deserialize, Serialize};
use cell_dt_core::components::SaspDiffusionState;

// ─────────────────────────────────────────────────────────────────────────────
// Параметры
// ─────────────────────────────────────────────────────────────────────────────

/// Параметры пространственной диффузии SASP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaspDiffusionParams {
    /// Коэффициент диффузии SASP от соседей [0..1].
    pub diffusion_coefficient: f32,
    /// Скорость распада SASP [/год] (не используется в текущей версии, зарезервирован).
    pub decay_rate: f32,
    /// Порог bystander-эффекта [0..1].
    pub bystander_threshold: f32,
}

impl Default for SaspDiffusionParams {
    fn default() -> Self {
        Self {
            diffusion_coefficient: 0.15,
            decay_rate: 0.20,
            bystander_threshold: 0.40,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Основная функция обновления
// ─────────────────────────────────────────────────────────────────────────────

/// Обновить SaspDiffusionState за один шаг.
///
/// # Аргументы
/// * `state`           — изменяемый SaspDiffusionState.
/// * `params`          — параметры диффузии.
/// * `neighbor_sasps`  — уровни SASP у соседних ниш (из их InflammagingState.sasp_intensity).
///
/// `state.local_sasp` должен быть установлен снаружи из InflammagingState.sasp_intensity
/// до вызова этой функции.
pub fn update_sasp_diffusion_state(
    state: &mut SaspDiffusionState,
    params: &SaspDiffusionParams,
    neighbor_sasps: &[f32],
) {
    // Среднее SASP у соседей
    let mean_neighbor = if neighbor_sasps.is_empty() {
        0.0
    } else {
        neighbor_sasps.iter().sum::<f32>() / neighbor_sasps.len() as f32
    };

    // Полученный SASP через диффузию
    state.received_sasp = mean_neighbor * params.diffusion_coefficient;

    // Эффективный SASP
    state.effective_sasp = (state.local_sasp + state.received_sasp).clamp(0.0, 1.0);
}

/// Вычислить дополнительный ROS-буст от bystander-эффекта SASP.
///
/// Если effective_sasp > bystander_threshold → ROS-boost пропорционален превышению.
pub fn sasp_bystander_effect(
    state: &SaspDiffusionState,
    params: &SaspDiffusionParams,
) -> f32 {
    if state.effective_sasp > params.bystander_threshold {
        (state.effective_sasp - params.bystander_threshold) * 0.10
    } else {
        0.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Нет соседей → received_sasp = 0, effective = local
    #[test]
    fn test_no_neighbors() {
        let mut state = SaspDiffusionState::default();
        state.local_sasp = 0.30;
        let params = SaspDiffusionParams::default();
        update_sasp_diffusion_state(&mut state, &params, &[]);

        assert!((state.received_sasp - 0.0).abs() < 1e-6,
            "received_sasp при нет соседей: {:.4}", state.received_sasp);
        assert!((state.effective_sasp - 0.30).abs() < 1e-5,
            "effective = local: {:.4}", state.effective_sasp);
    }

    /// Один высокий сосед → received_sasp > 0
    #[test]
    fn test_one_high_neighbor() {
        let mut state = SaspDiffusionState::default();
        state.local_sasp = 0.10;
        let params = SaspDiffusionParams::default();
        update_sasp_diffusion_state(&mut state, &params, &[0.80]);

        assert!(state.received_sasp > 0.0,
            "Высокий сосед должен создавать received_sasp: {:.4}", state.received_sasp);
        let expected_received = 0.80 * 0.15;
        assert!((state.received_sasp - expected_received).abs() < 1e-5,
            "received = 0.80×0.15: {:.4}", state.received_sasp);
    }

    /// Ниже порога → bystander_effect = 0
    #[test]
    fn test_below_threshold_no_bystander() {
        let mut state = SaspDiffusionState::default();
        state.effective_sasp = 0.30; // < threshold 0.40
        let params = SaspDiffusionParams::default();

        let effect = sasp_bystander_effect(&state, &params);
        assert!((effect - 0.0).abs() < 1e-6,
            "Ниже порога: bystander_effect = 0, получено: {:.4}", effect);
    }

    /// Выше порога → bystander_effect > 0
    #[test]
    fn test_above_threshold_bystander_positive() {
        let mut state = SaspDiffusionState::default();
        state.effective_sasp = 0.60; // > threshold 0.40
        let params = SaspDiffusionParams::default();

        let effect = sasp_bystander_effect(&state, &params);
        assert!(effect > 0.0,
            "Выше порога: bystander_effect > 0, получено: {:.4}", effect);
        let expected = (0.60 - 0.40) * 0.10;
        assert!((effect - expected).abs() < 1e-6,
            "bystander = (eff-threshold)×0.10: {:.4} vs {:.4}", effect, expected);
    }

    /// Несколько соседей → усреднение
    #[test]
    fn test_multiple_neighbors_averaged() {
        let mut state = SaspDiffusionState::default();
        state.local_sasp = 0.0;
        let params = SaspDiffusionParams::default();
        let neighbors = [0.20_f32, 0.40, 0.60];
        update_sasp_diffusion_state(&mut state, &params, &neighbors);

        let expected_mean = (0.20 + 0.40 + 0.60) / 3.0;
        let expected_received = expected_mean * 0.15;
        assert!((state.received_sasp - expected_received).abs() < 1e-5,
            "Среднее соседей × коэфф: {:.4} vs {:.4}",
            state.received_sasp, expected_received);
    }
}
