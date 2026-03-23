//! Пероксисомы — детоксикация H₂O₂ (уровень -1: органоиды).
//!
//! Каталаза: 2 H₂O₂ → 2 H₂O + O₂. Снижается с возрастом.
//! Меньше каталазы → больше H₂O₂ → Fenton-реакция сильнее → больше OH·.
//!
//! Связь с ROSCascadeState:
//!   h2o2_clearance_rate → снижает накопление hydrogen_peroxide в ROSCascadeState.

use cell_dt_core::components::PeroxisomeState;

/// Параметры пероксисомальной динамики.
#[derive(Debug, Clone)]
pub struct PeroxisomeParams {
    /// Скорость возрастного снижения каталазы [/год после 40 лет].
    /// Tian et al. 1998: ~0.4%/год → 0.004/год.
    pub catalase_aging_rate: f32,
    /// Вклад ROS в инактивацию каталазы (оксидативная инактивация).
    pub ros_catalase_inactivation: f32,
}

impl Default for PeroxisomeParams {
    fn default() -> Self {
        Self {
            catalase_aging_rate:       0.004,
            ros_catalase_inactivation: 0.08,
        }
    }
}

/// Обновить PeroxisomeState на один шаг.
///
/// # Аргументы
/// * `pex`       — состояние (in/out).
/// * `age_years` — возраст организма (лет).
/// * `ros_level` — суммарный ROS [0..1].
/// * `params`    — параметры.
/// * `dt_years`  — шаг (лет).
pub fn update_peroxisome_state(
    pex:       &mut PeroxisomeState,
    age_years: f32,
    ros_level: f32,
    params:    &PeroxisomeParams,
    dt_years:  f32,
) {
    // Возрастное снижение каталазы (только после 40 лет)
    let age_loss = if age_years > 40.0 {
        params.catalase_aging_rate * dt_years
    } else {
        0.0
    };

    // ROS-инактивация каталазы
    let ros_loss = ros_level * params.ros_catalase_inactivation * dt_years;

    pex.catalase_activity = (pex.catalase_activity - age_loss - ros_loss).clamp(0.10, 1.0);

    // fatty_acid_oxidation снижается пропорционально каталазе (общее здоровье пероксисомы)
    pex.fatty_acid_oxidation = (pex.catalase_activity * 0.95).clamp(0.0, 1.0);

    pex.update_derived(); // h2o2_clearance_rate
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> PeroxisomeParams { PeroxisomeParams::default() }

    #[test]
    fn test_pristine_full_catalase() {
        let s = PeroxisomeState::pristine();
        assert!((s.catalase_activity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_no_aging_below_40() {
        let mut s = PeroxisomeState::pristine();
        update_peroxisome_state(&mut s, 30.0, 0.0, &p(), 10.0);
        assert!((s.catalase_activity - 1.0).abs() < 1e-4,
            "До 40 лет возрастного снижения нет");
    }

    #[test]
    fn test_aging_reduces_catalase_after_40() {
        let mut s = PeroxisomeState::pristine();
        update_peroxisome_state(&mut s, 60.0, 0.0, &p(), 20.0);
        assert!(s.catalase_activity < 1.0,
            "После 40 лет каталаза снижается");
    }

    #[test]
    fn test_ros_inactivates_catalase() {
        let mut s = PeroxisomeState::pristine();
        update_peroxisome_state(&mut s, 30.0, 1.0, &p(), 5.0);
        assert!(s.catalase_activity < 1.0,
            "ROS → оксидативная инактивация каталазы");
    }

    #[test]
    fn test_h2o2_clearance_proportional_to_catalase() {
        let mut s = PeroxisomeState::pristine();
        s.catalase_activity = 0.60;
        s.update_derived();
        assert!(s.h2o2_clearance_rate <= 0.60,
            "clearance ≤ catalase_activity");
    }

    #[test]
    fn test_catalase_floor() {
        let mut s = PeroxisomeState::pristine();
        for _ in 0..500 {
            update_peroxisome_state(&mut s, 80.0, 1.0, &p(), 1.0);
        }
        assert!(s.catalase_activity >= 0.10,
            "Каталаза не уходит ниже 10%");
    }
}
