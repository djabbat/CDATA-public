//! HPA-ось (уровень +3: организм — нейроэндокринная регуляция).
//!
//! Хронический стресс → кортизол↑ → иммуносупрессия → нарушение SASP-клиренса.
//! Кортизол > 0.6 → снижение NK-активности → старые клетки не элиминируются.
//!
//! # Формулы
//!
//! ```text
//! d(cortisol)/dt = (stress_input × reactivity − cortisol × recovery) × dt
//! d(chronic_stress)/dt = cortisol × 0.10 × dt     (необратимое накопление)
//! reactivity *= (1 − cortisol × 0.05 × dt)        (выгорание оси)
//! ```

use cell_dt_core::components::HPAAxisState;

/// Параметры HPA-оси.
#[derive(Debug, Clone)]
pub struct HPAAxisParams {
    /// Скорость восстановления кортизола к базальному [/год].
    pub cortisol_recovery_rate: f32,
    /// Скорость выгорания (истощения) реактивности оси при хрон. активации.
    pub reactivity_burnout_rate: f32,
    /// Скорость накопления хронического стресса от кортизола.
    pub chronic_accumulation_rate: f32,
}

impl Default for HPAAxisParams {
    fn default() -> Self {
        Self {
            cortisol_recovery_rate:    2.0,
            reactivity_burnout_rate:   0.05,
            chronic_accumulation_rate: 0.10,
        }
    }
}

/// Обновить HPAAxisState на один шаг.
///
/// # Аргументы
/// * `hpa`          — состояние (in/out).
/// * `stress_input` — входящий психологический/физиологический стресс [0..1].
/// * `params`       — параметры.
/// * `dt_years`     — шаг (лет).
pub fn update_hpa_axis_state(
    hpa:          &mut HPAAxisState,
    stress_input: f32,
    params:       &HPAAxisParams,
    dt_years:     f32,
) {
    // Кортизол: стресс → повышает, recovery → снижает
    let cortisol_delta = stress_input * hpa.hpa_reactivity
        - params.cortisol_recovery_rate * (hpa.cortisol_level - 0.20).max(0.0);
    hpa.cortisol_level =
        (hpa.cortisol_level + cortisol_delta * dt_years).clamp(0.10, 1.0);

    // Хронический стресс: необратимо накапливается
    hpa.chronic_stress_index =
        (hpa.chronic_stress_index
            + hpa.cortisol_level * params.chronic_accumulation_rate * dt_years)
            .clamp(0.0, 1.0);

    // Реактивность: выгорает при хронической активации
    hpa.hpa_reactivity =
        (hpa.hpa_reactivity
            - hpa.cortisol_level * params.reactivity_burnout_rate * dt_years)
            .clamp(0.10, 1.0);
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> HPAAxisParams { HPAAxisParams::default() }

    #[test]
    fn test_pristine_moderate_cortisol() {
        let s = HPAAxisState::pristine();
        assert!((s.cortisol_level - 0.30).abs() < 1e-5);
    }

    #[test]
    fn test_stress_raises_cortisol() {
        let mut s = HPAAxisState::pristine();
        update_hpa_axis_state(&mut s, 1.0, &p(), 1.0);
        assert!(s.cortisol_level > 0.30,
            "Стресс → кортизол растёт");
    }

    #[test]
    fn test_no_stress_recovers_cortisol() {
        let mut s = HPAAxisState::pristine();
        s.cortisol_level = 0.80;
        update_hpa_axis_state(&mut s, 0.0, &p(), 2.0);
        assert!(s.cortisol_level < 0.80,
            "Без стресса → кортизол снижается");
    }

    #[test]
    fn test_chronic_stress_accumulates() {
        let mut s = HPAAxisState::pristine();
        for _ in 0..10 {
            update_hpa_axis_state(&mut s, 0.8, &p(), 1.0);
        }
        assert!(s.chronic_stress_index > 0.0,
            "Хронический стресс → необратимое накопление");
    }

    #[test]
    fn test_reactivity_burnout_under_chronic_stress() {
        let mut s = HPAAxisState::pristine();
        for _ in 0..20 {
            update_hpa_axis_state(&mut s, 1.0, &p(), 1.0);
        }
        assert!(s.hpa_reactivity < 0.70,
            "Хронический стресс → выгорание оси");
    }
}
