//! Термодинамика клеточного старения — уровень -4 (атомный).
//!
//! Реализует температурозависимость молекулярных повреждений через уравнение Аррениуса
//! и отслеживает производство энтропии от необратимых PTM.
//!
//! # Физическая основа
//!
//! Уравнение Аррениуса: k(T) = A × exp(-Eₐ / R / T)
//! Отношение скоростей: mult(T) = k(T) / k(T_ref) = exp(Eₐ/R × (1/T_ref − 1/T))
//!
//! При повышении температуры на 1°C (T=38°C vs 37°C):
//!   mult(Eₐ=40 кДж/моль) ≈ 1.053 (+5.3%)
//!   mult(Eₐ=80 кДж/моль) ≈ 1.106 (+10.6%)
//!
//! При febris (T=39°C, +2°C):
//!   mult(Eₐ=40) ≈ 1.108 (+10.8%)
//!   mult(Eₐ=80) ≈ 1.221 (+22.1%)
//!
//! # Ze Theory connection
//!
//! PTM-накопление = необратимый перевод биологического времени в пространство (Tkemaladze).
//! Производство энтропии dS/dt > 0 (ΔG < 0 для карбонилирования/агрегации) =
//!   скорость «расхода» биологического времени → ze_velocity_analog.
//! v* = 0.456 — критическая точка равновесия T/S квантов Ze-поля ≈ молодой здоровый организм.

use serde::{Deserialize, Serialize};
use cell_dt_core::components::{CentriolarDamageState, ThermodynamicState};

// ─────────────────────────────────────────────────────────────────────────────
// Константы активационных энергий
// ─────────────────────────────────────────────────────────────────────────────

/// R = 8.314 Дж/(моль·К) — универсальная газовая постоянная.
const R_J_MOL_K: f32 = 8.314;

/// Энергии активации в Дж/моль для каждого трека PTM-повреждений.
///
/// Источники:
///   Карбонилирование: Stadtman & Levine (2003) Ann NY Acad Sci 1012:17-24.
///     ~50 кДж/моль — ROS-опосредованное окисление боковых цепей Met/His/Arg.
///   Ацетилирование: Albaugh et al. (2011) ACS Chem Biol — ферментативное
///     снижение HDAC6/SIRT2, активационный барьер пониженный (~40 кДж/моль).
///   Агрегация: Oosawa & Asakura (1975) нуклеация — барьер ~80 кДж/моль,
///     после нуклеации — автокаталитическое удлинение (Ea не применяется).
///   Фосфо-дисрегуляция: Seger & Krebs (1995) — дисбаланс PLK4/NEK2: ~45 кДж/моль.
///   CEP-придатки (OH· окисление): ~55 кДж/моль (среднее между Cys/Met целями).

/// Карбонилирование белков центриолей (SAS-6, CEP135) — [Дж/моль]
pub const EA_CARBONYLATION: f32 = 50_000.0;
/// Гиперацетилирование α-тубулина (HDAC6/SIRT2 depletion) — [Дж/моль]
pub const EA_ACETYLATION: f32   = 40_000.0;
/// Нуклеация агрегатов (CPAP, CEP290) — [Дж/моль]
pub const EA_AGGREGATION: f32   = 80_000.0;
/// Фосфорилирование-дисрегуляция (PLK4/NEK2/PP1 дисбаланс) — [Дж/моль]
pub const EA_PHOSPHO: f32       = 45_000.0;
/// Потеря белков придатков (CEP164 OH·-окисление, среднее) — [Дж/моль]
pub const EA_APPENDAGE: f32     = 55_000.0;

/// Взвешенная средняя Eₐ по всем PTM-трекам (используется для единого mult).
///
/// Веса пропорциональны вкладу трека в total_damage_score:
///   карбонилирование (×1), ацетилирование (×1), агрегация (×1.5 — нуклеация критична),
///   фосфо (×0.8), придатки (×1.5 — прямой функциональный эффект).
const EA_MEAN: f32 = (EA_CARBONYLATION * 1.0
    + EA_ACETYLATION  * 1.0
    + EA_AGGREGATION  * 1.5
    + EA_PHOSPHO      * 0.8
    + EA_APPENDAGE    * 1.5)
    / (1.0 + 1.0 + 1.5 + 0.8 + 1.5);

