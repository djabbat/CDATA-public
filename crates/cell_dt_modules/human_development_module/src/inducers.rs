//! Индукторы развития и кислородная логика отщепления (CDATA)

use serde::{Deserialize, Serialize};
use rand::Rng;
use cell_dt_core::components::{CentriolarDamageState, CentriolarInducerPair};

/// Уровни морфогенеза человека
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HumanMorphogeneticLevel {
    /// Зародышевый (0–2 недели)
    Embryonic,
    /// Эмбриональный (2–8 недель)
    Fetal,
    /// Плодный (9–40 недель)
    Prenatal,
    /// Постнатальный (после рождения)
    Postnatal,
    /// Взрослый
    Adult,
    /// Старение
    Aging,
}

/// Вспомогательные функции для морфогенетических уровней
pub struct HumanInducers;

impl HumanInducers {
    /// Определить морфогенетический уровень по возрасту (в днях)
    pub fn get_morphogenetic_level(age_days: f64) -> HumanMorphogeneticLevel {
        if age_days < 14.0 {
            HumanMorphogeneticLevel::Embryonic
        } else if age_days < 56.0 {
            HumanMorphogeneticLevel::Fetal
        } else if age_days < 280.0 {
            HumanMorphogeneticLevel::Prenatal
        } else if age_days < 6570.0 {
            // 18 лет
            HumanMorphogeneticLevel::Postnatal
        } else if age_days < 18250.0 {
            // 50 лет
            HumanMorphogeneticLevel::Adult
        } else {
            HumanMorphogeneticLevel::Aging
        }
    }
}

// ---------------------------------------------------------------------------
// Кислородная логика отщепления индукторов (CDATA)
// ---------------------------------------------------------------------------

/// Вычислить уровень O₂ у центриолей из молекулярного состояния центриоли.
///
/// В норме митохондрии поглощают весь кислород до центра клетки.
/// Fallback-оценка O₂ у центросомы, когда MitochondrialModule не зарегистрирован.
///
/// Используется ТОЛЬКО как приближение — предпочтительнее `mito_shield_contribution`
/// из `MitochondrialState` (формула: fusion×0.40 + Ψm×0.35 + (1−ros_prod)×0.25).
///
/// Причинность: ROS и белковые агрегаты нарушают митопhagy → митохондрии
/// не успевают перехватить O₂ → он достигает центросомы.
/// Карбонилирование центриолей — СЛЕДСТВИЕ O₂-воздействия, а не причина слабости щита;
/// поэтому оно здесь НЕ используется.
///
/// Возвращает [0..1]: 0 = центриоли защищены, 1 = максимальное воздействие O₂.
pub fn centrosomal_oxygen_level(damage: &CentriolarDamageState) -> f32 {
    let mito_shield = (1.0
        - damage.ros_level          * 0.60  // ROS — главный деструктор митохондрий
        - damage.protein_aggregates * 0.40) // агрегаты блокируют митофагию
        .max(0.0);
    (1.0 - mito_shield).clamp(0.0, 1.0)
}

/// PTM-опосредованное истощение материнского комплекта индукторов.
///
/// Второй, независимый от O₂ механизм. Структурные ПТМ (ацетилирование,
/// карбонилирование, фосфорилирование) ослабляют связи индукторов с молекулярным
/// каркасом центриоли. Чем сильнее PTM-асимметрия мать−дочь, тем выше вероятность
/// потери индуктора материнским комплектом.
///
/// **CDATA:** это механизм ИСТОЩЕНИЯ стволовых клеток, а не нормальной
/// дифференцировки: клетку не выбор, а молекулярный износ выталкивает из
/// стволового состояния. Применяется ТОЛЬКО к матери.
///
/// Вероятность = `ptm_asymmetry × ptm_exhaustion_scale`
/// `ptm_asymmetry` = (mother_ptm_avg − daughter_ptm_avg).max(0.0)
pub fn detach_by_ptm_exhaustion(
    pair: &mut CentriolarInducerPair,
    ptm_asymmetry: f32,
    rng: &mut impl Rng,
) -> bool {
    if !pair.mother_set.has_any() { return false; }
    let scale = pair.detachment_params.ptm_exhaustion_scale;
    if scale <= 0.0 { return false; }
    let prob = (ptm_asymmetry * scale).clamp(0.0, 1.0);
    if prob > 0.0 && rng.gen::<f32>() < prob {
        pair.mother_set.detach_one();
        return true;
    }
    false
}

