//! Параметры и механизм накопления повреждений центриоли (CDATA)

use serde::{Deserialize, Serialize};

/// Параметры накопления молекулярных повреждений центриоли
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageParams {
    // --- Базовые скорости повреждения (в год) ---

    /// Базовая скорость карбонилирования белков (SAS-6, CEP135) через ROS
    pub base_ros_damage_rate: f32,
    /// Скорость нарастания гиперацетилирования (снижение HDAC6/SIRT2)
    pub acetylation_rate: f32,
    /// Скорость накопления агрегатов (CPAP, CEP290)
    pub aggregation_rate: f32,
    /// Скорость нарушения фосфорилирования (PLK4/NEK2/PP1 дисбаланс)
    pub phospho_dysregulation_rate: f32,

    // --- Потеря дистальных придатков (в год) ---
    pub cep164_loss_rate: f32,
    pub cep89_loss_rate:  f32,
    pub ninein_loss_rate: f32,
    pub cep170_loss_rate: f32,

    // --- Репарация придатков (P5) ---
    // По умолчанию 0.0 — необратимость (обратная совместимость).
    // При значениях > 0 включается активная репарация, усиленная митофагией.
    // Источник: USP21/TTBK2-зависимое восстановление CEP164 при снятии оксидативного стресса
    // (Klinger et al., 2014).
    /// Базовая скорость репарации CEP164 [/год]. 0.0 = выключена.
    pub cep164_repair_rate: f32,
    /// Базовая скорость репарации CEP89 [/год]. 0.0 = выключена.
    pub cep89_repair_rate: f32,
    /// Базовая скорость репарации Ninein [/год]. 0.0 = выключена.
    pub ninein_repair_rate: f32,
    /// Базовая скорость репарации CEP170 [/год]. 0.0 = выключена.
    pub cep170_repair_rate: f32,
    /// Усиление репарации от митофагии [0..2].
    /// 0.0 = митофагия не влияет на репарацию придатков;
    /// 1.0 = mitophagy_flux умножает базовую скорость репарации.
    /// Реализует связь PINK1/Parkin → удаление повреждённых митохондрий →
    /// снижение локального ROS → восстановление белков центриолярных придатков.
    pub appendage_repair_mitophagy_coupling: f32,

    // --- Чувствительность придатков к OH· (AppendageProteinState, P21) ---
    //
    // OH· (гидроксил-радикал) образуется через Fenton: Fe²⁺ + H₂O₂ → OH·.
    // Относительная чувствительность определяется доступностью Met/Cys-остатков
    // и локализацией белка в дистальном vs субдистальном аппендаже.
    //
    // Источник: Stadtman & Levine (2003) Ann. NY Acad. Sci. 1012:17-24
    // (карбонилирование коiled-coil-доменов OH·).

    /// Чувствительность CEP164 к OH· [безразмерный]. Дефолт: 1.50.
    /// Объяснение: длинный coiled-coil + Met228/Cys-домен → первичная мишень OH·.
    pub cep164_oh_sensitivity: f32,
    /// Чувствительность CEP89 к OH·. Дефолт: 1.00.
    pub cep89_oh_sensitivity: f32,
    /// Чувствительность Ninein к OH·. Дефолт: 0.75.
    /// Объяснение: субдистальная локализация → меньше доступ к Fenton-ROS.
    pub ninein_oh_sensitivity: f32,
    /// Чувствительность CEP170 к OH·. Дефолт: 0.55.
    /// Объяснение: наиболее защищён; Met-бедный N-конец.
    pub cep170_oh_sensitivity: f32,

    // --- Параметры петли обратной связи ---

    /// Коэффициент: повреждение центриоли → рост ROS
    /// (нарушение митофагии → дисфункция митохондрий → больше ROS)
    pub ros_feedback_coefficient: f32,

    /// Возраст (в годах), с которого активируется SASP (inflammaging)
    pub sasp_onset_age: f32,

    /// Порог суммарного повреждения для входа в сенесценцию.
    /// Синхронизируется в `CentriolarDamageState::senescence_threshold`
    /// через `accumulate_damage()` каждый шаг.
    pub senescence_threshold: f32,

    /// Максимальный множитель повреждения в среднем возрасте (антагонистическая плейотропия).
    /// Применяется через сигмоидный переход, не ступенькой.
    /// Дефолт 1.6 — активируется плавно вокруг `midlife_transition_center`.
    pub midlife_damage_multiplier: f32,

    // --- Сигмоидный переход среднего возраста (P4) ---

    /// Центр логистического перехода множителя [лет]. Дефолт: 42.5.
    /// Физически — середина гормональной перестройки (менопауза/андропауза).
    pub midlife_transition_center: f32,
    /// Полуширина логистического перехода [лет]. Дефолт: 7.5.
    /// Меньше = резче переход; больше = плавнее.
    pub midlife_transition_width: f32,

    // --- Стохастический шум (P3) ---

    /// Масштаб Ланжевен-шума для молекулярных повреждений.
    /// 0.0 (дефолт) = детерминированный режим (обратная совместимость).
    /// Рекомендуемое значение для популяционных симуляций: 0.1.
    /// Шум применяется в `HumanDevelopmentModule::step()` ПОСЛЕ вызова
    /// `accumulate_damage()`, используя seeded RNG модуля.
    pub noise_scale: f32,
}