/// Весовые коэффициенты для расчёта производства энтропии по трекам.
/// Отражают долю необратимости реакции (ΔS при завершённой реакции).
pub struct EntropyWeights {
    /// Карбонилирование: C=O — конечный продукт, ΔG << 0, высокая необратимость.
    pub carbonylation: f32,
    /// Агрегация: конформационный коллапс — огромное ΔS (многие конфигурации).
    pub aggregation: f32,
    /// Ацетилирование: модифицирует заряд Lys, умеренная необратимость без HDAC6.
    pub acetylation: f32,
    /// Фосфо-дисрегуляция: частично обратима при наличии PP1/PP2A.
    pub phospho: f32,
    /// Потеря придатков: структурный коллапс, высокая необратимость.
    pub appendage: f32,
}

impl Default for EntropyWeights {
    fn default() -> Self {
        Self {
            carbonylation: 1.2,
            aggregation:   2.0,
            acetylation:   0.8,
            phospho:       0.5,
            appendage:     1.5,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Параметры термодинамики
// ─────────────────────────────────────────────────────────────────────────────

/// Параметры термодинамического модуля.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermodynamicParams {
    /// Базовая температура ниши [°C].
    ///
    /// HSC (костный мозг): 36.6°C — прохладнее ядра тела (мозг / сердце = 37.0°C).
    /// Нейральные ниши (СВЗ): 36.8°C.
    pub baseline_temp_celsius: f32,

    /// Максимальный вклад SASP в локальную температуру [°C].
    ///
    /// При sasp_intensity = 1.0: T += sasp_temp_contribution_max.
    /// Дефолт: 2.4°C (TNF/IL-6 → локальный отёк → T≤39°C; Hasday & Singh 2000).
    pub sasp_temp_contribution_max: f32,

    /// Активировать ли термодинамический множитель на накопление повреждений.
    /// `false` (дефолт) — обратная совместимость; множитель = 1.0.
    /// `true` — Аррениус масштабирует effective_dt.
    pub enable_arrhenius: bool,

    /// Масштабный коэффициент производства энтропии [нормировка].
    ///
    /// Определяет скорость нарастания entropy_production относительно
    /// скоростей повреждения. Дефолт: 1.0.
    pub entropy_scale: f32,
}

impl Default for ThermodynamicParams {
    fn default() -> Self {
        Self {
            baseline_temp_celsius:       36.6,
            sasp_temp_contribution_max:  2.4,
            enable_arrhenius:            false, // обратная совместимость
            entropy_scale:               1.0,
        }
    }
}

impl ThermodynamicParams {
    /// Аррениус включён (для новых симуляций с реалистичной термодинамикой).
    pub fn with_arrhenius() -> Self {
        Self {
            enable_arrhenius: true,
            ..Self::default()
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Основные функции
// ─────────────────────────────────────────────────────────────────────────────

/// Вычислить множитель Аррениуса для заданной Eₐ и температуры.
///
/// mult = exp(Eₐ / R × (1/T_ref − 1/T_local))
///
/// # Аргументы
/// * `ea_j_per_mol` — энергия активации [Дж/моль] (используйте константы EA_*).
/// * `temp_celsius` — локальная температура [°C].
///
/// # Возвращает
/// Безразмерный множитель ≥ 1.0 при T > T_ref, ≤ 1.0 при T < T_ref.
#[inline]
pub fn arrhenius_multiplier(ea_j_per_mol: f32, temp_celsius: f32) -> f32 {
    let t_k = temp_celsius + 273.15;
    let exponent = ea_j_per_mol / R_J_MOL_K * (1.0 / ThermodynamicState::T_REF_K - 1.0 / t_k);
    exponent.exp()
}

/// Обновить `ThermodynamicState` за один шаг.
///
/// # Порядок вычислений
/// 1. Обновить `local_temp_celsius` из `sasp_intensity` (lинейная интерполяция).
/// 2. Вычислить `damage_rate_multiplier` по взвешенной средней Eₐ (Аррениус).
/// 3. Накопить `entropy_production` из текущих скоростей повреждений (dS/dt × dt).
/// 4. Вычислить `ze_velocity_analog` из кумулятивной энтропии (Hill-функция).
///
/// # Аргументы
/// * `thermo` — изменяемая ссылка на ThermodynamicState.
/// * `damage` — текущие повреждения для расчёта скорости энтропийного производства.
/// * `sasp_intensity` — интенсивность SASP [0..1] из InflammagingState (0 если нет).
/// * `params` — параметры термодинамического модуля.
/// * `dt_years` — шаг времени [лет].
pub fn update_thermodynamic_state(
    thermo: &mut ThermodynamicState,
    damage: &CentriolarDamageState,
    sasp_intensity: f32,
    params: &ThermodynamicParams,
    dt_years: f32,
) {
    // 1. Локальная температура: baseline + SASP-вклад
    thermo.local_temp_celsius = params.baseline_temp_celsius
        + sasp_intensity * params.sasp_temp_contribution_max;

    // 2. Множитель Аррениуса по взвешенной средней Eₐ
    if params.enable_arrhenius {
        thermo.damage_rate_multiplier = arrhenius_multiplier(EA_MEAN, thermo.local_temp_celsius);
    } else {
        thermo.damage_rate_multiplier = 1.0;
    }

    // 3. Производство энтропии
    // dS/dt ≈ Σᵢ (damage_rate_i × weight_i) — скорость нарастания PTM-энтропии.
    // Используем скорости из производных изменений за шаг (приближение через ΔD/dt):
    // Но damage — текущие уровни, а не скорости. Используем уровни как прокси:
    //   entropy_rate ≈ Σ (D_i × rate_weight_i) — чем больше накоплено, тем выше dS/dt.
    // Это отражает автокаталитическое ускорение — разрушенная структура ускоряет
    // дальнейшее разрушение (ROS-петля, агрегационная нуклеация).
    let w = EntropyWeights::default();
    let entropy_rate = (damage.protein_carbonylation   * w.carbonylation
                      + damage.protein_aggregates      * w.aggregation
                      + damage.tubulin_hyperacetylation * w.acetylation
                      + damage.phosphorylation_dysregulation * w.phospho
                      + (1.0 - damage.ciliary_function) * w.appendage)
        * params.entropy_scale
        * thermo.damage_rate_multiplier; // воспаление ускоряет энтропийное производство

    thermo.entropy_production += entropy_rate * dt_years;

    // 4. Ze velocity analog: Hill-функция с K_ze = 2.0
    // ze_velocity = entropy / (entropy + K_ze)
    // При entropy=0: ze_velocity=0 (новорождённая клетка)
    // При entropy=K_ze=2.0: ze_velocity=0.5
    // При entropy≈1.95: ze_velocity≈0.494 ≈ v* для ~20 лет (default params)
    thermo.ze_velocity_analog = thermo.entropy_production
        / (thermo.entropy_production + ThermodynamicState::ZE_K);
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// При T = T_ref (37°C): множитель Аррениуса = 1.0 (все треки)
    #[test]
    fn test_arrhenius_at_reference_temp_is_one() {
        let temp_ref = ThermodynamicState::T_REF_K - 273.15; // 36.85°C
        for ea in [EA_CARBONYLATION, EA_ACETYLATION, EA_AGGREGATION, EA_PHOSPHO] {
            let mult = arrhenius_multiplier(ea, temp_ref);
            assert!((mult - 1.0).abs() < 1e-4,
                "При T_ref mult={:.6} (Ea={:.0})", mult, ea);
        }
    }

    /// При T > T_ref: множитель > 1.0 (повреждения ускоряются)
    #[test]
    fn test_arrhenius_fever_increases_rate() {
        let mult_38 = arrhenius_multiplier(EA_MEAN, 38.0);
        let mult_39 = arrhenius_multiplier(EA_MEAN, 39.0);
        assert!(mult_38 > 1.0, "T=38°C должен давать mult>1: {:.4}", mult_38);
        assert!(mult_39 > mult_38, "T=39°C должен давать mult>38°C: {:.4} vs {:.4}", mult_39, mult_38);
        // При T=39°C (+2°C): mult ≈ 1.14–1.22, не должно быть слишком большим
        assert!(mult_39 < 1.35, "mult при 39°C не должен быть > 1.35: {:.4}", mult_39);
    }

    /// Высокая Eₐ → более сильная температурная зависимость
    #[test]
    fn test_higher_ea_more_sensitive_to_temp() {
        let temp = 39.0;
        let mult_low  = arrhenius_multiplier(EA_ACETYLATION,    temp); // 40 кДж/моль
        let mult_high = arrhenius_multiplier(EA_AGGREGATION,    temp); // 80 кДж/моль
        assert!(mult_high > mult_low,
            "Агрегация (80 кДж/моль) должна сильнее реагировать на T: {:.4} vs {:.4}",
            mult_high, mult_low);
    }

    /// При T < T_ref: mult < 1.0 (холодовая защита)
    #[test]
    fn test_arrhenius_cold_slows_damage() {
        let mult_cold = arrhenius_multiplier(EA_MEAN, 35.0); // −2°C от 37°C
        assert!(mult_cold < 1.0,
            "При 35°C mult должен быть < 1.0: {:.4}", mult_cold);
    }

    /// Производство энтропии нарастает со временем при ненулевых повреждениях
    #[test]
    fn test_entropy_increases_over_time() {
        let mut thermo = ThermodynamicState::pristine();
        let mut damage = CentriolarDamageState::pristine();
        // Симулировать уже накопленные повреждения
        damage.protein_carbonylation = 0.3;
        damage.protein_aggregates    = 0.2;
        let params = ThermodynamicParams::with_arrhenius();

        let dt = 1.0 / 365.25_f32;
        for _ in 0..365 {
            update_thermodynamic_state(&mut thermo, &damage, 0.0, &params, dt);
        }

        assert!(thermo.entropy_production > 0.0,
            "entropy_production должен расти: {:.4}", thermo.entropy_production);
        assert!(thermo.ze_velocity_analog > 0.0,
            "ze_velocity_analog должен быть > 0: {:.4}", thermo.ze_velocity_analog);
    }

    /// Ze velocity ≈ v* при типичных повреждениях молодого организма (~20 лет)
    #[test]
    fn test_ze_velocity_near_optimal_at_young_age() {
        let mut thermo = ThermodynamicState::pristine();
        // Типичные повреждения 20-летнего: минимальные
        let mut damage = CentriolarDamageState::pristine();
        damage.protein_carbonylation    = 0.02;
        damage.protein_aggregates       = 0.01;
        damage.tubulin_hyperacetylation = 0.02;
        damage.phosphorylation_dysregulation = 0.01;
        damage.ciliary_function         = 0.98;

        let params = ThermodynamicParams::with_arrhenius();
        let dt = 1.0 / 365.25_f32;

        // Симулировать 20 лет
        for _ in 0..(20 * 365) {
            update_thermodynamic_state(&mut thermo, &damage, 0.0, &params, dt);
        }

        // v* = 0.456: молодой организм должен быть вблизи (0.2–0.6 диапазон приемлем)
        assert!(thermo.ze_velocity_analog > 0.1 && thermo.ze_velocity_analog < 0.7,
            "ze_velocity у 20-летнего должен быть в [0.1..0.7]: {:.4}",
            thermo.ze_velocity_analog);
    }

    /// SASP повышает локальную температуру
    #[test]
    fn test_sasp_raises_local_temperature() {
        let mut thermo = ThermodynamicState::pristine();
        let damage = CentriolarDamageState::pristine();
        let params = ThermodynamicParams::default();
        let dt = 1.0 / 365.25_f32;

        update_thermodynamic_state(&mut thermo, &damage, 0.0, &params, dt);
        let temp_no_sasp = thermo.local_temp_celsius;

        update_thermodynamic_state(&mut thermo, &damage, 0.5, &params, dt);
        let temp_with_sasp = thermo.local_temp_celsius;

        assert!(temp_with_sasp > temp_no_sasp,
            "SASP=0.5 должен повысить T: {:.2} vs {:.2}", temp_with_sasp, temp_no_sasp);
    }

    /// Аррениус: enable_arrhenius=false → mult=1.0 (обратная совместимость)
    #[test]
    fn test_arrhenius_disabled_gives_mult_one() {
        let mut thermo = ThermodynamicState::pristine();
        let damage = CentriolarDamageState::pristine();
        let params = ThermodynamicParams::default(); // enable_arrhenius = false

        update_thermodynamic_state(&mut thermo, &damage, 0.8, &params, 0.1);

        assert!((thermo.damage_rate_multiplier - 1.0).abs() < 1e-6,
            "При disable_arrhenius mult должен быть 1.0: {:.6}", thermo.damage_rate_multiplier);
    }
}
