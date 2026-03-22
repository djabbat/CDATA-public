//! ROS-каскад — уровень -3 (молекулярный): O₂⁻ → H₂O₂ → OH· (Фентон).
//!
//! Детализирует единый `ros_level` в 4-переменный каскад реактивных форм кислорода.
//! Каждая переменная имеет собственную кинетику образования и детоксикации.
//!
//! # Каскад
//!
//! ```text
//! Митохондрии (комплекс I/III)
//!     → O₂⁻  (superoxide, утечка ~2% от O₂-потока)
//!         ↓ SOD1/SOD2  (k_sod ≈ 2×10⁹ M⁻¹s⁻¹)
//!     → H₂O₂ (hydrogen_peroxide)
//!         ↓ каталаза/GPx  → H₂O + ½O₂  (детоксикация)
//!         ↓ + Fe²⁺ (Fenton)  → OH·  (гидроксил-радикал)
//!     → OH·  (hydroxyl_radical, t½ < 1 нс)
//!         → CEP164/CEP89/Ninein/CEP170 (окисление придатков)
//!         → SAS-6/CEP135 карбонилирование
//!   labile_iron: Fe²⁺/Fe³⁺ ← ферритин-деградация
//!                          → ферропортин-экспорт
//! ```
//!
//! # Обратная совместимость
//!
//! При отсутствии ROSCascadeState: `accumulate_damage()` использует скалярный
//! `ros_level` из `CentriolarDamageState` как прежде. При наличии — `ros_level`
//! синхронизируется из `ROSCascadeState.hydrogen_peroxide` и OH· передаётся
//! в `accumulate_appendage_damage()`.

use serde::{Deserialize, Serialize};
use cell_dt_core::components::ROSCascadeState;

// ─────────────────────────────────────────────────────────────────────────────
// Параметры каскада
// ─────────────────────────────────────────────────────────────────────────────

/// Параметры ROS-каскада (все скорости нормированы к шагу 1 год).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROSCascadeParams {
    // ── Образование O₂⁻ ──────────────────────────────────────────────────────

    /// Базовый поток образования O₂⁻ от митохондрий [/год].
    ///
    /// Пропорционален `mito_ros_production` (MitochondrialState).
    /// Дефолт: 0.04 (= физиологический флюкс ~2% от O₂-потока в год).
    pub superoxide_production_rate: f32,

    // ── SOD: O₂⁻ → H₂O₂ ─────────────────────────────────────────────────────

    /// Скорость конверсии O₂⁻ → H₂O₂ через SOD1/SOD2 [/год].
    ///
    /// Высокая (SOD очень эффективна): дефолт 8.0/год → O₂⁻ быстро убывает.
    pub sod_rate: f32,

    // ── Детоксикация H₂O₂ ────────────────────────────────────────────────────

    /// Скорость детоксикации H₂O₂ каталазой/GPx [/год].
    ///
    /// При старении каталаза снижается → H₂O₂ накапливается.
    /// Дефолт: 4.0/год. Снижается с возрастом через `age_detox_decay`.
    pub catalase_rate: f32,

    /// Снижение скорости каталазы с возрастом [/год на год].
    ///
    /// catalase_eff = catalase_rate × (1 - age_years × age_detox_decay).
    /// Дефолт: 0.003 → при 70 лет: catalase ×0.79. Источник: Tian et al. 1998.
    pub age_detox_decay: f32,

    // ── Фентон: H₂O₂ + Fe²⁺ → OH· ───────────────────────────────────────────

    /// Скорость Фентон-реакции [/год].
    ///
    /// Пропорциональна labile_iron × hydrogen_peroxide.
    /// Дефолт: 2.0/год. Источник: Halliwell & Gutteridge 1984.
    pub fenton_rate: f32,

    /// Амплитуда усиления OH· от лабильного железа [безразмерный].
    ///
    /// effective_oh = hydroxyl_radical × (1 + labile_iron × fenton_amplification).
    /// Дефолт: 1.5.
    pub fenton_amplification: f32,

    // ── Распад OH· ────────────────────────────────────────────────────────────

    /// Скорость «потребления» OH· — реагирует с биомолекулами [/год].
    ///
    /// OH· не накапливается (t½ < 1 нс) — быстро расходуется.
    /// Дефолт: 50.0/год (псевдо-стационарное состояние за шаг дня).
    pub oh_decay_rate: f32,

    // ── Железо ────────────────────────────────────────────────────────────────

    /// Скорость накопления лабильного железа (феррофагия ферритина) [/год].
    ///
    /// Растёт с повреждениями аутофагии и возрастом.
    /// Дефолт: 0.005/год.
    pub iron_accumulation_rate: f32,

    /// Скорость экспорта/хелатирования железа [/год].
    ///
    /// Через ферропортин и GSH-хелатирование.
    /// Дефолт: 0.02/год.
    pub iron_export_rate: f32,
}

