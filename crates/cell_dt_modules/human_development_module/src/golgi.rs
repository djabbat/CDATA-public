//! Аппарат Гольджи (уровень -1: органоиды).
//!
//! Связывает органелльный уровень с молекулярным через цепочку:
//!   ROS/SASP → фрагментация Гольджи → нарушение гликозилирования CEP164
//!            → ускоренная деградация дистальных придатков → CAII↓
//!
//! # Биологический контекст
//!
//! Аппарат Гольджи — центральный хаб процессинга белков. CEP164 (главный
//! организатор дистальных придатков) проходит N-гликозилирование в Гольджи,
//! что необходимо для его стабильного фолдинга и правильного рекрутирования
//! к базальному телу (Sundaramoorthy et al. 2016).
//!
//! При хроническом ROS/SASP: Гольджи фрагментируется (cisternae распадаются
//! на везикулы) → гликозилирующий конвейер нарушается → CEP164 выходит
//! недогликозилированным → распознаётся E3-лигазами (CHIP/STUB1) →
//! убиквитин-протеасомная деградация ускоряется.
//!
//! # Уравнение динамики
//!
//! ```text
//! dFragmentation/dt = ros × ros_rate + sasp × sasp_rate
//!                   − repair_rate × Fragmentation
//! ```
//!
//! Аутофагия снижает фрагментацию через удаление повреждённых цистерн
//! (Golgi-fаgy; Navrakas et al. 2023).

use serde::{Deserialize, Serialize};
use cell_dt_core::components::GolgiState;

// ─────────────────────────────────────────────────────────────────────────────
// Параметры
// ─────────────────────────────────────────────────────────────────────────────

/// Параметры динамики аппарата Гольджи.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolgiParams {
    /// Скорость фрагментации от ROS [/год на единицу ros_level].
    ///
    /// ROS → окисление GRASP65/GRASP55 → утрата ленточной структуры.
    /// Дефолт: 0.12/год. Источник: Colanzi & Corda 2007.
    pub ros_fragmentation_rate: f32,

    /// Скорость фрагментации от SASP (TNF/IL-6) [/год на единицу sasp_intensity].
    ///
    /// TNF → JNK → GRASP65-фосфорилирование → деленьение ленты.
    /// Дефолт: 0.08/год.
    pub sasp_fragmentation_rate: f32,

    /// Скорость репарации Гольджи [/год].
    ///
    /// Биогенез новых цистерн из ЭПС (COPI-транспорт) + Golgi-фагия.
    /// Дефолт: 0.30/год → Гольджи восстанавливается за ~3 года без стресса.
    pub repair_rate: f32,

    /// Вклад аутофагии в репарацию Гольджи [безразмерный].
    ///
    /// При высоком autophagy_flux: repair += autophagy_flux × autophagy_boost.
    /// Дефолт: 0.15.
    pub autophagy_boost: f32,

    /// Чувствительность CEP164 к гипогликозилированию [/год на единицу недостатка].
    ///
    /// Дополнительная скорость потери CEP164:
    ///   extra_rate = (1 − cep164_glycosylation) × sensitivity × dt_years
    /// Дефолт: 0.08/год. При glycosylation=0.5: CEP164 теряет доп. 4% в год.
    pub cep164_glycosyl_sensitivity: f32,
}