/// Разрушить M-IDI индукторы на материнской центриоли под действием O₂.
///
/// **CDATA (исправленный механизм):**
/// O₂ воздействует ТОЛЬКО на M-комплект (M-IDI) — через окислительное
/// повреждение структурных связей материнской центриоли. D-комплект (D-IDI)
/// высвобождается ИСКЛЮЧИТЕЛЬНО функционально при делении (см. `detach_d_idi_by_division`).
///
/// # Логика отщепления:
/// - **Оба комплекта непусты (тотипотент / плюрипотент):**
///   отщепляем только от M с вероятностью `mother_prob`.
/// - **M непуст, D уже пуст (редкий случай):**
///   отщепляем M с базовой вероятностью.
/// - **M пуст (клетка в Олиго/Унипотентном состоянии):**
///   ничего — D-IDI ждёт делительного события.
/// - **Оба пусты:**
///   ничего (апоптоз должен быть запущен выше).
///
/// Возвращает `true` если хотя бы один M-IDI был разрушен.
pub fn detach_by_oxygen(
    pair: &mut CentriolarInducerPair,
    oxygen_level: f32,
    age_years: f32,
    rng: &mut impl Rng,
) -> bool {
    if oxygen_level <= 0.0 {
        return false;
    }

    let m_has = pair.mother_set.has_any();
    let d_has = pair.daughter_set.has_any();
    let params = pair.detachment_params;

    match (m_has, d_has) {
        (true, true) => {
            // Тотипотент или Плюрипотент: O₂ разрушает ТОЛЬКО M-IDI.
            // D-IDI высвобождается только при делении — не трогаем.
            if rng.gen::<f32>() < params.mother_prob(oxygen_level, age_years) {
                pair.mother_set.detach_one();
                return true;
            }
        }
        (true, false) => {
            // D уже пуст: O₂ продолжает разрушать M-IDI с базовой вероятностью.
            let p = oxygen_level * params.base_detach_probability;
            if rng.gen::<f32>() < p {
                pair.mother_set.detach_one();
                return true;
            }
        }
        (false, _) => {
            // M пуст: D-IDI высвобождается только делением, O₂ его не затрагивает.
        }
    }

    false
}