impl Default for ROSCascadeParams {
    fn default() -> Self {
        Self {
            superoxide_production_rate: 0.04,
            sod_rate:                   8.0,
            catalase_rate:              4.0,
            age_detox_decay:            0.003,
            fenton_rate:                2.0,
            fenton_amplification:       1.5,
            oh_decay_rate:              50.0,
            iron_accumulation_rate:     0.005,
            iron_export_rate:           0.020,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Основная функция обновления
// ─────────────────────────────────────────────────────────────────────────────

/// Обновить ROSCascadeState за один шаг.
///
/// Реализует дифференциальные уравнения:
///   dO₂⁻/dt  = production − SOD × O₂⁻
///   dH₂O₂/dt = SOD × O₂⁻ − catalase × H₂O₂ − fenton × Fe²⁺ × H₂O₂
///   dOH·/dt  = fenton × Fe²⁺ × H₂O₂ − oh_decay × OH·
///   dFe/dt   = iron_accum − iron_export × Fe
///
/// Все уравнения решаются явным методом Эйлера (достаточно при малом dt).
///
/// # Аргументы
/// * `ros` — изменяемый ROSCascadeState.
/// * `mito_ros_production` — вклад митохондрий [0..1]; 0.0 если MitochondrialState отсутствует.
/// * `autophagy_flux` — поток аутофагии [0..1]; высокий → ферропортин-экспорт Fe²⁺↑.
/// * `params` — параметры каскада.
/// * `age_years` — возраст [лет] → снижение каталазы.
/// * `dt_years` — шаг времени [лет].
pub fn update_ros_cascade(
    ros: &mut ROSCascadeState,
    mito_ros_production: f32,
    autophagy_flux: f32,
    params: &ROSCascadeParams,
    age_years: f32,
    dt_years: f32,
) {
    // O₂⁻: образование от митохондрий, разрушение SOD
    let superoxide_production = params.superoxide_production_rate * (0.5 + mito_ros_production * 0.5);
    let superoxide_decay       = params.sod_rate * ros.superoxide;
    ros.superoxide = (ros.superoxide
        + (superoxide_production - superoxide_decay) * dt_years)
        .clamp(0.0, 1.0);

    // H₂O₂: приток от SOD, убыль через каталазу и Фентон
    let sod_flux     = params.sod_rate * ros.superoxide;
    // Возрастное снижение каталазы
    let cat_eff      = params.catalase_rate * (1.0 - age_years * params.age_detox_decay).max(0.1);
    let fenton_flux  = params.fenton_rate * ros.labile_iron * ros.hydrogen_peroxide;
    ros.hydrogen_peroxide = (ros.hydrogen_peroxide
        + (sod_flux - cat_eff * ros.hydrogen_peroxide - fenton_flux) * dt_years)
        .clamp(0.0, 1.0);

    // OH·: образование Фентон, быстрое потребление биомолекулами
    let oh_production = fenton_flux;
    let oh_decay      = params.oh_decay_rate * ros.hydroxyl_radical;
    ros.hydroxyl_radical = (ros.hydroxyl_radical
        + (oh_production - oh_decay) * dt_years)
        .clamp(0.0, 1.0);

    // Лабильное железо: накопление (феррофагия) − экспорт
    // Аутофагия снижает лабильное железо через правильную деградацию ферритина
    let iron_prod = params.iron_accumulation_rate * (1.0 - autophagy_flux * 0.5);
    let iron_exp  = params.iron_export_rate * ros.labile_iron;
    ros.labile_iron = (ros.labile_iron + (iron_prod - iron_exp) * dt_years)
        .clamp(0.0, 1.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 365.25;

    /// Физиологическое начальное состояние корректно инициализировано
    #[test]
    fn test_physiological_init() {
        let ros = ROSCascadeState::physiological();
        assert!(ros.superoxide > 0.0 && ros.superoxide < 0.1);
        assert!(ros.hydrogen_peroxide > 0.0 && ros.hydrogen_peroxide < 0.15);
        assert!(ros.hydroxyl_radical < 0.05);
        assert!(ros.labile_iron > 0.0 && ros.labile_iron < 0.25);
    }

    /// При нормальных митохондриях система сходится к псевдостационару
    #[test]
    fn test_steady_state_normal_mito() {
        let mut ros = ROSCascadeState::physiological();
        let params = ROSCascadeParams::default();
        // Прогнать 10 лет без стресса
        for _ in 0..(10 * 365) {
            update_ros_cascade(&mut ros, 0.1, 0.5, &params, 30.0, DT);
        }
        // H₂O₂ должен оставаться в физиологическом диапазоне
        assert!(ros.hydrogen_peroxide < 0.3,
            "H₂O₂ при норме: {:.4}", ros.hydrogen_peroxide);
        // OH· крайне мал (быстро расходуется)
        assert!(ros.hydroxyl_radical < 0.05,
            "OH· при норме должен быть мал: {:.4}", ros.hydroxyl_radical);
    }

    /// Высокий митохондриальный стресс → рост H₂O₂ и OH·
    #[test]
    fn test_high_mito_stress_raises_h2o2_and_oh() {
        let mut ros_normal = ROSCascadeState::physiological();
        let mut ros_stress = ROSCascadeState::physiological();
        let params = ROSCascadeParams::default();

        for _ in 0..(5 * 365) {
            update_ros_cascade(&mut ros_normal, 0.1, 0.5, &params, 40.0, DT);
            update_ros_cascade(&mut ros_stress, 0.8, 0.2, &params, 40.0, DT); // высокий ROS, низкая аутофагия
        }

        assert!(ros_stress.hydrogen_peroxide > ros_normal.hydrogen_peroxide,
            "Стресс: H₂O₂ {:.4} должен быть > норма {:.4}",
            ros_stress.hydrogen_peroxide, ros_normal.hydrogen_peroxide);
        assert!(ros_stress.hydroxyl_radical > ros_normal.hydroxyl_radical,
            "Стресс: OH· {:.4} должен быть > норма {:.4}",
            ros_stress.hydroxyl_radical, ros_normal.hydroxyl_radical);
    }

    /// Возрастное снижение каталазы → рост H₂O₂ при старении
    #[test]
    fn test_age_reduces_catalase_raises_h2o2() {
        let mut ros_young = ROSCascadeState::physiological();
        let mut ros_old   = ROSCascadeState::physiological();
        let params = ROSCascadeParams::default();

        for _ in 0..(5 * 365) {
            update_ros_cascade(&mut ros_young, 0.2, 0.4, &params, 25.0, DT);
            update_ros_cascade(&mut ros_old,   0.2, 0.4, &params, 75.0, DT);
        }

        assert!(ros_old.hydrogen_peroxide >= ros_young.hydrogen_peroxide,
            "H₂O₂ у старого ({:.4}) должен быть ≥ молодого ({:.4})",
            ros_old.hydrogen_peroxide, ros_young.hydrogen_peroxide);
    }

    /// Аутофагия снижает лабильное железо
    #[test]
    fn test_autophagy_reduces_labile_iron() {
        let mut ros_low_auto  = ROSCascadeState::physiological();
        let mut ros_high_auto = ROSCascadeState::physiological();
        let params = ROSCascadeParams::default();

        for _ in 0..(20 * 365) {
            update_ros_cascade(&mut ros_low_auto,  0.2, 0.1, &params, 50.0, DT);
            update_ros_cascade(&mut ros_high_auto, 0.2, 0.9, &params, 50.0, DT);
        }

        assert!(ros_high_auto.labile_iron < ros_low_auto.labile_iron,
            "Аутофагия↑ должна снижать Fe: {:.4} vs {:.4}",
            ros_high_auto.labile_iron, ros_low_auto.labile_iron);
    }

    /// effective_oh: высокое железо усиливает OH·-эффект
    #[test]
    fn test_effective_oh_amplified_by_iron() {
        let mut ros_low_fe  = ROSCascadeState::physiological();
        let mut ros_high_fe = ROSCascadeState::physiological();
        ros_low_fe.labile_iron  = 0.05;
        ros_high_fe.labile_iron = 0.50;
        ros_low_fe.hydroxyl_radical  = 0.02;
        ros_high_fe.hydroxyl_radical = 0.02;

        let amp = 1.5;
        let eff_low  = ros_low_fe.effective_oh(amp);
        let eff_high = ros_high_fe.effective_oh(amp);

        assert!(eff_high > eff_low,
            "Высокое Fe должно усиливать OH·-эффект: {:.4} vs {:.4}",
            eff_high, eff_low);
    }

    /// ros_level_compat() = hydrogen_peroxide
    #[test]
    fn test_ros_level_compat_equals_h2o2() {
        let ros = ROSCascadeState::physiological();
        assert!((ros.ros_level_compat() - ros.hydrogen_peroxide).abs() < 1e-6);
    }
}
