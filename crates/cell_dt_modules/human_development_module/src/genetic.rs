//! Генетическая гетерогенность стволовых ниш (уровень 0: клетка).
//!
//! Применяет SNP-зависимые мультипликаторы к `DamageParams` каждой ниши,
//! обеспечивая популяционную вариабельность темпов молекулярного повреждения.
//!
//! # Принцип
//!
//! Вместо единого `DamageParams` для всего модуля каждая сущность может иметь
//! `GeneticProfile` — набор мультипликаторов, которые масштабируют базовые
//! параметры повреждения. При отсутствии профиля используются базовые параметры
//! (обратная совместимость).
//!
//! # Пример применения в популяционной симуляции
//!
//! ```text
//! 100 стволовых ниш HSC:
//!   60% — GeneticProfile::average()    → смерть ~78 лет
//!   15% — GeneticProfile::apoe4()      → смерть ~68 лет
//!   10% — GeneticProfile::lrrk2_g2019s() → сенесценция ~52 лет (паркинсонизм)
//!   15% — GeneticProfile::foxo3a_longevity() → смерть ~91 лет
//! ```
//!
//! # Мультипликативная схема
//!
//! effective_rate = base_rate × risk_modifier × longevity_factor
//!
//! Для субдистальных придатков (Ninein, CEP170):
//!   subdistal_modifier = 1.0 + (appendage_risk − 1.0) × 0.5
//! (половина эффекта, поскольку субдистальные придатки менее доступны для ROS).

use crate::damage::DamageParams;
use cell_dt_core::components::GeneticProfile;

// ─────────────────────────────────────────────────────────────────────────────
// Основная функция
// ─────────────────────────────────────────────────────────────────────────────