impl Default for DamageParams {
    fn default() -> Self {
        Self {
            // Калибровка для шага dt = 1 день (1/365.25 лет):
            // при этих значениях is_senescent (total_damage_score > 0.75)
            // наступает ~78 лет (норма, с сигмоидным midlife_damage_multiplier ×1.6
            // и петлёй обратной связи ROS).
            //
            // Коэффициент ×4.2 — эмпирическое масштабирование от первичных биохимических оценок:
            // Первичная оценка базируется на скорости окисления SAS-6 через ROS (H₂O₂-опосредованное
            // карбонилирование): ~0.0018/год при физиологическом уровне митохондриальных ROS.
            // Источник первичной оценки: Bratic & Larsson (2013) "The role of mitochondria in aging",
            // J Cell Biol 198(1):7-19; данные Chance et al. (1979) о скорости утечки электронов
            // в комплексе I (≈2% от потока O₂ → H₂O₂).
            // Множитель ×4.2 идентифицирован путём калибровки: минимальное значение, при котором
            // is_senescent наступает в [70..85] лет в детерминированном режиме (seed=42).
            // SA-анализ (P2, 2026-03-10): midlife_damage_multiplier — наиболее чувствительный параметр
            // (Δlifespan ≈ −13yr при +50%, +38yr при −50%), подтверждая важность антагонистической
            // плейотропии как механизма CDATA.
            //
            // Молекулярные скорости ×4.2 от первичных биохимических оценок:
            base_ros_damage_rate:       0.0076,   // карбонилирование SAS-6 / CEP135 (×ROS)
            acetylation_rate:           0.0059,   // гиперацетилирование α-тубулина
            aggregation_rate:           0.0059,   // агрегаты CPAP / CEP290
            phospho_dysregulation_rate: 0.0042,   // дисбаланс PLK4 / NEK2 / PP1

            // Потеря дистальных придатков (×4.2):
            cep164_loss_rate: 0.0113,  // инициация ресничек (CEP164)
            cep89_loss_rate:  0.0084,  // CEP89
            ninein_loss_rate: 0.0084,  // Ninein (субдистальные придатки)
            cep170_loss_rate: 0.0067,  // CEP170

            // Репарация выключена по умолчанию (обратная совместимость)
            cep164_repair_rate:                  0.0,
            cep89_repair_rate:                   0.0,
            ninein_repair_rate:                  0.0,
            cep170_repair_rate:                  0.0,
            appendage_repair_mitophagy_coupling: 0.0,

            // OH·-чувствительность (P21): калибровка по относительной скорости окисления
            // coiled-coil-доменов; CEP164 = эталон 1.50 (наибольшая Met/Cys-плотность).
            cep164_oh_sensitivity: 1.50,
            cep89_oh_sensitivity:  1.00,
            ninein_oh_sensitivity: 0.75,
            cep170_oh_sensitivity: 0.55,

            ros_feedback_coefficient:   0.12,
            sasp_onset_age:             45.0,
            senescence_threshold:       0.75,
            midlife_damage_multiplier:  1.6,

            // Сигмоидный переход: центр 42.5, ширина 7.5 лет
            midlife_transition_center:  42.5,
            midlife_transition_width:   7.5,

            // Детерминированный режим по умолчанию
            noise_scale: 0.0,
        }
    }
}

