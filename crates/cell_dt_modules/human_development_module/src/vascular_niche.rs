//! Сосудистая ниша (уровень +1: ткань).
//!
//! При старении ангиогенез снижается (VEGF↓, HIF-1α дисрегуляция).
//! Менее плотная сосудистая сеть → относительная гипоксия → митохондрии
//! усиливают гликолиз → ROS↑ → CDATA-ускорение.
//!
//! # Формулы
//!
//! ```text
//! d(angiogenesis)/dt = (−age_rate − ros_rate × ros_level) × dt
//! oxygen_supply = growth_factor_gradient = angiogenesis_index
//! ```

use cell_dt_core::components::VascularNicheState;

/// Параметры сосудистой ниши.
#[derive(Debug, Clone)]
pub struct VascularNicheParams {
    /// Возрастная скорость редукции сосудистой сети [/год после 50].
    pub age_angio_loss_rate: f32,
    /// Скорость повреждения сосудов от ROS.
    pub ros_angio_loss_rate: f32,
}

impl Default for VascularNicheParams {
    fn default() -> Self {
        Self {
            age_angio_loss_rate: 0.003,
            ros_angio_loss_rate: 0.05,
        }
    }
}

/// Обновить VascularNicheState на один шаг.
pub fn update_vascular_niche_state(
    vas:       &mut VascularNicheState,
    age_years: f32,
    ros_level: f32,
    params:    &VascularNicheParams,
    dt_years:  f32,
) {
    let age_loss = if age_years > 50.0 {
        params.age_angio_loss_rate * (age_years - 50.0).min(30.0) / 30.0
    } else {
        0.0
    };
    let ros_loss = ros_level * params.ros_angio_loss_rate;

    vas.angiogenesis_index =
        (vas.angiogenesis_index - (age_loss + ros_loss) * dt_years)
            .clamp(0.10, 1.0);

    vas.update_derived();
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> VascularNicheParams { VascularNicheParams::default() }

    #[test]
    fn test_pristine_full_vasculature() {
        let s = VascularNicheState::pristine();
        assert!((s.angiogenesis_index - 1.0).abs() < 1e-5);
        assert!((s.oxygen_supply - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_stable_before_50_without_ros() {
        let mut s = VascularNicheState::pristine();
        update_vascular_niche_state(&mut s, 40.0, 0.0, &p(), 10.0);
        assert!((s.angiogenesis_index - 1.0).abs() < 1e-4,
            "До 50 лет без ROS → сосуды стабильны");
    }

    #[test]
    fn test_aging_reduces_vasculature() {
        let mut s = VascularNicheState::pristine();
        update_vascular_niche_state(&mut s, 70.0, 0.0, &p(), 20.0);
        assert!(s.angiogenesis_index < 1.0,
            "Старение → редукция сосудов");
    }

    #[test]
    fn test_ros_damages_vasculature() {
        let mut s = VascularNicheState::pristine();
        update_vascular_niche_state(&mut s, 30.0, 1.0, &p(), 5.0);
        assert!(s.angiogenesis_index < 1.0,
            "ROS → повреждение сосудов");
    }

    #[test]
    fn test_oxygen_follows_angiogenesis() {
        let mut s = VascularNicheState::pristine();
        update_vascular_niche_state(&mut s, 70.0, 0.5, &p(), 15.0);
        assert!((s.oxygen_supply - s.angiogenesis_index).abs() < 1e-4,
            "oxygen_supply = angiogenesis_index");
    }

    #[test]
    fn test_floor_at_010() {
        let mut s = VascularNicheState::pristine();
        for _ in 0..200 {
            update_vascular_niche_state(&mut s, 80.0, 1.0, &p(), 1.0);
        }
        assert!(s.angiogenesis_index >= 0.10,
            "Ангиогенез не уходит ниже 10%");
    }
}
