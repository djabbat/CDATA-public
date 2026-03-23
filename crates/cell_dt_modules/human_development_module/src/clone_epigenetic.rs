//! Клон-специфическая эпигенетическая память (Уровень 0: клетка).
//!
//! Разные клоны накапливают methylation_age с разной скоростью.
//! TET2/DNMT3A мутантные CHIP-клоны: ускоренный дрейф.
//!
//! # Формула
//!
//! ```text
//! clone_drift_accumulated += clone_drift_rate × damage_multiplier × dt_years
//! effective_methylation = epi_clock.methylation_age + clone_drift_accumulated
//! ```

use cell_dt_core::components::CloneEpigeneticState;

/// Параметры клон-специфического эпигенетического дрейфа.
#[derive(Debug, Clone)]
pub struct CloneEpigeneticParams {
    /// Как сильно суммарный ущерб усиливает дрейф.
    /// total_damage=0.5 → drift_multiplier = 1 + 0.5 × damage_amplification.
    pub damage_amplification: f32,
}

impl Default for CloneEpigeneticParams {
    fn default() -> Self {
        Self { damage_amplification: 0.50 }
    }
}

/// Обновить CloneEpigeneticState на один шаг.
///
/// # Аргументы
/// * `ce`           — состояние (in/out).
/// * `total_damage` — суммарный ущерб из CentriolarDamageState [0..1].
/// * `params`       — параметры.
/// * `dt_years`     — шаг (лет).
pub fn update_clone_epigenetic_state(
    ce:           &mut CloneEpigeneticState,
    total_damage: f32,
    params:       &CloneEpigeneticParams,
    dt_years:     f32,
) {
    let damage_multiplier = 1.0 + total_damage * params.damage_amplification;
    ce.clone_drift_accumulated =
        (ce.clone_drift_accumulated
            + ce.clone_drift_rate * damage_multiplier * dt_years)
            .clamp(0.0, 1.0);
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> CloneEpigeneticParams { CloneEpigeneticParams::default() }

    #[test]
    fn test_neutral_clone_no_drift_initially() {
        let s = CloneEpigeneticState::neutral();
        assert!((s.clone_drift_accumulated - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_drift_accumulates_over_time() {
        let mut s = CloneEpigeneticState::neutral();
        update_clone_epigenetic_state(&mut s, 0.0, &p(), 10.0);
        assert!(s.clone_drift_accumulated > 0.0,
            "Нейтральный клон: дрейф накапливается со временем");
    }

    #[test]
    fn test_damage_amplifies_drift() {
        let mut lo = CloneEpigeneticState::neutral();
        let mut hi = CloneEpigeneticState::neutral();
        update_clone_epigenetic_state(&mut lo, 0.0, &p(), 10.0);
        update_clone_epigenetic_state(&mut hi, 0.8, &p(), 10.0);
        assert!(hi.clone_drift_accumulated > lo.clone_drift_accumulated,
            "Больше повреждений → более быстрый эпигенетический дрейф");
    }

    #[test]
    fn test_chip_clone_faster_drift() {
        let mut neutral  = CloneEpigeneticState::neutral();
        let mut chip     = CloneEpigeneticState::tet2_chip();
        update_clone_epigenetic_state(&mut neutral, 0.0, &p(), 10.0);
        update_clone_epigenetic_state(&mut chip,    0.0, &p(), 10.0);
        assert!(chip.clone_drift_accumulated > neutral.clone_drift_accumulated,
            "TET2-CHIP-клон: дрейф быстрее нейтрального");
    }

    #[test]
    fn test_effective_contribution_includes_baseline() {
        let s = CloneEpigeneticState::tet2_chip();
        assert!(s.effective_methylation_contribution() >= 0.10,
            "TET2 базовая линия: вклад ≥ 10%");
    }

    #[test]
    fn test_drift_clamped_at_one() {
        let mut s = CloneEpigeneticState::neutral();
        for _ in 0..500 {
            update_clone_epigenetic_state(&mut s, 1.0, &p(), 1.0);
        }
        assert!(s.clone_drift_accumulated <= 1.0,
            "Дрейф не превышает 1.0");
    }
}
