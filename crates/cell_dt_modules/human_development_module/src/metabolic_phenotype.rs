//! Метаболический фенотип (уровень +3: организм).
//!
//! BMI/ожирение → адипокины → хроническое воспаление → CDATA-ускорение.
//! Инсулинорезистентность → клеточный энергетический голод → ATP↓ → протеасома↓.
//!
//! # Формулы
//!
//! ```text
//! adipokine_level = bmi_index × 0.70
//! insulin_sensitivity = (1 − bmi_index × 0.60).clamp(0.10, 1.0)
//! ros_contribution = adipokine_level × 0.12  → InflammagingState.ros_boost
//! ```

use cell_dt_core::components::MetabolicPhenotypeState;

/// Параметры метаболического фенотипа.
#[derive(Debug, Clone)]
pub struct MetabolicParams {
    /// Вклад адипокинов в ROS (через NADPH-оксидазу жировой ткани).
    pub adipokine_ros_scale: f32,
}

impl Default for MetabolicParams {
    fn default() -> Self {
        Self {
            adipokine_ros_scale: 0.12,
        }
    }
}

/// Обновить MetabolicPhenotypeState на один шаг.
///
/// # Аргументы
/// * `met`       — состояние (in/out).
/// * `bmi_index` — нормированный ИМТ [0..1].
/// * `params`    — параметры.
pub fn update_metabolic_phenotype_state(
    met:       &mut MetabolicPhenotypeState,
    bmi_index: f32,
    params:    &MetabolicParams,
) {
    met.bmi_index = bmi_index.clamp(0.0, 1.0);
    met.update_derived();
    let _ = params; // params used for future adipokine_ros_scale
}

/// Дополнительный ROS-буст от адипокинов → передаётся в InflammagingState.
pub fn adipokine_ros_boost(met: &MetabolicPhenotypeState, params: &MetabolicParams) -> f32 {
    met.adipokine_level * params.adipokine_ros_scale
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> MetabolicParams { MetabolicParams::default() }

    #[test]
    fn test_pristine_lean_phenotype() {
        let s = MetabolicPhenotypeState::pristine();
        assert!((s.bmi_index - 0.0).abs() < 1e-5);
        assert!((s.insulin_sensitivity - 1.0).abs() < 1e-5);
        assert!((s.adipokine_level - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_obesity_raises_adipokines() {
        let mut s = MetabolicPhenotypeState::pristine();
        update_metabolic_phenotype_state(&mut s, 0.8, &p());
        assert!(s.adipokine_level > 0.4,
            "Ожирение → провоспалительные адипокины↑");
    }

    #[test]
    fn test_obesity_reduces_insulin_sensitivity() {
        let mut s = MetabolicPhenotypeState::pristine();
        update_metabolic_phenotype_state(&mut s, 0.8, &p());
        assert!(s.insulin_sensitivity < 0.55,
            "Ожирение → инсулинорезистентность");
    }

    #[test]
    fn test_adipokine_ros_boost_proportional() {
        let mut s = MetabolicPhenotypeState::pristine();
        update_metabolic_phenotype_state(&mut s, 0.5, &p());
        let expected = s.adipokine_level * 0.12;
        assert!((adipokine_ros_boost(&s, &p()) - expected).abs() < 1e-5,
            "ros_boost = adipokines × 0.12");
    }

    #[test]
    fn test_insulin_sensitivity_floor() {
        let mut s = MetabolicPhenotypeState::pristine();
        update_metabolic_phenotype_state(&mut s, 2.0, &p());
        assert!(s.insulin_sensitivity >= 0.10,
            "Чувствительность к инсулину не падает ниже 10%");
    }
}