/// Функциональное высвобождение одного D-IDI при асимметричном делении.
///
/// **CDATA (исправленный механизм):**
/// D-IDI высвобождается как молекулярный СИГНАЛ — не разрушается — в момент
/// структурного перехода «дочерняя → материнская» при синтезе прокентриоли
/// внучатой клетки. Это происходит ТОЛЬКО в уже отделившейся клетке-предшественнике
/// и ТОЛЬКО при асимметричном делении.
///
/// Одно событие деления = одно высвобождение D-IDI (детерминированно).
/// Вызывать по одному разу на каждое зарегистрированное новое асимметричное деление.
///
/// Два D-IDI-пути (определяются типом молекулы, не здесь):
/// - Соматический: сигнал → апоптоз прогениторной линии
/// - Зародышевый: сигнал → мейоз → элиминация центриоли → сингамия → de novo сброс
///
/// Возвращает `true`, если один D-IDI был высвобожден.
pub fn detach_d_idi_by_division(
    pair: &mut CentriolarInducerPair,
    rng: &mut impl Rng,
) -> bool {
    if !pair.daughter_set.has_any() { return false; }
    let prob = pair.detachment_params.base_detach_probability
        * (1.0 - pair.detachment_params.effective_mother_bias(0.0));
    // Высвобождение при делении значительно вероятнее, чем случайное O₂-разрушение:
    // масштабируем × 10 — одно деление ≈ 10× дневной O₂-дозы для D-IDI
    let release_prob = (prob * 10.0).clamp(0.0, 1.0);
    if release_prob <= 0.0 { return false; }
    if rng.gen::<f32>() < release_prob {
        pair.daughter_set.detach_one();
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::components::{CentriolarDamageState, CentriolarInducerPair, PotencyLevel};

    // --- PTM-exhaustion tests ---

    #[test]
    fn test_ptm_exhaustion_zero_asymmetry_no_detach() {
        // Нет PTM-асимметрии → мать не теряет индукторы по этому пути
        let mut pair = CentriolarInducerPair::zygote(10, 8);
        let initial_m = pair.mother_set.remaining;
        let mut rng = rand::thread_rng();
        // 1000 попыток: без асимметрии — не должно ни разу сработать при scale=0.001
        for _ in 0..1000 {
            detach_by_ptm_exhaustion(&mut pair, 0.0, &mut rng);
        }
        assert_eq!(pair.mother_set.remaining, initial_m,
            "no asymmetry → mother should not lose inducers");
    }

    #[test]
    fn test_ptm_exhaustion_zero_scale_disabled() {
        // ptm_exhaustion_scale = 0 → механизм отключён
        let mut pair = CentriolarInducerPair::zygote(10, 8);
        pair.detachment_params.ptm_exhaustion_scale = 0.0;
        let initial_m = pair.mother_set.remaining;
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            detach_by_ptm_exhaustion(&mut pair, 0.9, &mut rng);
        }
        assert_eq!(pair.mother_set.remaining, initial_m,
            "scale=0 → mechanism disabled, mother unchanged");
    }

    #[test]
    fn test_ptm_exhaustion_high_asymmetry_detaches_mother_only() {
        // Высокая PTM-асимметрия → мать теряет индукторы, дочь — нет
        let mut pair = CentriolarInducerPair::zygote(10, 8);
        pair.detachment_params.ptm_exhaustion_scale = 1.0; // 100% вероятность
        let initial_d = pair.daughter_set.remaining;
        let mut rng = rand::thread_rng();
        // При scale=1.0 и asymmetry=1.0, prob=1.0 → каждый вызов снимает один индуктор
        detach_by_ptm_exhaustion(&mut pair, 1.0, &mut rng);
        assert_eq!(pair.mother_set.remaining, 9,
            "high asymmetry → mother loses 1 inductor");
        assert_eq!(pair.daughter_set.remaining, initial_d,
            "daughter is unaffected by ptm exhaustion path");
    }

    #[test]
    fn test_ptm_exhaustion_daughter_unchanged_vs_oxygen() {
        // PTM-путь затрагивает только мать.
        // Проверяем, что дочь остаётся нетронутой при PTM-истощении.
        let mut pair = CentriolarInducerPair::zygote(10, 8);
        pair.detachment_params.ptm_exhaustion_scale = 0.5;
        let d_before = pair.daughter_set.remaining;
        let mut rng = rand::thread_rng();
        for _ in 0..500 {
            detach_by_ptm_exhaustion(&mut pair, 0.8, &mut rng);
        }
        // Дочь должна быть неизменной
        assert_eq!(pair.daughter_set.remaining, d_before,
            "ptm exhaustion path must not touch daughter set");
    }

    // --- detach_by_oxygen (исправленный: только M-IDI) ---

    #[test]
    fn test_oxygen_detaches_only_mother_when_both_present() {
        // Когда оба комплекта непусты, O₂ затрагивает ТОЛЬКО M-IDI.
        // Устанавливаем base_detach_probability = 1.0 и mother_bias = 1.0
        // → 100% вероятность разрушения M, 0% для D за один вызов.
        let mut pair = CentriolarInducerPair::zygote(10, 8);
        pair.detachment_params.base_detach_probability = 1.0;
        pair.detachment_params.mother_bias = 1.0;
        let d_before = pair.daughter_set.remaining;
        let mut rng = rand::thread_rng();
        detach_by_oxygen(&mut pair, 1.0, 0.0, &mut rng);
        assert_eq!(pair.mother_set.remaining, 9,
            "O₂ must remove exactly 1 M-IDI");
        assert_eq!(pair.daughter_set.remaining, d_before,
            "D-IDI must not be touched by O₂ when both sets present");
    }

    #[test]
    fn test_oxygen_does_not_touch_daughter_when_mother_empty() {
        // Когда M пуст (Олиго/Унипотент), O₂ не затрагивает D-IDI.
        // D-IDI ждёт делительного события — O₂ его не разрушает.
        let mut pair = CentriolarInducerPair::zygote(0, 8);
        pair.mother_set.remaining = 0;
        let d_before = pair.daughter_set.remaining;
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            detach_by_oxygen(&mut pair, 1.0, 30.0, &mut rng);
        }
        assert_eq!(pair.daughter_set.remaining, d_before,
            "O₂ must not touch D-IDI when M-set is empty");
    }

    // --- detach_d_idi_by_division ---

    #[test]
    fn test_d_idi_division_releases_daughter_only() {
        // Одно делительное событие: высвобождает D-IDI, не трогает M-IDI.
        // Устанавливаем очень высокую вероятность (base=1.0, mother_bias=0.0)
        // → release_prob = 1.0 * 1.0 * 10 → clamp(1.0).
        let mut pair = CentriolarInducerPair::zygote(10, 8);
        pair.detachment_params.base_detach_probability = 1.0;
        pair.detachment_params.mother_bias = 0.0; // вся вероятность — на D
        let m_before = pair.mother_set.remaining;
        let mut rng = rand::thread_rng();
        let released = detach_d_idi_by_division(&mut pair, &mut rng);
        assert!(released, "division event must release D-IDI at prob=1.0");
        assert_eq!(pair.daughter_set.remaining, 7,
            "D-set must lose exactly 1 inductor on division");
        assert_eq!(pair.mother_set.remaining, m_before,
            "M-set must be untouched by division-path D-IDI release");
    }

    #[test]
    fn test_d_idi_no_release_when_daughter_empty() {
        // Если D-комплект уже пуст, функция возвращает false.
        let mut pair = CentriolarInducerPair::zygote(10, 0);
        pair.daughter_set.remaining = 0;
        let mut rng = rand::thread_rng();
        let released = detach_d_idi_by_division(&mut pair, &mut rng);
        assert!(!released, "must return false when D-set is empty");
        assert_eq!(pair.mother_set.remaining, 10,
            "M-set unchanged when D-set empty");
    }

    #[test]
    fn test_morphogenetic_level() {
        assert!(matches!(
            HumanInducers::get_morphogenetic_level(7.0),
            HumanMorphogeneticLevel::Embryonic
        ));
        assert!(matches!(
            HumanInducers::get_morphogenetic_level(20000.0),
            HumanMorphogeneticLevel::Aging
        ));
    }

    #[test]
    fn test_centrosomal_oxygen_pristine() {
        let damage = CentriolarDamageState::pristine();
        // Молодая клетка: ROS=0.05, нет агрегатов → очень мало O₂ у центриолей
        let oxygen = centrosomal_oxygen_level(&damage);
        assert!(oxygen < 0.1, "pristine cell should have low centrosomal O₂, got {}", oxygen);
    }

    #[test]
    fn test_centrosomal_oxygen_damaged() {
        let mut damage = CentriolarDamageState::pristine();
        damage.ros_level = 0.8;
        damage.protein_aggregates = 0.7;
        let oxygen = centrosomal_oxygen_level(&damage);
        assert!(oxygen > 0.5, "damaged cell should have high centrosomal O₂, got {}", oxygen);
    }

    #[test]
    fn test_potency_progression() {
        let mut pair = CentriolarInducerPair::zygote(3, 2);
        assert_eq!(pair.potency_level(), PotencyLevel::Totipotent);

        pair.mother_set.detach_one();
        assert_eq!(pair.potency_level(), PotencyLevel::Pluripotent);

        pair.daughter_set.remaining = 0;
        assert_eq!(pair.potency_level(), PotencyLevel::Oligopotent);

        pair.mother_set.remaining = 1;
        assert_eq!(pair.potency_level(), PotencyLevel::Unipotent);

        pair.mother_set.remaining = 0;
        assert_eq!(pair.potency_level(), PotencyLevel::Apoptosis);
    }

    #[test]
    fn test_divide_inheritance() {
        let mut pair = CentriolarInducerPair::zygote(10, 8);
        // Симулируем частичную потерю
        pair.mother_set.remaining = 7;
        pair.daughter_set.remaining = 5;

        let (cell_a, cell_b) = pair.divide();

        // Клетка A наследует материнскую (7) + новую дочернюю (7, от матери)
        assert_eq!(cell_a.mother_set.remaining, 7);
        assert_eq!(cell_a.daughter_set.remaining, 7);
        assert_eq!(cell_a.daughter_set.inherited_count, 7); // не 8!

        // Клетка B наследует старую дочернюю (5) + новую дочернюю (5, от дочки)
        assert_eq!(cell_b.mother_set.remaining, 5);
        assert_eq!(cell_b.daughter_set.remaining, 5);
        assert_eq!(cell_b.daughter_set.inherited_count, 5); // не 8!
    }
}