impl DamageParams {
    /// Стандартное нормальное старение (алиас для `Default::default()`).
    /// Используйте вместо `DamageParams::default()` для явности в интеграционных тестах.
    pub fn normal_aging() -> Self {
        Self::default()
    }

    /// Вариант "ускоренного старения" (прогерия) — все скорости ×5
    pub fn progeria() -> Self {
        let mut p = Self::default();
        p.base_ros_damage_rate       *= 5.0;
        p.acetylation_rate           *= 5.0;
        p.aggregation_rate           *= 5.0;
        p.phospho_dysregulation_rate *= 5.0;
        p.cep164_loss_rate           *= 5.0;
        p.cep89_loss_rate            *= 5.0;
        p.ninein_loss_rate           *= 5.0;
        p.cep170_loss_rate           *= 5.0;
        p.midlife_damage_multiplier   = 3.0;
        p
    }

    /// Вариант "замедленного старения" (долгожители) — все скорости ×0.6
    pub fn longevity() -> Self {
        let mut p = Self::default();
        p.base_ros_damage_rate       *= 0.6;
        p.acetylation_rate           *= 0.6;
        p.aggregation_rate           *= 0.6;
        p.phospho_dysregulation_rate *= 0.6;
        p.cep164_loss_rate           *= 0.6;
        p.cep89_loss_rate            *= 0.6;
        p.ninein_loss_rate           *= 0.6;
        p.cep170_loss_rate           *= 0.6;
        p.midlife_damage_multiplier   = 1.2;
        p
    }

    /// Вариант "антиоксидантная защита" (P5) — сниженные молекулярные скорости + активная репарация.
    ///
    /// Моделирует эффект длительного приёма антиоксидантов и активаторов митофагии:
    /// - ROS-повреждение ×0.5 (аналог NAC/MitoQ-терапии)
    /// - Базовая репарация придатков включена (аналог TTBK2/USP21-активаторов)
    /// - Связь митофагии с репарацией активирована (`appendage_repair_mitophagy_coupling = 1.0`)
    pub fn antioxidant() -> Self {
        let mut p = Self::default();
        p.base_ros_damage_rate       *= 0.5;  // снижение оксидативного повреждения
        p.aggregation_rate           *= 0.7;  // протеостаз частично улучшен
        p.cep164_repair_rate         = 0.003; // базальная репарация ~×0.25 от скорости потери
        p.cep89_repair_rate          = 0.002;
        p.ninein_repair_rate         = 0.002;
        p.cep170_repair_rate         = 0.0015;
        p.appendage_repair_mitophagy_coupling = 1.0; // митофагия удваивает репарацию
        p
    }

