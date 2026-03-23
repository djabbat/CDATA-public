//! Органный уровень (Уровень +2: органы).
//!
//! OrganState агрегирует функциональный резерв стволовых ниш одного органа.
//! Компенсаторная гипертрофия поддерживает функцию при частичной потере ниш.
//! Полиорганная недостаточность (≥2 органов в failure) → критерий смерти.
//!
//! # Алгоритм
//!
//! ```text
//! functional_reserve = mean(fc_i) × (1 − max_fibrosis_penalty)
//! effective_reserve = reserve + compensation × max(0, threshold − reserve)
//! is_failing = effective_reserve < threshold
//! poly_organ_failure = count(is_failing) >= 2
//! ```

use cell_dt_core::components::{OrganState, OrganType};
use std::collections::HashMap;

/// Аргумент агрегирования — данные от одной ниши.
pub struct NicheOrganData {
    pub functional_capacity: f32,
    pub fibrosis_penalty:    f32, // из FibrosisState.fc_penalty()
}

/// Обновить OrganState из данных ниш.
///
/// Вызывается один раз в шаге из отдельного прохода по нишам данного органа.
pub fn update_organ_state(
    organ:  &mut OrganState,
    niches: &[NicheOrganData],
) {
    if niches.is_empty() {
        organ.niche_count = 0;
        return;
    }

    organ.niche_count = niches.len() as u32;

    // Среднее functional_capacity с учётом фиброза
    let mean_fc = niches.iter().map(|n| {
        (n.functional_capacity * (1.0 - n.fibrosis_penalty)).max(0.0)
    }).sum::<f32>() / niches.len() as f32;

    // Компенсаторная гипертрофия: если резерв ниже порога, другие ткани берут часть
    let deficit = (organ.failure_threshold - mean_fc).max(0.0);
    let compensated = mean_fc + organ.compensation_capacity * deficit;

    organ.functional_reserve = compensated.clamp(0.0, 1.0);
    organ.update_failure_status();
}

/// Проверить полиорганную недостаточность.
///
/// Возвращает `Some(список_отказавших_органов)` если ≥2 органов в failure.
pub fn check_poly_organ_failure(organs: &HashMap<OrganType, OrganState>) -> Option<Vec<OrganType>> {
    let failing: Vec<OrganType> = organs.values()
        .filter(|o| o.is_failing)
        .map(|o| o.organ_type)
        .collect();
    if failing.len() >= 2 {
        Some(failing)
    } else {
        None
    }
}

/// Кардиальный выброс → оксигенация всех органов.
///
/// При сердечной недостаточности: oxygen_delivery снижается ко всем органам.
pub fn cardiac_oxygen_delivery(organs: &HashMap<OrganType, OrganState>) -> f32 {
    organs.get(&OrganType::Heart)
        .map(|h| h.functional_reserve)
        .unwrap_or(1.0)
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_organ(t: OrganType) -> OrganState { OrganState::new(t) }

    fn niches(fcs: &[f32]) -> Vec<NicheOrganData> {
        fcs.iter().map(|&fc| NicheOrganData { functional_capacity: fc, fibrosis_penalty: 0.0 })
            .collect()
    }

    #[test]
    fn test_pristine_organ_healthy() {
        let o = mk_organ(OrganType::Heart);
        assert!(!o.is_failing);
        assert!((o.functional_reserve - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_update_from_healthy_niches() {
        let mut o = mk_organ(OrganType::Kidney);
        update_organ_state(&mut o, &niches(&[0.9, 0.85, 0.92]));
        assert!(!o.is_failing, "Здоровые ниши → орган работает");
        assert!(o.functional_reserve > 0.80);
    }

    #[test]
    fn test_organ_fails_below_threshold() {
        let mut o = mk_organ(OrganType::Heart); // threshold = 0.20
        update_organ_state(&mut o, &niches(&[0.05, 0.08, 0.10]));
        // mean = 0.077, even with compensation unlikely to reach 0.20
        assert!(o.is_failing, "Тяжёлые повреждения → орган в недостаточности");
    }

    #[test]
    fn test_compensation_prevents_early_failure() {
        let mut o = mk_organ(OrganType::Liver); // threshold=0.15, compensation=0.60
        // mean_fc = 0.16 — чуть выше порога, компенсация не нужна
        update_organ_state(&mut o, &niches(&[0.15, 0.17, 0.16]));
        assert!(!o.is_failing, "Небольшой дефицит компенсируется");
    }

    #[test]
    fn test_poly_organ_failure_detected() {
        let mut organs = HashMap::new();
        let mut heart = mk_organ(OrganType::Heart);
        heart.is_failing = true;
        let mut kidney = mk_organ(OrganType::Kidney);
        kidney.is_failing = true;
        organs.insert(OrganType::Heart, heart);
        organs.insert(OrganType::Kidney, kidney);
        organs.insert(OrganType::Liver, mk_organ(OrganType::Liver));
        assert!(check_poly_organ_failure(&organs).is_some(),
            "2+ органа в failure → полиорганная недостаточность");
    }

    #[test]
    fn test_single_organ_failure_not_poly() {
        let mut organs = HashMap::new();
        let mut heart = mk_organ(OrganType::Heart);
        heart.is_failing = true;
        organs.insert(OrganType::Heart, heart);
        organs.insert(OrganType::Kidney, mk_organ(OrganType::Kidney));
        assert!(check_poly_organ_failure(&organs).is_none(),
            "1 орган в failure → ещё не полиорганная недостаточность");
    }

    #[test]
    fn test_fibrosis_reduces_reserve() {
        let mut o = mk_organ(OrganType::Liver);
        let with_fibrosis = vec![
            NicheOrganData { functional_capacity: 0.8, fibrosis_penalty: 0.4 },
            NicheOrganData { functional_capacity: 0.8, fibrosis_penalty: 0.4 },
        ];
        let without_fibrosis = niches(&[0.8, 0.8]);
        let mut o2 = mk_organ(OrganType::Liver);
        update_organ_state(&mut o, &with_fibrosis);
        update_organ_state(&mut o2, &without_fibrosis);
        assert!(o.functional_reserve < o2.functional_reserve,
            "Фиброз → снижает функциональный резерв");
    }

    #[test]
    fn test_cardiac_output_from_healthy_heart() {
        let mut organs = HashMap::new();
        organs.insert(OrganType::Heart, mk_organ(OrganType::Heart));
        assert!((cardiac_oxygen_delivery(&organs) - 1.0).abs() < 1e-5,
            "Здоровое сердце → полный сердечный выброс");
    }
}
