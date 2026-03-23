//! Состояние 3D-хроматина и TAD-структуры (уровень -3: молекулы).
//!
//! TAD-домены нарушаются при деметилировании CTCF-сайтов (methylation_age↑).
//! Потеря гетерохроматина → разборка SAHF → транскрипционный шум, SASP↑.
//! dna_accessibility↑ → DDR реагирует агрессивнее → gamma_h2ax↑.
//!
//! # Формулы
//!
//! ```text
//! tad_integrity = (1 − methylation_age × 0.60).clamp(0.1, 1.0)
//! heterochromatin_fraction = (0.30 − total_damage × 0.20).clamp(0.05, 0.30)
//! dna_accessibility = update_derived(tad, het)
//! ```

use cell_dt_core::components::ChromatinState;

/// Параметры хроматиновой динамики.
#[derive(Debug, Clone)]
pub struct ChromatinParams {
    /// Насколько сильно methylation_age нарушает TAD-целостность [0..1].
    /// 0.60 → при methylation_age=1.0: tad_integrity = 0.40.
    pub methylation_tad_sensitivity:    f32,
    /// Насколько сильно суммарное повреждение снижает гетерохроматин.
    /// Исходное значение гетерохроматина: 0.30 (30% генома).
    pub damage_het_sensitivity:         f32,
}

impl Default for ChromatinParams {
    fn default() -> Self {
        Self {
            methylation_tad_sensitivity: 0.60,
            damage_het_sensitivity:      0.20,
        }
    }
}

/// Обновить ChromatinState на один шаг.
///
/// # Аргументы
/// * `ch`             — текущее состояние (изменяется на месте).
/// * `methylation_age` — нормированный эпигенетический возраст [0..1].
/// * `total_damage`   — суммарный урон (из CentriolarDamageState) [0..1].
/// * `params`         — параметры модели.
pub fn update_chromatin_state(
    ch:              &mut ChromatinState,
    methylation_age: f32,
    total_damage:    f32,
    params:          &ChromatinParams,
) {
    // TAD нарушаются при деметилировании CTCF-сайтов
    ch.tad_integrity =
        (1.0 - methylation_age * params.methylation_tad_sensitivity)
            .clamp(0.10, 1.0);

    // SAHF разрушаются по мере накопления повреждений
    ch.heterochromatin_fraction =
        (0.30 - total_damage * params.damage_het_sensitivity)
            .clamp(0.05, 0.30);

    ch.update_derived();
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> ChromatinParams { ChromatinParams::default() }

    #[test]
    fn test_pristine_tad_intact() {
        let s = ChromatinState::pristine();
        assert!((s.tad_integrity - 1.0).abs() < 1e-5,
            "Молодая клетка: TAD интактны");
    }

    #[test]
    fn test_methylation_reduces_tad() {
        let mut s = ChromatinState::pristine();
        update_chromatin_state(&mut s, 0.8, 0.0, &p());
        assert!(s.tad_integrity < 0.6,
            "Высокий methylation_age → TAD нарушены");
    }

    #[test]
    fn test_damage_reduces_heterochromatin() {
        let mut s = ChromatinState::pristine();
        update_chromatin_state(&mut s, 0.0, 1.0, &p());
        assert!(s.heterochromatin_fraction < 0.20,
            "Высокий ущерб → гетерохроматин снижается");
    }

    #[test]
    fn test_accessibility_increases_with_damage() {
        let young = ChromatinState::pristine();
        let mut old = ChromatinState::pristine();
        update_chromatin_state(&mut old, 0.9, 0.8, &p());
        assert!(old.dna_accessibility > young.dna_accessibility,
            "Старая клетка: ДНК более доступна (TAD нарушены)");
    }

    #[test]
    fn test_clamps_prevent_invalid_values() {
        let mut s = ChromatinState::pristine();
        update_chromatin_state(&mut s, 2.0, 2.0, &p());
        assert!(s.tad_integrity >= 0.10);
        assert!(s.heterochromatin_fraction >= 0.05);
        assert!(s.dna_accessibility <= 1.0);
    }
}