    /// Вычислить возрастной множитель через логистическую функцию (P4).
    ///
    /// Заменяет ступенчатую функцию `if age > 40 { multiplier } else { 1.0 }`.
    /// Логистика: плавный переход от 1.0 к `midlife_damage_multiplier` в диапазоне
    /// `center ± width` лет. При `width → 0` воспроизводит поведение шаговой функции.
    ///
    /// # Пример (дефолт: center=42.5, width=7.5, multiplier=1.6)
    /// - age=30: mult ≈ 1.09  (эффект начинается)
    /// - age=42.5: mult = 1.30 (середина перехода)
    /// - age=55: mult ≈ 1.54  (почти максимум)
    /// - age=70: mult ≈ 1.59  (практически 1.6)
    #[inline]
    pub fn age_multiplier(&self, age_years: f32) -> f32 {
        let sigmoid = 1.0
            / (1.0
                + (-(age_years - self.midlife_transition_center)
                    / self.midlife_transition_width)
                    .exp());
        1.0 + (self.midlife_damage_multiplier - 1.0) * sigmoid
    }
}

/// Обновить состояние повреждений центриоли за один временной шаг (dt_years).
///
/// # Параметры
/// * `ros_level_boost` — внешний аддитивный буст ROS от воспаления (InflammagingState).
///   Применяется ДО вычисления `protein_carbonylation`, обеспечивая корректную петлю:
///   `inflammaging → ros_level↑ → protein_carbonylation↑`.
///   Значение 0.0 означает отсутствие буста (нормальный режим).
pub fn accumulate_damage(
    damage: &mut cell_dt_core::components::CentriolarDamageState,
    params: &DamageParams,
    age_years: f32,
    dt_years: f32,
    ros_level_boost: f32,
) {
    // Синхронизировать senescence_threshold из DamageParams
    damage.senescence_threshold = params.senescence_threshold;

    // P4: Сигмоидный возрастной множитель (плавный переход, без разрыва в 40 лет)
    let age_multiplier = params.age_multiplier(age_years);

    // Петля обратной связи: накопленный ущерб усиливает ROS.
    // ros_level_boost — внешний вклад от inflammaging (межшаговая петля).
    let base_ros = 0.05 + age_years * 0.005;
    let intrinsic_ros = base_ros
        + params.ros_feedback_coefficient * damage.total_damage_score();
    // Применяем буст ПЕРЕД расчётом повреждений — так ros_boost влияет на carbonylation
    damage.ros_level = (intrinsic_ros + ros_level_boost).min(1.0);

    let ros_boost = 1.0 + params.ros_feedback_coefficient * damage.total_damage_score();
    let effective_dt = dt_years * age_multiplier * ros_boost;

    // Молекулярные повреждения (используют обновлённый ros_level)
    damage.protein_carbonylation = (damage.protein_carbonylation
        + params.base_ros_damage_rate * damage.ros_level * effective_dt).min(1.0);

    damage.tubulin_hyperacetylation = (damage.tubulin_hyperacetylation
        + params.acetylation_rate * effective_dt).min(1.0);

    damage.protein_aggregates = (damage.protein_aggregates
        + params.aggregation_rate * effective_dt).min(1.0);

    damage.phosphorylation_dysregulation = (damage.phosphorylation_dysregulation
        + params.phospho_dysregulation_rate * effective_dt).min(1.0);

    // Потеря придатков (необратима при repair_rate = 0.0)
    damage.cep164_integrity = (damage.cep164_integrity
        - params.cep164_loss_rate * effective_dt).max(0.0);
    damage.cep89_integrity  = (damage.cep89_integrity
        - params.cep89_loss_rate  * effective_dt).max(0.0);
    damage.ninein_integrity = (damage.ninein_integrity
        - params.ninein_loss_rate * effective_dt).max(0.0);
    damage.cep170_integrity = (damage.cep170_integrity
        - params.cep170_loss_rate * effective_dt).max(0.0);

    // Пересчёт производных метрик (spindle_fidelity, ciliary_function, is_senescent)
    damage.update_functional_metrics();
}