/// Применить генетические модификаторы к `DamageParams`.
///
/// Возвращает новый `DamageParams` со скорректированными rate-полями.
/// Не изменяет: `senescence_threshold`, `midlife_transition_center/width`,
/// `noise_scale`, `oh_sensitivity` (они не зависят от SNP-фона).
///
/// # Аргументы
/// * `base`    — базовые параметры (с учётом интервенций).
/// * `profile` — генетический профиль сущности.
pub fn apply_genetic_modifiers(base: &DamageParams, profile: &GeneticProfile) -> DamageParams {
    let mut p = base.clone();
    let lf = profile.longevity_factor;   // применяется ко всем rate-полям

    // ── Молекулярные повреждения ─────────────────────────────────────────────
    p.base_ros_damage_rate       *= profile.carbonylation_risk * lf;
    p.acetylation_rate           *= profile.acetylation_risk   * lf;
    p.aggregation_rate           *= profile.aggregation_risk   * lf;
    p.phospho_dysregulation_rate *= profile.phospho_risk       * lf;

    // ── Дистальные придатки (CEP164/CEP89): полный appendage_risk ────────────
    let distal_mod = profile.appendage_risk * lf;
    p.cep164_loss_rate *= distal_mod;
    p.cep89_loss_rate  *= distal_mod;

    // ── Субдистальные придатки (Ninein/CEP170): половина эффекта ────────────
    // Биологически: субдистальные находятся глубже → меньше ROS-доступность.
    let subdistal_mod = 1.0 + (profile.appendage_risk - 1.0) * 0.5 * lf;
    p.ninein_loss_rate  *= subdistal_mod;
    p.cep170_loss_rate  *= subdistal_mod;

    // ── Петля обратной связи ROS ─────────────────────────────────────────────
    // ros_feedback_risk НЕ масштабируется longevity_factor — это структурный
    // параметр митохондриального фенотипа, независимый от общей longevity.
    p.ros_feedback_coefficient *= profile.ros_feedback_risk;

    // ── Репарация: защитные варианты усиливают репарацию ────────────────────
    // longevity_factor < 1.0 → ниши чуть эффективнее репарируют придатки.
    if lf < 1.0 {
        let repair_boost = 1.0 + (1.0 - lf) * 0.5; // max +25% при lf=0.5
        p.cep164_repair_rate *= repair_boost;
        p.cep89_repair_rate  *= repair_boost;
        p.ninein_repair_rate *= repair_boost;
        p.cep170_repair_rate *= repair_boost;
    }

    p
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::components::GeneticProfile;

    fn base() -> DamageParams { DamageParams::default() }

    /// Средний профиль — идентичное отображение
    #[test]
    fn test_average_profile_is_identity() {
        let profile = GeneticProfile::average();
        let base = base();
        let effective = apply_genetic_modifiers(&base, &profile);
        assert!((effective.base_ros_damage_rate - base.base_ros_damage_rate).abs() < 1e-6,
            "Average profile не должен изменять rates");
        assert!((effective.aggregation_rate - base.aggregation_rate).abs() < 1e-6);
        assert!((effective.ros_feedback_coefficient - base.ros_feedback_coefficient).abs() < 1e-6);
        assert!((effective.cep164_loss_rate - base.cep164_loss_rate).abs() < 1e-6);
    }

    /// APOE4 повышает carbonylation и ros_feedback
    #[test]
    fn test_apoe4_increases_ros_rates() {
        let avg  = apply_genetic_modifiers(&base(), &GeneticProfile::average());
        let apoe4 = apply_genetic_modifiers(&base(), &GeneticProfile::apoe4());
        assert!(apoe4.base_ros_damage_rate > avg.base_ros_damage_rate,
            "APOE4: carbonylation rate должен расти");
        assert!(apoe4.ros_feedback_coefficient > avg.ros_feedback_coefficient,
            "APOE4: ros_feedback должен расти");
        assert!(apoe4.aggregation_rate > avg.aggregation_rate,
            "APOE4: aggregation rate должен расти");
    }

    /// FOXO3a снижает все rate-поля через longevity_factor
    #[test]
    fn test_foxo3a_reduces_all_rates() {
        let avg    = apply_genetic_modifiers(&base(), &GeneticProfile::average());
        let foxo3a = apply_genetic_modifiers(&base(), &GeneticProfile::foxo3a_longevity());
        assert!(foxo3a.base_ros_damage_rate < avg.base_ros_damage_rate,
            "FOXO3a: carbonylation должен снижаться");
        assert!(foxo3a.aggregation_rate < avg.aggregation_rate,
            "FOXO3a: aggregation должен снижаться");
        assert!(foxo3a.cep164_loss_rate < avg.cep164_loss_rate,
            "FOXO3a: CEP164 loss rate должен снижаться");
    }

    /// LRRK2-G2019S повышает phospho и aggregation
    #[test]
    fn test_lrrk2_raises_phospho_and_aggregation() {
        let avg   = apply_genetic_modifiers(&base(), &GeneticProfile::average());
        let lrrk2 = apply_genetic_modifiers(&base(), &GeneticProfile::lrrk2_g2019s());
        assert!(lrrk2.phospho_dysregulation_rate > avg.phospho_dysregulation_rate,
            "LRRK2: phospho rate должен расти");
        assert!(lrrk2.aggregation_rate > avg.aggregation_rate,
            "LRRK2: aggregation rate должен расти");
        // phospho_risk=1.40 > carbonylation_risk=1.10
        assert!(lrrk2.phospho_dysregulation_rate / avg.phospho_dysregulation_rate >
                lrrk2.base_ros_damage_rate / avg.base_ros_damage_rate,
            "LRRK2: phospho эффект должен быть сильнее carbonylation");
    }

    /// Субдистальные придатки менее чувствительны к appendage_risk
    #[test]
    fn test_subdistal_appendages_less_sensitive_than_distal() {
        // Создадим профиль с высоким appendage_risk
        let mut custom = GeneticProfile::average();
        custom.appendage_risk = 1.8;
        let base = base();
        let eff = apply_genetic_modifiers(&base, &custom);

        let cep164_ratio = eff.cep164_loss_rate / base.cep164_loss_rate; // = 1.8
        let ninein_ratio = eff.ninein_loss_rate  / base.ninein_loss_rate; // = 1 + 0.8×0.5 = 1.4
        assert!(ninein_ratio < cep164_ratio,
            "Ninein менее чувствителен: {:.3} < {:.3}", ninein_ratio, cep164_ratio);
        let cep170_ratio = eff.cep170_loss_rate / base.cep170_loss_rate;
        assert!(cep170_ratio < cep164_ratio,
            "CEP170 менее чувствителен: {:.3} < {:.3}", cep170_ratio, cep164_ratio);
    }

    /// SOD2 Ala16Val повышает ros_feedback_coefficient
    #[test]
    fn test_sod2_raises_ros_feedback() {
        let avg  = apply_genetic_modifiers(&base(), &GeneticProfile::average());
        let sod2 = apply_genetic_modifiers(&base(), &GeneticProfile::sod2_ala16val());
        assert!(sod2.ros_feedback_coefficient > avg.ros_feedback_coefficient,
            "SOD2-Ala16Val: ros_feedback должен расти");
        // ros_feedback НЕ масштабируется longevity_factor (у SOD2 lf=1.0)
        let expected = avg.ros_feedback_coefficient * 1.25;
        assert!((sod2.ros_feedback_coefficient - expected).abs() < 1e-5,
            "SOD2: ros_feedback = ×1.25: {:.5} vs {:.5}",
            sod2.ros_feedback_coefficient, expected);
    }

    /// FOXO3a усиливает репарацию придатков
    #[test]
    fn test_foxo3a_boosts_appendage_repair() {
        // Нужны ненулевые базовые repair rates
        let mut b = base();
        b.cep164_repair_rate = 0.01;
        b.cep89_repair_rate  = 0.005;

        let avg    = apply_genetic_modifiers(&b, &GeneticProfile::average());
        let foxo3a = apply_genetic_modifiers(&b, &GeneticProfile::foxo3a_longevity());
        assert!(foxo3a.cep164_repair_rate > avg.cep164_repair_rate,
            "FOXO3a: repair rate должен расти: {:.5} vs {:.5}",
            foxo3a.cep164_repair_rate, avg.cep164_repair_rate);
    }

    /// Немутируемые поля (senescence_threshold, noise_scale) не изменяются
    #[test]
    fn test_immutable_fields_unchanged() {
        let base = base();
        let profile = GeneticProfile::lrrk2_g2019s();
        let eff = apply_genetic_modifiers(&base, &profile);
        assert!((eff.senescence_threshold - base.senescence_threshold).abs() < 1e-6,
            "senescence_threshold не должен изменяться");
        assert!((eff.noise_scale - base.noise_scale).abs() < 1e-6,
            "noise_scale не должен изменяться");
        assert!((eff.midlife_damage_multiplier - base.midlife_damage_multiplier).abs() < 1e-6,
            "midlife_damage_multiplier не должен изменяться");
    }

    /// Монотонность: APOE4 > average > APOE2 по суммарному риску
    #[test]
    fn test_apoe_risk_ordering() {
        let b = base();
        let apoe4 = apply_genetic_modifiers(&b, &GeneticProfile::apoe4());
        let avg   = apply_genetic_modifiers(&b, &GeneticProfile::average());
        let apoe2 = apply_genetic_modifiers(&b, &GeneticProfile::apoe2());

        assert!(apoe4.base_ros_damage_rate > avg.base_ros_damage_rate,
            "APOE4 > average (carbonylation)");
        assert!(avg.base_ros_damage_rate > apoe2.base_ros_damage_rate,
            "average > APOE2 (carbonylation)");
    }
}
