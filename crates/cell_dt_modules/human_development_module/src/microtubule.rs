//! Динамика микротрубочек стволовой ниши (уровень -2: цитоскелет).
//!
//! Детализирует скалярный `spindle_fidelity` через модель динамической нестабильности MT
//! (Mitchison & Kirschner 1984).
//!
//! # Механизм
//!
//! ```text
//! PTM (гиперацетилирование тубулина)
//!     → polymerization_rate ↓  (GTPase-инактивация; Bhattacharya 2008)
//!
//! Фосфо-дисрегуляция (PLK4/NEK2/Aurora B)
//!     → catastrophe_rate ↑  (Garner et al. 2004)
//!
//! DII = cat / (poly + cat)   [Mitchison & Kirschner 1984]
//!
//! Ninein (субдистальный придаток)
//!     → якорение минус-концов MT к центриоли
//!     → spindle_fidelity_derived = (1 − DII) × ninein_integrity
//! ```
//!
//! # Интеграция с ECS
//!
//! При наличии `MicrotubuleState` у сущности:
//! - `spindle_fidelity_derived` переписывает `CentriolarDamageState.spindle_fidelity`
//! - Обеспечивает обратную совместимость: без компонента используется скалярный `spindle_fidelity`

use serde::{Deserialize, Serialize};
use cell_dt_core::components::MicrotubuleState;

// ─────────────────────────────────────────────────────────────────────────────
// Параметры
// ─────────────────────────────────────────────────────────────────────────────

/// Параметры модели динамики микротрубочек.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrotubuleParams {
    /// Базовая скорость полимеризации при нулевом ацетилировании [0..1].
    ///
    /// Физиологически: 0.90. Определяет рабочую точку MT-цикла.
    pub baseline_poly: f32,

    /// Базовая частота катастроф при нулевой фосфо-дисрегуляции [0..1].
    ///
    /// Физиологически: 0.10 (≈10% времени MT в фазе распада).
    pub baseline_cat: f32,

    /// Чувствительность полимеризации к гиперацетилированию тубулина [0..1].
    ///
    /// poly_eff = baseline_poly × (1 − acetylation × sensitivity).
    /// Дефолт: 0.70 → при acetylation=1.0: poly = 0.90 × 0.30 = 0.27.
    /// Биологически: HDAC6-ингибирование → стабильные, но GTPase-неактивные MT.
    pub acetylation_poly_inhibition: f32,

    /// Чувствительность частоты катастроф к фосфо-дисрегуляции [0..1].
    ///
    /// cat_eff = baseline_cat + phospho × amplification.
    /// Дефолт: 0.80 → при phospho=1.0: cat = 0.10 + 0.80 = 0.90 (clamped).
    /// Биологически: PLK4/NEK2 дисбаланс → Aurora B нарушен → MT не стабилизируются.
    pub phospho_catastrophe_amplification: f32,
}