/// Применить репарацию дистальных придатков центриоли (P5).
///
/// Вызывается из `HumanDevelopmentModule::step()` ПОСЛЕ `accumulate_damage()`.
/// При `params.cep164_repair_rate == 0.0` — no-op (обратная совместимость).
///
/// # Параметры
/// * `mitophagy_flux` — текущий поток митофагии из `MitochondrialState` [0..1].
///   При `appendage_repair_mitophagy_coupling > 0` увеличивает эффективную скорость репарации:
///   `effective_repair = base_repair * (1 + coupling * mitophagy_flux)`.
///   Если `MitochondrialState` отсутствует — передавать 0.0.
pub fn apply_appendage_repair(
    damage: &mut cell_dt_core::components::CentriolarDamageState,
    params: &DamageParams,
    mitophagy_flux: f32,
    dt_years: f32,
) {
    // Быстрый выход если репарация выключена (дефолт — все repair_rate = 0.0)
    if params.cep164_repair_rate + params.cep89_repair_rate
        + params.ninein_repair_rate + params.cep170_repair_rate == 0.0
    {
        return;
    }

    // Коэффициент усиления от митофагии:
    // healthy mitophagy → удаление повреждённых митохондрий → снижение локального ROS
    // → условия для восстановления структурных белков придатков
    let mito_factor = 1.0 + params.appendage_repair_mitophagy_coupling * mitophagy_flux;
    let repair_dt = dt_years * mito_factor;

    damage.cep164_integrity = (damage.cep164_integrity
        + params.cep164_repair_rate * repair_dt).min(1.0);
    damage.cep89_integrity  = (damage.cep89_integrity
        + params.cep89_repair_rate  * repair_dt).min(1.0);
    damage.ninein_integrity = (damage.ninein_integrity
        + params.ninein_repair_rate * repair_dt).min(1.0);
    damage.cep170_integrity = (damage.cep170_integrity
        + params.cep170_repair_rate * repair_dt).min(1.0);

    // Пересчёт производных метрик после репарации
    damage.update_functional_metrics();
}

// ---------------------------------------------------------------------------
// P21: AppendageProteinState — независимая кинетика белков придатков
// ---------------------------------------------------------------------------

/// Обновить `AppendageProteinState` за один шаг (P21).
///
/// Каждый белок теряет целостность пропорционально:
///   1. Базовой скорости потери (из `DamageParams::cepXXX_loss_rate`).
///   2. Уровню OH· (гидроксил-радикал): `oh_level = ros_level²`
///      (квадратичная связь: высокий H₂O₂ → Fenton → OH·).
///   3. Чувствительности белка к OH· (`cepXXX_oh_sensitivity`).
///   4. Возрастному множителю (общий `effective_dt`).
///
/// После обновления пересчитывается CAII.
///
/// # Синхронизация
/// Вызывать ПОСЛЕ `accumulate_damage()`. После вызова синхронизировать
/// `CentriolarDamageState.cepXXX_integrity` из `AppendageProteinState.cepXXX`
/// и вызвать `damage.update_functional_metrics()`.
///
/// # Параметры
/// * `oh_level` — текущий уровень OH· [0..1].
///   Рекомендуемое вычисление: `ros_level.powi(2)` (Fenton-зависимость).
///   При отсутствии данных — передать `damage.ros_level.powi(2)`.
pub fn accumulate_appendage_damage(
    appendage: &mut cell_dt_core::components::AppendageProteinState,
    params: &DamageParams,
    oh_level: f32,
    age_years: f32,
    dt_years: f32,
) {
    let age_mult = params.age_multiplier(age_years);
    let effective_dt = dt_years * age_mult;

    // Потеря = базовая + OH·-компонент (независимая чувствительность к Fenton)
    appendage.cep164 = (appendage.cep164
        - (params.cep164_loss_rate
            + oh_level * params.cep164_oh_sensitivity * params.base_ros_damage_rate)
            * effective_dt
    ).max(0.0);

    appendage.cep89 = (appendage.cep89
        - (params.cep89_loss_rate
            + oh_level * params.cep89_oh_sensitivity * params.base_ros_damage_rate)
            * effective_dt
    ).max(0.0);

    appendage.ninein = (appendage.ninein
        - (params.ninein_loss_rate
            + oh_level * params.ninein_oh_sensitivity * params.base_ros_damage_rate)
            * effective_dt
    ).max(0.0);

    appendage.cep170 = (appendage.cep170
        - (params.cep170_loss_rate
            + oh_level * params.cep170_oh_sensitivity * params.base_ros_damage_rate)
            * effective_dt
    ).max(0.0);

    appendage.update_caii();
}