impl Default for GolgiParams {
    fn default() -> Self {
        Self {
            ros_fragmentation_rate:    0.12,
            sasp_fragmentation_rate:   0.08,
            repair_rate:               0.30,
            autophagy_boost:           0.15,
            cep164_glycosyl_sensitivity: 0.08,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Основная функция обновления
// ─────────────────────────────────────────────────────────────────────────────

/// Обновить GolgiState за один шаг.
///
/// Реализует:
///   dFragmentation/dt = ros × ros_rate + sasp × sasp_rate
///                     − (repair_rate + autophagy × boost) × Fragmentation
///
/// После обновления fragmentation → пересчёт glycosylation_capacity,
/// cep164_glycosylation, vesicle_trafficking_rate через `update_derived()`.
///
/// # Аргументы
/// * `golgi`           — изменяемый GolgiState.
/// * `ros_level`       — уровень ROS [0..1] (ros_level из CentriolarDamageState,
///                        или H₂O₂ из ROSCascadeState если доступен).
/// * `sasp_intensity`  — интенсивность SASP [0..1] (из InflammagingState).
/// * `autophagy_flux`  — поток аутофагии [0..1] (из AutophagyState или дефолт 0.4).
/// * `params`          — параметры модели.
/// * `dt_years`        — шаг времени [лет].
pub fn update_golgi_state(
    golgi: &mut GolgiState,
    ros_level: f32,
    sasp_intensity: f32,
    autophagy_flux: f32,
    params: &GolgiParams,
    dt_years: f32,
) {
    // Рост фрагментации от ROS и SASP
    let frag_production = ros_level   * params.ros_fragmentation_rate
                        + sasp_intensity * params.sasp_fragmentation_rate;

    // Репарация: базовая + аутофагия-усиленная
    let effective_repair = params.repair_rate + autophagy_flux * params.autophagy_boost;
    let frag_decay = effective_repair * golgi.fragmentation_index;

    golgi.fragmentation_index = (golgi.fragmentation_index
        + (frag_production - frag_decay) * dt_years)
        .clamp(0.0, 1.0);

    // Пересчёт производных метрик
    golgi.update_derived();
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 365.25;

    /// Молодое состояние: нет стресса → фрагментация остаётся низкой
    #[test]
    fn test_pristine_no_stress_stable() {
        let mut golgi = GolgiState::pristine();
        let params = GolgiParams::default();
        // Прогнать 5 лет без стресса
        for _ in 0..(5 * 365) {
            update_golgi_state(&mut golgi, 0.05, 0.0, 0.5, &params, DT);
        }
        assert!(golgi.fragmentation_index < 0.15,
            "Без стресса фрагментация должна быть мала: {:.4}", golgi.fragmentation_index);
        assert!(golgi.glycosylation_capacity > 0.85,
            "Ёмкость гликозилирования должна быть высокой: {:.4}", golgi.glycosylation_capacity);
    }

    /// Высокий ROS → рост фрагментации
    #[test]
    fn test_high_ros_increases_fragmentation() {
        let params = GolgiParams::default();
        let mut golgi_low  = GolgiState::pristine();
        let mut golgi_high = GolgiState::pristine();

        for _ in 0..(10 * 365) {
            update_golgi_state(&mut golgi_low,  0.05, 0.0, 0.5, &params, DT);
            update_golgi_state(&mut golgi_high, 0.60, 0.0, 0.5, &params, DT);
        }

        assert!(golgi_high.fragmentation_index > golgi_low.fragmentation_index,
            "Высокий ROS должен увеличивать фрагментацию: {:.4} vs {:.4}",
            golgi_high.fragmentation_index, golgi_low.fragmentation_index);
    }

    /// SASP → рост фрагментации
    #[test]
    fn test_sasp_increases_fragmentation() {
        let params = GolgiParams::default();
        let mut golgi_no_sasp   = GolgiState::pristine();
        let mut golgi_with_sasp = GolgiState::pristine();

        for _ in 0..(10 * 365) {
            update_golgi_state(&mut golgi_no_sasp,   0.1, 0.0, 0.5, &params, DT);
            update_golgi_state(&mut golgi_with_sasp, 0.1, 0.7, 0.5, &params, DT);
        }

        assert!(golgi_with_sasp.fragmentation_index > golgi_no_sasp.fragmentation_index,
            "SASP должен увеличивать фрагментацию: {:.4} vs {:.4}",
            golgi_with_sasp.fragmentation_index, golgi_no_sasp.fragmentation_index);
    }

    /// Аутофагия снижает фрагментацию
    #[test]
    fn test_autophagy_reduces_fragmentation() {
        let params = GolgiParams::default();
        let mut golgi_low_auto  = GolgiState::pristine();
        let mut golgi_high_auto = GolgiState::pristine();

        for _ in 0..(20 * 365) {
            update_golgi_state(&mut golgi_low_auto,  0.3, 0.2, 0.1, &params, DT);
            update_golgi_state(&mut golgi_high_auto, 0.3, 0.2, 0.9, &params, DT);
        }

        assert!(golgi_high_auto.fragmentation_index < golgi_low_auto.fragmentation_index,
            "Аутофагия↑ → фрагментация↓: {:.4} vs {:.4}",
            golgi_high_auto.fragmentation_index, golgi_low_auto.fragmentation_index);
    }

    /// Фрагментация → снижение гликозилирования CEP164
    #[test]
    fn test_fragmentation_reduces_cep164_glycosylation() {
        let params = GolgiParams::default();
        let mut golgi_intact    = GolgiState::pristine();
        let mut golgi_stressed  = GolgiState::pristine();

        for _ in 0..(15 * 365) {
            update_golgi_state(&mut golgi_intact,   0.05, 0.0, 0.5, &params, DT);
            update_golgi_state(&mut golgi_stressed, 0.50, 0.5, 0.1, &params, DT);
        }

        assert!(golgi_stressed.cep164_glycosylation < golgi_intact.cep164_glycosylation,
            "Стресс → cep164_glycosylation↓: {:.4} vs {:.4}",
            golgi_stressed.cep164_glycosylation, golgi_intact.cep164_glycosylation);
    }

    /// cep164_extra_loss_rate: линейна и 0 при полном гликозилировании
    #[test]
    fn test_cep164_extra_loss_rate_zero_when_full() {
        let mut golgi = GolgiState::pristine();
        // Форсировать полное гликозилирование
        golgi.cep164_glycosylation = 1.0;
        let rate = golgi.cep164_extra_loss_rate(0.08);
        assert!(rate < 1e-6, "При glycosylation=1.0 extra_loss должен быть 0: {:.6}", rate);
    }

    /// cep164_extra_loss_rate: растёт при снижении гликозилирования
    #[test]
    fn test_cep164_extra_loss_rate_grows_with_hypoglycosylation() {
        let mut golgi_good = GolgiState::pristine();
        let mut golgi_poor = GolgiState::pristine();
        golgi_good.cep164_glycosylation = 0.90;
        golgi_poor.cep164_glycosylation = 0.50;

        let sens = 0.08;
        assert!(golgi_poor.cep164_extra_loss_rate(sens) > golgi_good.cep164_extra_loss_rate(sens),
            "Низкое гликозилирование → выше скорость потери CEP164");
    }

    /// Производные метрики согласованы: glycosylation > cep164_glycosylation > traffic
    #[test]
    fn test_derived_metrics_ordering() {
        let params = GolgiParams::default();
        let mut golgi = GolgiState::pristine();
        for _ in 0..(10 * 365) {
            update_golgi_state(&mut golgi, 0.3, 0.3, 0.3, &params, DT);
        }
        // glycosylation_capacity >= cep164_glycosylation (×0.95)
        assert!(golgi.glycosylation_capacity >= golgi.cep164_glycosylation - 1e-4,
            "glycosyl >= cep164_glycosyl: {:.4} vs {:.4}",
            golgi.glycosylation_capacity, golgi.cep164_glycosylation);
        // glycosylation_capacity >= vesicle_trafficking_rate (×0.90)
        assert!(golgi.glycosylation_capacity >= golgi.vesicle_trafficking_rate - 1e-4,
            "glycosyl >= traffic: {:.4} vs {:.4}",
            golgi.glycosylation_capacity, golgi.vesicle_trafficking_rate);
    }

    /// fragmentation_index зажат в [0, 1]
    #[test]
    fn test_fragmentation_clamped() {
        let params = GolgiParams::default();
        let mut golgi = GolgiState::pristine();
        // Экстремальный стресс — 100 лет
        for _ in 0..(100 * 365) {
            update_golgi_state(&mut golgi, 1.0, 1.0, 0.0, &params, DT);
        }
        assert!(golgi.fragmentation_index <= 1.0,
            "fragmentation_index не должен превышать 1.0: {:.4}", golgi.fragmentation_index);
        assert!(golgi.glycosylation_capacity >= 0.10,
            "glycosylation_capacity не должен опускаться ниже 0.10: {:.4}",
            golgi.glycosylation_capacity);
    }
}