impl Default for MicrotubuleParams {
    fn default() -> Self {
        Self {
            baseline_poly:                    0.90,
            baseline_cat:                     0.10,
            acetylation_poly_inhibition:      0.70,
            phospho_catastrophe_amplification: 0.80,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Основная функция обновления
// ─────────────────────────────────────────────────────────────────────────────

/// Обновить MicrotubuleState за один шаг (квазистационарное равновесие).
///
/// Реализует модель:
///   poly = baseline_poly × (1 − acetylation × acetylation_poly_inhibition)
///   cat  = baseline_cat + phospho_dysreg × phospho_catastrophe_amplification
///   DII  = cat / (poly + cat)
///   spindle_fidelity_derived = (1 − DII) × ninein_integrity
///
/// Квазистационарное приближение: poly/cat задаются PTM напрямую (нет ОДУ).
/// Это корректно, поскольку MT-динамика устанавливается за секунды, а
/// PTM-состояния меняются за годы (разделение временных масштабов).
///
/// # Аргументы
/// * `mt`                — изменяемый MicrotubuleState.
/// * `tubulin_acetylation` — гиперацетилирование тубулина [0..1] (из CentriolePair.PTM).
/// * `phospho_dysreg`    — фосфо-дисрегуляция [0..1] (из CentriolarDamageState).
/// * `ninein_integrity`  — целостность Ninein [0..1] (якорение минус-концов MT).
/// * `params`            — параметры модели.
pub fn update_microtubule_state(
    mt: &mut MicrotubuleState,
    tubulin_acetylation: f32,
    phospho_dysreg: f32,
    ninein_integrity: f32,
    params: &MicrotubuleParams,
) {
    // Полимеризация: гиперацетилирование → GTPase-инактивация → MT менее динамичны
    mt.polymerization_rate = (params.baseline_poly
        * (1.0 - tubulin_acetylation * params.acetylation_poly_inhibition))
        .clamp(0.10, 1.0);

    // Катастрофа: фосфо-дисбаланс (PLK4/NEK2/Aurora B) → MT нестабильны
    mt.catastrophe_rate = (params.baseline_cat
        + phospho_dysreg * params.phospho_catastrophe_amplification)
        .clamp(0.0, 0.90);

    // DII = cat / (poly + cat), затем spindle_fidelity_derived через Ninein
    mt.update_derived(ninein_integrity);
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Нулевые PTM + Ninein=1.0 → физиологическое состояние (poly=0.90, cat=0.10)
    #[test]
    fn test_pristine_state_no_damage() {
        let mut mt = MicrotubuleState::pristine();
        let params = MicrotubuleParams::default();
        update_microtubule_state(&mut mt, 0.0, 0.0, 1.0, &params);

        assert!((mt.polymerization_rate - 0.90).abs() < 1e-4,
            "poly: {:.4}", mt.polymerization_rate);
        assert!((mt.catastrophe_rate - 0.10).abs() < 1e-4,
            "cat: {:.4}", mt.catastrophe_rate);
        // DII = 0.10 / (0.90 + 0.10) = 0.10
        assert!((mt.dynamic_instability_index - 0.10).abs() < 1e-4,
            "DII: {:.4}", mt.dynamic_instability_index);
        // spindle = (1 − 0.10) × 1.0 = 0.90
        assert!((mt.spindle_fidelity_derived - 0.90).abs() < 1e-4,
            "spindle: {:.4}", mt.spindle_fidelity_derived);
    }

    /// Высокое ацетилирование снижает polymerization_rate
    #[test]
    fn test_high_acetylation_reduces_poly() {
        let params = MicrotubuleParams::default();
        let mut mt_low  = MicrotubuleState::pristine();
        let mut mt_high = MicrotubuleState::pristine();
        update_microtubule_state(&mut mt_low,  0.0, 0.0, 1.0, &params);
        update_microtubule_state(&mut mt_high, 0.8, 0.0, 1.0, &params);
        assert!(mt_high.polymerization_rate < mt_low.polymerization_rate,
            "Ацетилирование↑ должно снижать poly: {:.4} vs {:.4}",
            mt_high.polymerization_rate, mt_low.polymerization_rate);
    }

    /// Высокая фосфо-дисрегуляция повышает catastrophe_rate
    #[test]
    fn test_high_phospho_raises_catastrophe() {
        let params = MicrotubuleParams::default();
        let mut mt_low  = MicrotubuleState::pristine();
        let mut mt_high = MicrotubuleState::pristine();
        update_microtubule_state(&mut mt_low,  0.0, 0.0, 1.0, &params);
        update_microtubule_state(&mut mt_high, 0.0, 0.8, 1.0, &params);
        assert!(mt_high.catastrophe_rate > mt_low.catastrophe_rate,
            "Фосфо-дисрег↑ должна повышать cat: {:.4} vs {:.4}",
            mt_high.catastrophe_rate, mt_low.catastrophe_rate);
    }

    /// DII вычисляется правильно по формуле cat/(poly+cat)
    #[test]
    fn test_dii_formula_correct() {
        let params = MicrotubuleParams::default();
        let mut mt = MicrotubuleState::pristine();
        let acetyl  = 0.5_f32;
        let phospho = 0.3_f32;
        update_microtubule_state(&mut mt, acetyl, phospho, 1.0, &params);

        let expected_poly = (0.90 * (1.0 - acetyl * 0.70)).clamp(0.10, 1.0);
        let expected_cat  = (0.10 + phospho * 0.80).clamp(0.0, 0.90);
        let expected_dii  = expected_cat / (expected_poly + expected_cat);
        assert!((mt.dynamic_instability_index - expected_dii).abs() < 1e-5,
            "DII: {:.6} vs expected {:.6}", mt.dynamic_instability_index, expected_dii);
    }

    /// Низкая целостность Ninein → снижение spindle_fidelity_derived
    #[test]
    fn test_ninein_loss_reduces_spindle_fidelity() {
        let params = MicrotubuleParams::default();
        let mut mt_good = MicrotubuleState::pristine();
        let mut mt_poor = MicrotubuleState::pristine();
        update_microtubule_state(&mut mt_good, 0.0, 0.0, 1.0, &params);
        update_microtubule_state(&mut mt_poor, 0.0, 0.0, 0.3, &params);
        assert!(mt_poor.spindle_fidelity_derived < mt_good.spindle_fidelity_derived,
            "Ninein↓ → spindle↓: {:.4} vs {:.4}",
            mt_poor.spindle_fidelity_derived, mt_good.spindle_fidelity_derived);
    }

    /// Максимальное повреждение → spindle_fidelity_derived стремится к 0
    #[test]
    fn test_full_damage_collapses_spindle_fidelity() {
        let params = MicrotubuleParams::default();
        let mut mt = MicrotubuleState::pristine();
        // acetylation=1.0, phospho=1.0, ninein=0.0
        update_microtubule_state(&mut mt, 1.0, 1.0, 0.0, &params);
        assert!(mt.spindle_fidelity_derived < 0.01,
            "При полном повреждении spindle_fidelity_derived должен быть ≈0: {:.4}",
            mt.spindle_fidelity_derived);
    }

    /// catastrophe_rate зажат при phospho=1.0: не превышает 0.90
    #[test]
    fn test_catastrophe_rate_clamped_at_max() {
        let params = MicrotubuleParams::default();
        let mut mt = MicrotubuleState::pristine();
        update_microtubule_state(&mut mt, 0.0, 1.0, 1.0, &params);
        // cat = (0.10 + 1.0 × 0.80).clamp(0, 0.90) = 0.90
        assert!((mt.catastrophe_rate - 0.90).abs() < 1e-4,
            "catastrophe clamped at 0.90: {:.4}", mt.catastrophe_rate);
    }

    /// Монотонность по phospho_dysreg: cat и DII растут
    #[test]
    fn test_dii_monotone_in_phospho() {
        let params = MicrotubuleParams::default();
        let mut prev_dii = -1.0_f32;
        for &phospho in &[0.0_f32, 0.2, 0.4, 0.6, 0.8, 1.0] {
            let mut mt = MicrotubuleState::pristine();
            update_microtubule_state(&mut mt, 0.0, phospho, 1.0, &params);
            assert!(mt.dynamic_instability_index >= prev_dii - 1e-6,
                "DII должен монотонно расти при phospho={}: {:.4}", phospho, mt.dynamic_instability_index);
            prev_dii = mt.dynamic_instability_index;
        }
    }
}