/// Применить репарацию к `AppendageProteinState` (P21).
///
/// Аналог `apply_appendage_repair`, но оперирует на отдельном компоненте.
/// При `params.cep164_repair_rate == 0.0` — no-op.
pub fn apply_appendage_protein_repair(
    appendage: &mut cell_dt_core::components::AppendageProteinState,
    params: &DamageParams,
    mitophagy_flux: f32,
    dt_years: f32,
) {
    if params.cep164_repair_rate + params.cep89_repair_rate
        + params.ninein_repair_rate + params.cep170_repair_rate == 0.0
    {
        return;
    }

    let mito_factor = 1.0 + params.appendage_repair_mitophagy_coupling * mitophagy_flux;
    let repair_dt = dt_years * mito_factor;

    appendage.cep164 = (appendage.cep164 + params.cep164_repair_rate * repair_dt).min(1.0);
    appendage.cep89  = (appendage.cep89  + params.cep89_repair_rate  * repair_dt).min(1.0);
    appendage.ninein = (appendage.ninein + params.ninein_repair_rate  * repair_dt).min(1.0);
    appendage.cep170 = (appendage.cep170 + params.cep170_repair_rate * repair_dt).min(1.0);

    appendage.update_caii();
}

// ---------------------------------------------------------------------------
// Тесты
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::components::{AppendageProteinState, CentriolarDamageState};

    const DT: f32 = 1.0 / 365.25; // один день

    // --- P4: Сигмоидный возрастной множитель ---

    #[test]
    fn test_age_multiplier_is_smooth_at_40() {
        let p = DamageParams::default();
        // Множитель должен быть непрерывным (нет разрыва в 40 лет)
        let m39 = p.age_multiplier(39.9);
        let m40 = p.age_multiplier(40.0);
        let m41 = p.age_multiplier(40.1);
        // Разница между соседними точками должна быть мала (< 0.01)
        assert!((m40 - m39).abs() < 0.01,
            "разрыв в 40 лет: m39={:.4}, m40={:.4}", m39, m40);
        assert!((m41 - m40).abs() < 0.01,
            "разрыв в 40 лет: m40={:.4}, m41={:.4}", m40, m41);
    }

    #[test]
    fn test_age_multiplier_range() {
        let p = DamageParams::default();
        // Множитель всегда в диапазоне [1.0, midlife_damage_multiplier]
        for age in [0.0, 10.0, 30.0, 40.0, 50.0, 65.0, 80.0, 100.0] {
            let m = p.age_multiplier(age);
            assert!(m >= 1.0,
                "multiplier < 1.0 при age={}: {:.4}", age, m);
            assert!(m <= p.midlife_damage_multiplier + 0.001,
                "multiplier > max при age={}: {:.4}", age, m);
        }
    }

    #[test]
    fn test_age_multiplier_center_is_half_way() {
        let p = DamageParams::default();
        // В точке центра сигмоид = 0.5 → multiplier = 1.0 + 0.5*(max-1.0)
        let m_center = p.age_multiplier(p.midlife_transition_center);
        let expected = 1.0 + 0.5 * (p.midlife_damage_multiplier - 1.0);
        assert!((m_center - expected).abs() < 0.001,
            "в центре сигмоид должен быть на полпути: got={:.4}, expected={:.4}",
            m_center, expected);
    }

    #[test]
    fn test_age_multiplier_monotone() {
        // Множитель должен быть монотонно возрастающим
        let p = DamageParams::default();
        let ages: Vec<f32> = (0..100).map(|i| i as f32).collect();
        for w in ages.windows(2) {
            let m0 = p.age_multiplier(w[0]);
            let m1 = p.age_multiplier(w[1]);
            assert!(m1 >= m0 - 1e-6,
                "множитель убывает в [{}, {}]: {:.4} → {:.4}", w[0], w[1], m0, m1);
        }
    }

    // --- P5: Репарация придатков ---

    #[test]
    fn test_repair_off_by_default() {
        // При repair_rate=0.0 (дефолт) репарация не применяется
        let params = DamageParams::default();
        let mut damage = CentriolarDamageState::pristine();
        damage.cep164_integrity = 0.5;
        let before = damage.cep164_integrity;

        apply_appendage_repair(&mut damage, &params, 1.0, 1.0);

        assert_eq!(damage.cep164_integrity, before, "дефолт: репарации нет");
    }

    #[test]
    fn test_antioxidant_preset_enables_repair() {
        let params = DamageParams::antioxidant();
        let mut damage = CentriolarDamageState::pristine();
        damage.cep164_integrity = 0.5;
        let before = damage.cep164_integrity;

        apply_appendage_repair(&mut damage, &params, 0.9, 1.0);

        assert!(damage.cep164_integrity > before,
            "antioxidant пресет должен восстанавливать CEP164: before={:.4}, after={:.4}",
            before, damage.cep164_integrity);
    }

    #[test]
    fn test_repair_capped_at_one() {
        // Репарация не должна поднимать integrity выше 1.0
        let mut params = DamageParams::default();
        params.cep164_repair_rate = 10.0; // очень высокая скорость
        let mut damage = CentriolarDamageState::pristine();
        damage.cep164_integrity = 0.99;

        apply_appendage_repair(&mut damage, &params, 0.0, 1.0);

        assert!(damage.cep164_integrity <= 1.0,
            "integrity не должна превышать 1.0: {:.4}", damage.cep164_integrity);
    }

    #[test]
    fn test_mitophagy_coupling_amplifies_repair() {
        let mut params = DamageParams::default();
        params.cep164_repair_rate = 0.01;
        params.appendage_repair_mitophagy_coupling = 1.0;
        let mut damage_low  = CentriolarDamageState::pristine();
        let mut damage_high = CentriolarDamageState::pristine();
        damage_low.cep164_integrity  = 0.5;
        damage_high.cep164_integrity = 0.5;

        // Низкий mitophagy_flux vs высокий
        apply_appendage_repair(&mut damage_low,  &params, 0.1, 1.0);
        apply_appendage_repair(&mut damage_high, &params, 0.9, 1.0);

        assert!(damage_high.cep164_integrity > damage_low.cep164_integrity,
            "высокий митофагический поток должен давать больше репарации: \
             low={:.4}, high={:.4}",
            damage_low.cep164_integrity, damage_high.cep164_integrity);
    }

    #[test]
    fn test_antioxidant_slower_damage_than_normal() {
        // Пресет antioxidant накапливает меньше повреждений за 50 лет
        let normal    = DamageParams::normal_aging();
        let antioxidant = DamageParams::antioxidant();
        let age = 50.0_f32;
        let dt  = 1.0 / 365.25_f32;

        let mut d_normal = CentriolarDamageState::pristine();
        let mut d_anti   = CentriolarDamageState::pristine();

        for _ in 0..(50 * 365) {
            accumulate_damage(&mut d_normal, &normal,    age, dt, 0.0);
            accumulate_damage(&mut d_anti,   &antioxidant, age, dt, 0.0);
        }

        assert!(d_anti.protein_carbonylation < d_normal.protein_carbonylation,
            "antioxidant: меньше карбонилирования: anti={:.3}, normal={:.3}",
            d_anti.protein_carbonylation, d_normal.protein_carbonylation);
    }

    // ── P21: AppendageProteinState ────────────────────────────────────────────

    /// CAII pristine = 1.0
    #[test]
    fn test_appendage_pristine_caii_is_one() {
        let a = AppendageProteinState::pristine();
        assert!((a.caii - 1.0).abs() < 1e-6, "pristine CAII={:.6}", a.caii);
    }

    /// CEP164 теряет целостность быстрее других при высоком OH·
    /// (чувствительность 1.50 vs 0.55 у CEP170).
    #[test]
    fn test_cep164_more_sensitive_to_oh_radical() {
        let params = DamageParams::default();
        let mut a = AppendageProteinState::pristine();
        // Высокий OH· (ros_level=0.9 → oh_level=0.81)
        let oh_high = 0.81_f32;
        let dt = 1.0 / 365.25_f32;

        for _ in 0..365 {
            accumulate_appendage_damage(&mut a, &params, oh_high, 50.0, dt);
        }

        assert!(a.cep164 < a.cep170,
            "CEP164 должен потерять больше при высоком OH·: cep164={:.3}, cep170={:.3}",
            a.cep164, a.cep170);
    }

    /// При OH·=0 порядок потери определяется только базовыми скоростями.
    /// cep164_loss_rate=0.0113 > cep89=ninein=0.0084 > cep170=0.0067.
    #[test]
    fn test_appendage_loss_order_without_oh() {
        let params = DamageParams::default();
        let mut a = AppendageProteinState::pristine();
        let dt = 1.0 / 365.25_f32;

        for _ in 0..(30 * 365) {
            accumulate_appendage_damage(&mut a, &params, 0.0, 30.0, dt);
        }

        // CEP164 теряет больше всех (0.0113/yr)
        assert!(a.cep164 < a.cep89,   "cep164 < cep89 за 30 лет: {:.3} vs {:.3}", a.cep164, a.cep89);
        // CEP89 = Ninein по скорости (одинаковые 0.0084/yr) → близкие значения
        let diff = (a.cep89 - a.ninein).abs();
        assert!(diff < 1e-4, "cep89 и ninein должны быть равны (одинаковая loss_rate): diff={:.6}", diff);
        // Ninein теряет больше, чем CEP170 (0.0084 vs 0.0067)
        assert!(a.ninein < a.cep170,  "ninein < cep170 за 30 лет: {:.3} vs {:.3}", a.ninein, a.cep170);
    }

    /// CAII = взвешенное геометрическое среднее; при частичной потере CEP164
    /// CAII снижается сильнее, чем при потере CEP170 (вес 0.40 vs 0.15).
    #[test]
    fn test_caii_cep164_weighted_more_than_cep170() {
        let mut a_cep164 = AppendageProteinState::pristine();
        a_cep164.cep164 = 0.5;
        a_cep164.update_caii();

        let mut a_cep170 = AppendageProteinState::pristine();
        a_cep170.cep170 = 0.5;
        a_cep170.update_caii();

        assert!(a_cep164.caii < a_cep170.caii,
            "CAII при CEP164=0.5 ({:.3}) должен быть ниже, чем при CEP170=0.5 ({:.3})",
            a_cep164.caii, a_cep170.caii);
    }

    /// Репарация восстанавливает целостность при включённом antioxidant-пресете.
    #[test]
    fn test_appendage_repair_restores_integrity() {
        let params = DamageParams::antioxidant();
        let mut a = AppendageProteinState::pristine();
        // Симулировать предварительное повреждение
        a.cep164 = 0.6;
        a.cep89  = 0.7;
        a.ninein = 0.75;
        a.cep170 = 0.8;
        a.update_caii();
        let caii_before = a.caii;

        // Применить репарацию за 1 год
        let dt = 1.0 / 365.25_f32;
        for _ in 0..365 {
            apply_appendage_protein_repair(&mut a, &params, 0.5, dt);
        }

        assert!(a.caii > caii_before,
            "CAII должен вырасти после репарации: before={:.3}, after={:.3}",
            caii_before, a.caii);
        assert!(a.cep164 > 0.6, "CEP164 должен восстановиться: {:.3}", a.cep164);
    }
}
