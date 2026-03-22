//! Трек G — Life-History Trade-off / Гормональный Часовой Механизм
//!
//! Теория Дж. Ткемаладзе: возраст начала половой зрелости положительно
//! коррелирует с продолжительностью жизни (r=0.78, R²=0.92 на библейских генеалогиях).
//!
//! ## Механизм (5 фаз)
//!
//! 1. **Препубертат** — HPG-ось молчит; нет гормонального фона → нет защиты центриолей,
//!    но и нет ускорения Life-History Trade-off.
//! 2. **Пубертат → пик** — эстроген/тестостерон нарастают; `hormonal_protection` растёт
//!    до 1.0 → ros_level снижается до 20%; `life_history_factor` нарастает до 1.20
//!    (тело «инвестирует» в репродукцию за счёт репарации центриолей).
//! 3. **Плато фертильности** (~25–36 лет) — полная гормональная защита.
//! 4. **Перименопауза** — постепенное снижение гормонального фона.
//! 5. **Постменопауза / Андропауза** — hormone_level = 0; защита снята;
//!    `life_history_factor = 1.20` сохраняется (trade-off уже произошёл необратимо).
//!
//! ## Связь с CDATA
//!
//! - `ros_reduction()` вычитается из `CentriolarDamageState.ros_level`
//! - `life_history_factor` умножает `base_detach_probability` (скорость потери индукторов)
//! - Длинное `puberty_age_years` → меньший life_history_factor в молодости →
//!   меньше накопленных повреждений → бо́льшая продолжительность жизни (r=0.78)

use cell_dt_core::components::{HormonalFertilityState, ReproductivePhase};

/// Возраст пика гормонального фона (лет)
const PEAK_FERTILITY_AGE: f32 = 25.0;
/// Сколько лет до менопаузы начинается перименопаузальное снижение
const PERIMENOPAUSE_WINDOW: f32 = 15.0;

/// Обновить `HormonalFertilityState` для заданного возраста.
///
/// Вызывается каждый шаг в `HumanDevelopmentModule::step()`.
pub fn update_hormonal_fertility(state: &mut HormonalFertilityState, age_years: f32) {
    let puberty = state.puberty_age_years;
    let menopause = state.menopause_age_years;
    let peri_start = menopause - PERIMENOPAUSE_WINDOW;
    // Пик не раньше чем через 3 года после пубертата
    let peak = PEAK_FERTILITY_AGE.max(puberty + 3.0);

    if age_years < puberty {
        // Препубертатный период: нет гормонов, нет торговли
        state.phase = ReproductivePhase::Prepubertal;
        state.hormone_level = 0.0;
        state.hormonal_protection = 0.0;
        state.life_history_factor = 1.0;

    } else if age_years < peak {
        // Нарастание: пубертат → пик фертильности
        state.phase = ReproductivePhase::Fertile;
        let rise = ((age_years - puberty) / (peak - puberty)).clamp(0.0, 1.0);
        state.hormone_level = rise;
        state.hormonal_protection = rise;
        // Life-History trade-off нарастает вместе с гормонами
        state.life_history_factor = 1.0 + 0.20 * rise;

    } else if age_years < peri_start {
        // Плато фертильности
        state.phase = ReproductivePhase::Fertile;
        state.hormone_level = 1.0;
        state.hormonal_protection = 1.0;
        state.life_history_factor = 1.20;

    } else if age_years < menopause {
        // Перименопауза: постепенное снижение
        state.phase = ReproductivePhase::Perimenopausal;
        let decline = ((age_years - peri_start) / (menopause - peri_start)).clamp(0.0, 1.0);
        state.hormone_level = (1.0 - decline).max(0.0);
        state.hormonal_protection = state.hormone_level;
        state.life_history_factor = 1.20; // сохраняется

    } else {
        // Постменопауза / Андропауза
        state.phase = ReproductivePhase::Postmenopausal;
        state.hormone_level = 0.0;
        state.hormonal_protection = 0.0;
        // Trade-off сохраняется необратимо: тело уже «заплатило» за репродукцию
        state.life_history_factor = 1.20;
    }
}

/// Аддитивная поправка к ROS (отрицательная = снижение, положительная = рост).
///
/// При полной гормональной защите (hormone_level=1.0): −0.20 (снижение на 20%).
/// После менопаузы: 0.0 (нет защиты).
pub fn hormonal_ros_modifier(state: &HormonalFertilityState) -> f32 {
    -state.ros_reduction()
}

