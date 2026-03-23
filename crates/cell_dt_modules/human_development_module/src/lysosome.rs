//! Лизосомальная система (уровень -1: органоиды).
//!
//! При возрастном защелачивании (pH↑) → снижение активности кислых гидролаз →
//! аутофагия не финализируется → агрегаты накапливаются.
//! При мембранной нестабильности → катепсины утекают → каспаз-сигнал.
//!
//! # Формулы
//!
//! ```text
//! d(ph)/dt = (ros_acidification − repair_rate × (ph − 5.0)) × dt
//! hydrolase_activity = (1 − (ph − 5.0) × 0.40).clamp(0, 1)
//! membrane_permeability = (oh_radical × 0.30).clamp(0, 1)
//! ```

use cell_dt_core::components::LysosomeState;

/// Параметры лизосомальной динамики.
#[derive(Debug, Clone)]
pub struct LysosomeParams {
    /// Скорость защелачивания при ROS [pH/год].
    /// OH· повреждает v-ATPase → pH повышается.
    pub ros_alkalinization_rate: f32,
    /// Скорость репарации кислой среды (v-ATPase repair) [/год].
    pub ph_repair_rate:          f32,
    /// Чувствительность мембраны к OH·.
    pub oh_membrane_sensitivity: f32,
}

impl Default for LysosomeParams {
    fn default() -> Self {
        Self {
            ros_alkalinization_rate: 0.50,
            ph_repair_rate:          0.80,
            oh_membrane_sensitivity: 0.30,
        }
    }
}

/// Обновить LysosomeState на один шаг.
///
/// # Аргументы
/// * `lys`        — состояние (in/out).
/// * `ros_level`  — суммарный ROS [0..1].
/// * `oh_radical` — уровень OH· [0..1].
/// * `params`     — параметры.
/// * `dt_years`   — шаг (лет).
pub fn update_lysosome_state(
    lys:       &mut LysosomeState,
    ros_level: f32,
    oh_radical: f32,
    params:    &LysosomeParams,
    dt_years:  f32,
) {
    // pH: защелачивается при ROS, восстанавливается репарацией
    let alkalinization = ros_level * params.ros_alkalinization_rate;
    let repair         = params.ph_repair_rate * (lys.ph_level - 5.0).max(0.0);
    lys.ph_level = (lys.ph_level + (alkalinization - repair) * dt_years)
        .clamp(4.5, 7.0);

    lys.update_derived(); // hydrolase_activity

    // Мембранная проницаемость от OH·
    lys.membrane_permeability =
        (oh_radical * params.oh_membrane_sensitivity).clamp(0.0, 1.0);
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> LysosomeParams { LysosomeParams::default() }

    #[test]
    fn test_pristine_acidic_and_active() {
        let s = LysosomeState::pristine();
        assert!((s.ph_level - 5.0).abs() < 1e-5);
        assert!((s.hydrolase_activity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ros_alkalinizes_lysosome() {
        let mut s = LysosomeState::pristine();
        update_lysosome_state(&mut s, 1.0, 0.0, &p(), 2.0);
        assert!(s.ph_level > 5.0,
            "ROS → защелачивание лизосомы");
    }

    #[test]
    fn test_high_ph_reduces_hydrolase() {
        let mut s = LysosomeState::pristine();
        s.ph_level = 6.5;
        s.update_derived();
        assert!(s.hydrolase_activity < 0.45,
            "pH=6.5 → гидролазы неактивны");
    }

    #[test]
    fn test_oh_damages_membrane() {
        let mut s = LysosomeState::pristine();
        update_lysosome_state(&mut s, 0.0, 1.0, &p(), 1.0);
        assert!(s.membrane_permeability > 0.0,
            "OH· → мембрана проницаема");
    }

    #[test]
    fn test_repair_normalizes_ph_at_low_ros() {
        let mut s = LysosomeState::pristine();
        s.ph_level = 6.0;
        // Без ROS → pH снижается к 5.0
        for _ in 0..5 {
            update_lysosome_state(&mut s, 0.0, 0.0, &p(), 0.5);
        }
        assert!(s.ph_level < 6.0,
            "Без ROS → pH восстанавливается");
    }
}
