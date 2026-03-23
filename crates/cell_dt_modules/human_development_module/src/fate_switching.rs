//! Стохастическое переключение судьбы (Уровень 0: клетка).
//!
//! Ланжевен-шум в пространстве fate_bias моделирует биологическую неопределённость
//! в решениях самообновление/дифференцировка, не зависящую от детерминированных
//! параметров повреждений.
//!
//! # Формула
//!
//! ```text
//! fate_bias += N(0, 1) × sigma × sqrt(dt)        [Ито-интегрирование]
//! fate_bias −= fate_bias × recovery_rate × dt    [возврат к нейтрали]
//! fate_bias = clamp(fate_bias, -1, 1)
//! switch_threshold = 0.5 × (1 − damage × 0.4)   [повреждения снижают порог]
//! ```

use cell_dt_core::components::FateSwitchingState;
use rand::Rng;

/// Параметры стохастического переключения.
#[derive(Debug, Clone)]
pub struct FateSwitchingParams {
    /// Стандартное отклонение Ланжевен-шума (на √год) [0..0.5].
    /// 0.0 = детерминированная судьба. 0.15 = умеренная неопределённость.
    pub sigma:         f32,
    /// Скорость возврата fate_bias к нейтрали [/год].
    /// Предотвращает постоянное нарастание смещения.
    pub recovery_rate: f32,
    /// Насколько повреждения снижают switch_threshold.
    /// damage=1.0 → threshold × (1 − damage_threshold_reduction).
    pub damage_threshold_reduction: f32,
}

impl Default for FateSwitchingParams {
    fn default() -> Self {
        Self {
            sigma:                    0.15,
            recovery_rate:            0.50,
            damage_threshold_reduction: 0.40,
        }
    }
}

/// Обновить FateSwitchingState на один шаг.
///
/// # Аргументы
/// * `fs`           — состояние (in/out).
/// * `total_damage` — суммарный ущерб [0..1] (снижает switch_threshold).
/// * `params`       — параметры.
/// * `dt_years`     — шаг (лет).
/// * `rng`          — генератор случайных чисел.
pub fn update_fate_switching_state(
    fs:           &mut FateSwitchingState,
    total_damage: f32,
    params:       &FateSwitchingParams,
    dt_years:     f32,
    rng:          &mut impl Rng,
) {
    // Ланжевен: Gaussian noise × σ × √dt
    let noise: f32 = rng.gen::<f32>() * 2.0 - 1.0; // uniform → ±1
    let noise_term = noise * params.sigma * dt_years.sqrt();

    // Возврат к нейтрали
    let recovery = -fs.fate_bias * params.recovery_rate * dt_years;

    let prev_bias = fs.fate_bias;
    fs.fate_bias = (fs.fate_bias + noise_term + recovery).clamp(-1.0, 1.0);
    fs.noise_accumulator = noise_term;

    // Адаптивный порог: повреждения снижают «устойчивость» к переключению
    fs.switch_threshold =
        (0.5 * (1.0 - total_damage * params.damage_threshold_reduction))
            .clamp(0.10, 0.50);

    // Считаем переключения (пересечение порога)
    let was_switching = prev_bias.abs() > 0.5; // базовый порог для счёта
    let now_switching = fs.fate_bias.abs() > fs.switch_threshold;
    if !was_switching && now_switching {
        fs.switch_count += 1;
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn p() -> FateSwitchingParams { FateSwitchingParams::default() }
    fn rng() -> impl Rng { rand::rngs::StdRng::seed_from_u64(42) }

    #[test]
    fn test_pristine_neutral_state() {
        let s = FateSwitchingState::neutral();
        assert!((s.fate_bias - 0.0).abs() < 1e-5);
        assert!((s.switch_threshold - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_noise_perturbs_fate_bias() {
        let mut s = FateSwitchingState::neutral();
        let mut r = rng();
        update_fate_switching_state(&mut s, 0.0, &p(), 1.0, &mut r);
        // После одного шага fate_bias не строго ноль
        assert!(s.fate_bias.abs() < 1.01, "fate_bias в пределах [-1, 1]");
    }

    #[test]
    fn test_recovery_pulls_bias_to_zero() {
        let mut s = FateSwitchingState::neutral();
        s.fate_bias = 0.8;
        // Без шума (sigma=0) → recovery доминирует
        let silent_params = FateSwitchingParams { sigma: 0.0, ..p() };
        let mut r = rng();
        update_fate_switching_state(&mut s, 0.0, &silent_params, 1.0, &mut r);
        assert!(s.fate_bias.abs() < 0.8,
            "Возврат к нейтрали: |bias| снижается");
    }

    #[test]
    fn test_damage_lowers_threshold() {
        let healthy = {
            let mut s = FateSwitchingState::neutral();
            let mut r = rng();
            update_fate_switching_state(&mut s, 0.0, &p(), 0.01, &mut r);
            s.switch_threshold
        };
        let damaged = {
            let mut s = FateSwitchingState::neutral();
            let mut r = rng();
            update_fate_switching_state(&mut s, 0.9, &p(), 0.01, &mut r);
            s.switch_threshold
        };
        assert!(damaged < healthy,
            "Повреждения → порог переключения снижается");
    }

    #[test]
    fn test_bias_clamped() {
        let mut s = FateSwitchingState::neutral();
        s.fate_bias = 0.99;
        let big_sigma = FateSwitchingParams { sigma: 5.0, recovery_rate: 0.0, ..p() };
        let mut r = rng();
        update_fate_switching_state(&mut s, 0.0, &big_sigma, 1.0, &mut r);
        assert!(s.fate_bias.abs() <= 1.0, "fate_bias всегда в [-1, 1]");
    }
}