/// Мультипликатор `base_detach_probability` из Life-History Trade-off.
///
/// Возвращает `life_history_factor` [1.0..1.20].
/// Умножается на базовую вероятность отщепления индукторов каждый шаг.
pub fn life_history_detach_multiplier(state: &HormonalFertilityState) -> f32 {
    state.life_history_factor
}

// ---------------------------------------------------------------------------
// Тесты
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(puberty: f32, menopause: f32) -> HormonalFertilityState {
        HormonalFertilityState {
            puberty_age_years: puberty,
            menopause_age_years: menopause,
            ..HormonalFertilityState::default()
        }
    }

    #[test]
    fn test_prepubertal_at_birth() {
        let mut s = make_state(14.0, 51.0);
        update_hormonal_fertility(&mut s, 5.0);
        assert_eq!(s.phase, ReproductivePhase::Prepubertal);
        assert_eq!(s.hormone_level, 0.0);
        assert_eq!(s.life_history_factor, 1.0);
    }

    #[test]
    fn test_fertile_at_peak() {
        let mut s = make_state(14.0, 51.0);
        update_hormonal_fertility(&mut s, 25.0);
        assert_eq!(s.phase, ReproductivePhase::Fertile);
        assert!((s.hormone_level - 1.0).abs() < 0.01,
            "Ожидается hormone_level=1.0, получено {}", s.hormone_level);
        assert!((s.life_history_factor - 1.20).abs() < 0.02,
            "Ожидается life_history_factor≈1.20, получено {}", s.life_history_factor);
    }

    #[test]
    fn test_plateau_before_peri() {
        let mut s = make_state(14.0, 51.0);
        update_hormonal_fertility(&mut s, 32.0); // 25 < 32 < 36 = peri_start
        assert_eq!(s.phase, ReproductivePhase::Fertile);
        assert!((s.hormone_level - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_perimenopausal_decline() {
        let mut s = make_state(14.0, 51.0);
        update_hormonal_fertility(&mut s, 43.5); // mid perimenopause: peri_start=36, menop=51
        assert_eq!(s.phase, ReproductivePhase::Perimenopausal);
        assert!(s.hormone_level > 0.0 && s.hormone_level < 1.0,
            "Гормон должен снижаться: {}", s.hormone_level);
    }

    #[test]
    fn test_postmenopausal() {
        let mut s = make_state(14.0, 51.0);
        update_hormonal_fertility(&mut s, 60.0);
        assert_eq!(s.phase, ReproductivePhase::Postmenopausal);
        assert_eq!(s.hormone_level, 0.0);
        assert_eq!(s.hormonal_protection, 0.0);
        assert!((s.life_history_factor - 1.20).abs() < 0.01,
            "Life-History factor сохраняется: {}", s.life_history_factor);
    }

    #[test]
    fn test_ros_reduction_at_peak() {
        let mut s = make_state(14.0, 51.0);
        update_hormonal_fertility(&mut s, 25.0);
        let modifier = hormonal_ros_modifier(&s);
        assert!((modifier - (-0.20)).abs() < 0.01,
            "Ожидается снижение ROS на 0.20, получено {}", modifier);
    }

    #[test]
    fn test_ros_reduction_postmenopausal() {
        let mut s = make_state(14.0, 51.0);
        update_hormonal_fertility(&mut s, 60.0);
        assert!((hormonal_ros_modifier(&s) - 0.0).abs() < 0.001,
            "Нет защиты после менопаузы");
    }

    #[test]
    fn test_late_puberty_lower_lhf() {
        // Life-History Trade-off: позднее начало пубертата → меньший накопленный trade-off
        // На возрасте 20 лет: у «ранней» (puberty=10) LHF=1.20; у «поздней» (puberty=18) LHF<1.20
        let mut early = make_state(10.0, 51.0);
        let mut late = make_state(18.0, 51.0);
        update_hormonal_fertility(&mut early, 20.0);
        update_hormonal_fertility(&mut late, 20.0);
        assert!(early.life_history_factor > late.life_history_factor,
            "Ранний пубертат → больший trade-off: {:.3} > {:.3}",
            early.life_history_factor, late.life_history_factor);
    }

    #[test]
    fn test_male_params() {
        let mut s = HormonalFertilityState::male();
        update_hormonal_fertility(&mut s, 80.0);
        assert_eq!(s.phase, ReproductivePhase::Postmenopausal);
    }
}
