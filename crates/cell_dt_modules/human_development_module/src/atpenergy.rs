//! Энергетический статус клетки через ATP/ADP-баланс (уровень -3: молекулы).
//!
//! Протеасома — АТФ-зависимый 26S-комплекс. При снижении energy_charge её активность
//! падает → убиквитинированные белки накапливаются → агрегаты растут.
//!
//! # Формула
//!
//! ```text
//! energy_charge = (ATP + 0.5·ADP) / (ATP + ADP + AMP)  [Atkinson 1968]
//! atp_adp_ratio_new = prev - ros_drain·dt + mito_supply·dt
//! energy_charge ≈ atp_adp_ratio × 0.9  (упрощённое соответствие)
//! proteasome_modifier = min(1.0, energy_charge / 0.70)
//! ```

use cell_dt_core::components::ATPEnergyState;

/// Параметры динамики АТФ.
#[derive(Debug, Clone)]
pub struct ATPEnergyParams {
    /// Скорость истощения АТФ из-за ROS-нагрузки [/год].
    /// При ros_level=1.0: atp_adp_ratio снижается на drain_per_ros за год.
    pub ros_drain:      f32,
    /// Скорость восстановления АТФ митохондриями [/год].
    /// mito_supply × membrane_potential → компенсирует ros_drain.
    pub mito_supply:    f32,
    /// Минимальное возможное значение atp_adp_ratio (клетка ещё жива).
    pub atp_min:        f32,
}

impl Default for ATPEnergyParams {
    fn default() -> Self {
        Self {
            ros_drain:   0.15,  // при ros=1.0: –0.15/год
            mito_supply: 0.20,  // при mito_potential=1.0: +0.20/год
            atp_min:     0.10,
        }
    }
}

/// Обновить ATPEnergyState на один шаг.
///
/// # Аргументы
/// * `atp`            — текущее состояние (изменяется на месте).
/// * `ros_level`      — текущий ROS-уровень [0..1].
/// * `mito_potential` — потенциал митохондриальной мембраны [0..1].
/// * `params`         — параметры модели.
/// * `dt_years`       — шаг времени (лет).
pub fn update_atp_energy_state(
    atp:            &mut ATPEnergyState,
    ros_level:      f32,
    mito_potential: f32,
    params:         &ATPEnergyParams,
    dt_years:       f32,
) {
    let drain   = ros_level * params.ros_drain * dt_years;
    let supply  = mito_potential * params.mito_supply * dt_years;
    atp.atp_adp_ratio =
        (atp.atp_adp_ratio - drain + supply).clamp(params.atp_min, 1.0);

    // Упрощённое соответствие: energy_charge ≈ atp_adp_ratio × 0.9
    // (учитывает небольшую долю AMP даже в здоровой клетке)
    atp.energy_charge = (atp.atp_adp_ratio * 0.90).clamp(0.0, 1.0);

    atp.update_derived();
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn pristine() -> ATPEnergyState { ATPEnergyState::pristine() }
    fn params()   -> ATPEnergyParams { ATPEnergyParams::default() }

    #[test]
    fn test_pristine_proteasome_modifier_is_one() {
        let s = pristine();
        assert!((s.proteasome_activity_modifier - 1.0).abs() < 1e-5,
            "Новая клетка: протеасома полная");
    }

    #[test]
    fn test_high_ros_depletes_atp() {
        let mut s = pristine();
        update_atp_energy_state(&mut s, 1.0, 0.0, &params(), 1.0);
        assert!(s.atp_adp_ratio < 0.90,
            "Высокий ROS + нет митохондрий → АТФ падает");
    }

    #[test]
    fn test_mito_restores_atp() {
        let mut s = pristine();
        s.atp_adp_ratio = 0.50;
        update_atp_energy_state(&mut s, 0.0, 1.0, &params(), 1.0);
        assert!(s.atp_adp_ratio > 0.50,
            "Хорошие митохондрии без ROS → АТФ восстанавливается");
    }

    #[test]
    fn test_proteasome_modifier_below_one_when_low_energy() {
        let mut s = pristine();
        s.atp_adp_ratio = 0.40;
        s.energy_charge = 0.36; // 0.40 × 0.9
        s.update_derived();
        assert!(s.proteasome_activity_modifier < 1.0,
            "Низкий заряд → протеасома < 100%");
    }

    #[test]
    fn test_atp_clamp_min() {
        let mut s = pristine();
        // Много шагов с максимальным ROS и нулевыми митохондриями
        for _ in 0..100 {
            update_atp_energy_state(&mut s, 1.0, 0.0, &params(), 1.0);
        }
        assert!(s.atp_adp_ratio >= params().atp_min,
            "АТФ не уходит ниже минимума");
    }

    #[test]
    fn test_energy_charge_proportional_to_atp() {
        let mut s = pristine();
        update_atp_energy_state(&mut s, 0.5, 0.5, &params(), 1.0);
        let expected_ec = s.atp_adp_ratio * 0.90;
        assert!((s.energy_charge - expected_ec).abs() < 1e-5,
            "energy_charge = atp_adp_ratio × 0.90");
    }
}
