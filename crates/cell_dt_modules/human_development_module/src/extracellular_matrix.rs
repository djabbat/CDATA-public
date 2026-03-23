//! Внеклеточный матрикс (уровень +1: ткань).
//!
//! Жёсткий матрикс → YAP/TAZ-ядерный → симметричные деления → истощение пула.
//! Перекрёстное сшивание коллагена через AGE + LOX-фермент (ROS-зависимый).
//!
//! # Формулы
//!
//! ```text
//! d(crosslinking)/dt = (ros_level × 0.06 + age_factor × 0.02 − repair × crosslinking) × dt
//! stiffness = crosslinking × 0.70
//! integrin_signaling = stiffness × 0.60 + 0.20
//! ```

use cell_dt_core::components::ExtracellularMatrixState;

/// Параметры ECM-динамики.
#[derive(Debug, Clone)]
pub struct ECMParams {
    /// Скорость сшивания через ROS/AGE-зависимый LOX [/год].
    pub ros_crosslink_rate:  f32,
    /// Возрастная скорость сшивания (пассивный процесс) [/год после 40].
    pub age_crosslink_rate:  f32,
    /// Скорость матрикс-металлопротеиназного ремоделирования [/год].
    pub mmp_remodel_rate:    f32,
}

impl Default for ECMParams {
    fn default() -> Self {
        Self {
            ros_crosslink_rate: 0.06,
            age_crosslink_rate: 0.02,
            mmp_remodel_rate:   0.10,
        }
    }
}

/// Обновить ExtracellularMatrixState на один шаг.
pub fn update_ecm_state(
    ecm:       &mut ExtracellularMatrixState,
    ros_level: f32,
    age_years: f32,
    params:    &ECMParams,
    dt_years:  f32,
) {
    let age_factor = if age_years > 40.0 { 1.0 } else { 0.0 };

    let production = ros_level * params.ros_crosslink_rate
        + age_factor * params.age_crosslink_rate;
    let repair = params.mmp_remodel_rate * ecm.collagen_crosslinking;

    ecm.collagen_crosslinking =
        (ecm.collagen_crosslinking + (production - repair) * dt_years)
            .clamp(0.0, 1.0);

    ecm.update_derived();
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> ECMParams { ECMParams::default() }

    #[test]
    fn test_pristine_soft_matrix() {
        let s = ExtracellularMatrixState::pristine();
        assert!((s.collagen_crosslinking - 0.0).abs() < 1e-5);
        assert!((s.stiffness - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_ros_increases_crosslinking() {
        let mut s = ExtracellularMatrixState::pristine();
        update_ecm_state(&mut s, 1.0, 30.0, &p(), 5.0);
        assert!(s.collagen_crosslinking > 0.0,
            "ROS → сшивание коллагена");
    }

    #[test]
    fn test_age_increases_crosslinking_after_40() {
        let mut s = ExtracellularMatrixState::pristine();
        update_ecm_state(&mut s, 0.0, 50.0, &p(), 10.0);
        assert!(s.collagen_crosslinking > 0.0,
            "После 40 лет → возрастное сшивание");
    }

    #[test]
    fn test_stiffness_follows_crosslinking() {
        let mut s = ExtracellularMatrixState::pristine();
        update_ecm_state(&mut s, 0.8, 50.0, &p(), 10.0);
        let expected_stiffness = (s.collagen_crosslinking * 0.70).clamp(0.0, 1.0);
        assert!((s.stiffness - expected_stiffness).abs() < 1e-4,
            "stiffness = crosslinking × 0.70");
    }

    #[test]
    fn test_no_crosslinking_below_40_without_ros() {
        let mut s = ExtracellularMatrixState::pristine();
        update_ecm_state(&mut s, 0.0, 30.0, &p(), 10.0);
        assert!(s.collagen_crosslinking < 1e-4,
            "До 40 лет без ROS → нет сшивания");
    }
}
