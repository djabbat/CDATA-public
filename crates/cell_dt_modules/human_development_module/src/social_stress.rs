//! Социальный стресс и биологическое старение — уровень +4.
//!
//! Социальная изоляция (одиночество) и социально-экономический стресс ускоряют
//! старение через ось HPA (кортизол) и подавление окситоцина.
//! Социальная сплочённость → окситоцин → противовоспалительный эффект.
//!
//! # Механизм
//!
//! ```text
//! social_cohesion = 1.0 − loneliness_index
//! oxytocin = cohesion × cohesion_oxytocin_factor + 0.20  (базальный уровень 0.20)
//!
//! allostatic_load ↑ ← (loneliness × 0.60 + socioeconomic × 0.40)
//!                      × allostatic_accumulation × dt
//!
//! Кортизол-буст = loneliness × loneliness_cortisol_factor
//! SASP-снижение = oxytocin × oxytocin_antiinflam
//! ```

use serde::{Deserialize, Serialize};
use cell_dt_core::components::SocialStressState;

// ─────────────────────────────────────────────────────────────────────────────
// Параметры
// ─────────────────────────────────────────────────────────────────────────────

/// Параметры социального стресса.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialStressParams {
    /// Коэффициент кортизольного эффекта одиночества.
    pub loneliness_cortisol_factor: f32,
    /// Коэффициент продукции окситоцина от социальной сплочённости.
    pub cohesion_oxytocin_factor: f32,
    /// Противовоспалительный коэффициент окситоцина.
    pub oxytocin_antiinflam: f32,
    /// Скорость накопления аллостатической нагрузки [/год].
    pub allostatic_accumulation: f32,
}

impl Default for SocialStressParams {
    fn default() -> Self {
        Self {
            loneliness_cortisol_factor: 0.25,
            cohesion_oxytocin_factor: 0.20,
            oxytocin_antiinflam: 0.15,
            allostatic_accumulation: 0.0001,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Основная функция обновления
// ─────────────────────────────────────────────────────────────────────────────

/// Обновить SocialStressState за один шаг.
///
/// `state.loneliness_index` и `state.socioeconomic_stress` устанавливаются
/// снаружи перед вызовом (из внешних данных или интервенций).
///
/// # Аргументы
/// * `state`  — изменяемый SocialStressState.
/// * `params` — параметры социального стресса.
/// * `dt`     — шаг времени [лет].
pub fn update_social_stress_state(
    state: &mut SocialStressState,
    params: &SocialStressParams,
    dt: f32,
) {
    // Социальная сплочённость — обратная к одиночеству
    state.social_cohesion = 1.0 - state.loneliness_index;

    // Окситоцин: базальный 0.20 + вклад сплочённости
    state.oxytocin_level = state.social_cohesion * params.cohesion_oxytocin_factor + 0.20;

    // Аллостатическая нагрузка накапливается от одиночества и соц-экон. стресса
    state.allostatic_load += (state.loneliness_index * 0.60
        + state.socioeconomic_stress * 0.40)
        * params.allostatic_accumulation
        * dt;
    state.allostatic_load = state.allostatic_load.clamp(0.0, 1.0);
}

/// Дополнительный кортизол-буст от одиночества.
///
/// Подаётся на HPAAxisState или ROS-каскад как дополнительный стрессор.
pub fn social_cortisol_boost(
    state: &SocialStressState,
    params: &SocialStressParams,
) -> f32 {
    state.loneliness_index * params.loneliness_cortisol_factor
}

/// Снижение SASP благодаря окситоцину.
///
/// Противовоспалительный эффект: oxytocin × oxytocin_antiinflam.
pub fn social_sasp_reduction(
    state: &SocialStressState,
    params: &SocialStressParams,
) -> f32 {
    state.oxytocin_level * params.oxytocin_antiinflam
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 365.25;

    /// Одиночество → низкая сплочённость, накапливается аллостатическая нагрузка
    #[test]
    fn test_lonely_accumulates_allostatic_load() {
        let mut state = SocialStressState::default();
        state.loneliness_index = 0.80;
        let params = SocialStressParams::default();

        // 20 лет одиночества
        for _ in 0..(20 * 365) {
            update_social_stress_state(&mut state, &params, DT);
        }

        assert!(state.allostatic_load > 0.0,
            "Одиночество должно накапливать аллостатическую нагрузку: {:.6}",
            state.allostatic_load);
        assert!((state.social_cohesion - 0.20).abs() < 1e-5,
            "Cohesion = 1 − loneliness: {:.4}", state.social_cohesion);
    }

    /// Социально включённый → высокий окситоцин
    #[test]
    fn test_connected_high_oxytocin() {
        let mut state = SocialStressState::default();
        state.loneliness_index = 0.0; // полностью социально включён
        let params = SocialStressParams::default();
        update_social_stress_state(&mut state, &params, DT);

        // oxytocin = 1.0 × 0.20 + 0.20 = 0.40
        let expected = 1.0_f32 * 0.20 + 0.20;
        assert!((state.oxytocin_level - expected).abs() < 1e-5,
            "Высокий окситоцин при включённости: {:.4} vs {:.4}",
            state.oxytocin_level, expected);
    }

    /// Окситоцин снижает SASP при высокой сплочённости
    #[test]
    fn test_oxytocin_reduces_sasp() {
        let mut state = SocialStressState::default();
        state.loneliness_index = 0.0; // высокая сплочённость
        let params = SocialStressParams::default();
        update_social_stress_state(&mut state, &params, DT);

        let reduction = social_sasp_reduction(&state, &params);
        assert!(reduction > 0.0,
            "Окситоцин должен снижать SASP: {:.4}", reduction);
    }

    /// Одиночество увеличивает кортизол-буст
    #[test]
    fn test_loneliness_increases_cortisol() {
        let mut state_alone = SocialStressState::default();
        state_alone.loneliness_index = 0.70;
        let mut state_connected = SocialStressState::default();
        state_connected.loneliness_index = 0.05;
        let params = SocialStressParams::default();

        let cortisol_alone = social_cortisol_boost(&state_alone, &params);
        let cortisol_connected = social_cortisol_boost(&state_connected, &params);

        assert!(cortisol_alone > cortisol_connected,
            "Одиночество↑ → больше кортизола: {:.4} vs {:.4}",
            cortisol_alone, cortisol_connected);
        assert!(cortisol_alone > 0.0,
            "Одиночество создаёт кортизол-буст: {:.4}", cortisol_alone);
    }

    /// Аллостатическая нагрузка накапливается со временем
    #[test]
    fn test_allostatic_load_accumulates_over_time() {
        let mut state = SocialStressState::default();
        state.loneliness_index = 0.50;
        state.socioeconomic_stress = 0.30;
        let params = SocialStressParams::default();

        let load_initial = state.allostatic_load;
        for _ in 0..(100 * 365) {
            update_social_stress_state(&mut state, &params, DT);
        }

        assert!(state.allostatic_load > load_initial,
            "Аллостатическая нагрузка должна расти: {:.6} -> {:.6}",
            load_initial, state.allostatic_load);
    }
}
