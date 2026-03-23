//! Рибосомальный аппарат (уровень -1: органоиды).
//!
//! Скорость трансляции → repair_rate придатков (CEP164 синтезируется de novo).
//! При снижении энергозаряда (ATP/GTP↓) → элонгация замедляется.
//! RQC (Ribosome Quality Control): при сбое → незавершённые цепи → агрегаты.
//!
//! # Формулы
//!
//! ```text
//! translation_rate = energy_charge × aminoacyl_availability × (1 − aggregates×0.25)
//! ribosome_quality = (1 − aggregates × 0.40).clamp(0.20, 1.0)
//! ```

use cell_dt_core::components::RibosomeState;

/// Параметры рибосомальной динамики.
#[derive(Debug, Clone)]
pub struct RibosomeParams {
    /// Чувствительность трансляции к агрегатам (блок рибосом).
    pub aggregates_translation_inhibition: f32,
    /// Чувствительность RQC к агрегатам.
    pub aggregates_rqc_inhibition:         f32,
}

impl Default for RibosomeParams {
    fn default() -> Self {
        Self {
            aggregates_translation_inhibition: 0.25,
            aggregates_rqc_inhibition:         0.40,
        }
    }
}

/// Обновить RibosomeState на один шаг (stateless — пересчёт каждый шаг).
///
/// # Аргументы
/// * `rib`                   — состояние (in/out).
/// * `energy_charge`         — энергетический заряд [0..1].
/// * `aminoacyl_availability` — доступность аминоацил-тРНК [0..1].
/// * `aggregates`            — уровень агрегатов [0..1].
/// * `params`                — параметры.
pub fn update_ribosome_state(
    rib:                   &mut RibosomeState,
    energy_charge:         f32,
    aminoacyl_availability: f32,
    aggregates:            f32,
    params:                &RibosomeParams,
) {
    rib.aminoacyl_availability = aminoacyl_availability;

    rib.translation_rate = (energy_charge
        * aminoacyl_availability
        * (1.0 - aggregates * params.aggregates_translation_inhibition))
        .clamp(0.0, 1.0);

    rib.ribosome_quality =
        (1.0 - aggregates * params.aggregates_rqc_inhibition)
            .clamp(0.20, 1.0);
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> RibosomeParams { RibosomeParams::default() }

    #[test]
    fn test_pristine_full_translation() {
        let s = RibosomeState::pristine();
        assert!((s.translation_rate - 1.0).abs() < 1e-5);
        assert!((s.ribosome_quality - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_low_energy_slows_translation() {
        let mut s = RibosomeState::pristine();
        update_ribosome_state(&mut s, 0.4, 1.0, 0.0, &p());
        assert!(s.translation_rate <= 0.41,
            "Низкий заряд → трансляция замедлена");
    }

    #[test]
    fn test_aggregates_block_translation() {
        let mut s = RibosomeState::pristine();
        update_ribosome_state(&mut s, 1.0, 1.0, 0.8, &p());
        assert!(s.translation_rate < 0.82,
            "Агрегаты → блок рибосом");
    }

    #[test]
    fn test_aggregates_degrade_rqc() {
        let mut s = RibosomeState::pristine();
        update_ribosome_state(&mut s, 1.0, 1.0, 1.0, &p());
        assert!(s.ribosome_quality <= 0.62,
            "Агрегаты → RQC ухудшается");
    }

    #[test]
    fn test_rqc_floor() {
        let mut s = RibosomeState::pristine();
        update_ribosome_state(&mut s, 0.0, 0.0, 2.0, &p());
        assert!(s.ribosome_quality >= 0.20,
            "RQC не падает ниже 20%");
    }

    #[test]
    fn test_translation_monotone_in_energy() {
        let mut lo = RibosomeState::pristine();
        let mut hi = RibosomeState::pristine();
        update_ribosome_state(&mut lo, 0.4, 1.0, 0.0, &p());
        update_ribosome_state(&mut hi, 0.9, 1.0, 0.0, &p());
        assert!(hi.translation_rate > lo.translation_rate,
            "Монотонность: больше ATP → быстрее трансляция");
    }
}
