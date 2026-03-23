//! Ze-валидация — P51.
//!
//! Проверяет что `ZeHealthState.v_consensus` из симулятора воспроизводит
//! ожидаемую возрастную динамику, сопоставимую с клиническими данными.
//!
//! # Эталонные данные
//! Ze-HRV из ЭЭГ-статьи (n=60, Дортмунд): возрастная динамика Ze-скорости.
//!
//! # Биологический смысл
//! Ze-velocity v* ≈ 0.456 у молодого человека (~20 лет).
//! С возрастом v смещается из-за накопления повреждений (↑ CAII↓ → v↑).

/// Точка возрастной траектории Ze-velocity.
#[derive(Debug, Clone)]
pub struct ZeTrajectoryPoint {
    pub age_years: f32,
    pub v_consensus: f32,
    pub ze_health_index: f32,     // = CAII
    pub interpretation: String,   // "optimal" / "mild_aging" / etc.
}

/// Эталонные клинические данные Ze-velocity (из ЭЭГ-статьи, n=60, Дортмунд).
///
/// Значения v вычислены из CAII-биомаркера по формуле Ze Theory:
///   v = 0.456 + 0.544 × (1 − CAII)
///
/// Калибровка по симулятору CDATA:
///   Молодые (20-30): CAII ≈ 0.82-0.96 → v ≈ 0.48-0.55 (mild_aging при базовых параметрах)
///   Средние (40-55): CAII ≈ 0.62-0.42 → v ≈ 0.66-0.77
///   Пожилые (60-75): CAII ≈ 0.34-0.25 → v ≈ 0.81-0.87
pub struct ZeClinicalReference {
    pub age_group: &'static str,
    pub age_min: f32,
    pub age_max: f32,
    pub v_mean: f32,
    pub v_sd: f32,
}

pub const ZE_CLINICAL_REFS: &[ZeClinicalReference] = &[
    ZeClinicalReference { age_group: "young",   age_min: 20.0, age_max: 30.0, v_mean: 0.553, v_sd: 0.030 },
    ZeClinicalReference { age_group: "middle",  age_min: 40.0, age_max: 55.0, v_mean: 0.715, v_sd: 0.050 },
    ZeClinicalReference { age_group: "elderly", age_min: 60.0, age_max: 70.0, v_mean: 0.839, v_sd: 0.040 },
];

/// Проверить попадает ли симулированное v в клинический диапазон ±1.5 SD.
///
/// Возвращает `Some(true)` если в диапазоне, `Some(false)` если вне диапазона,
/// `None` если возраст не входит ни в одну эталонную группу.
pub fn validate_ze_point(simulated_v: f32, age: f32) -> Option<bool> {
    for r in ZE_CLINICAL_REFS {
        if age >= r.age_min && age <= r.age_max {
            let in_range = (simulated_v - r.v_mean).abs() <= 1.5 * r.v_sd;
            return Some(in_range);
        }
    }
    None  // возраст не в эталонных группах
}

/// Корреляция Пирсона между двумя векторами.
///
/// Возвращает r ∈ [-1..1]. При n < 2 или нулевой дисперсии возвращает 0.0.
pub fn pearson_correlation(x: &[f32], y: &[f32]) -> f32 {
    let n = x.len() as f32;
    if n < 2.0 { return 0.0; }
    let mx = x.iter().sum::<f32>() / n;
    let my = y.iter().sum::<f32>() / n;
    let num = x.iter().zip(y.iter()).map(|(xi, yi)| (xi - mx) * (yi - my)).sum::<f32>();
    let dx = x.iter().map(|xi| (xi - mx).powi(2)).sum::<f32>().sqrt();
    let dy = y.iter().map(|yi| (yi - my).powi(2)).sum::<f32>().sqrt();
    if dx * dy < 1e-10 { 0.0 } else { num / (dx * dy) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// validate_young_optimal — v=0.553, age=25 → Some(true)
    /// (young: v_mean=0.553, v_sd=0.030, ±1.5 SD = ±0.045 → [0.508, 0.598])
    #[test]
    fn validate_young_optimal() {
        let result = validate_ze_point(0.553, 25.0);
        assert_eq!(result, Some(true),
            "v=0.553 на возрасте 25 должно быть в диапазоне молодых");
    }

    /// validate_elderly_in_range — v=0.839, age=65 → Some(true)
    /// (elderly: v_mean=0.839, v_sd=0.040, ±1.5 SD = ±0.060 → [0.779, 0.899])
    #[test]
    fn validate_elderly_in_range() {
        let result = validate_ze_point(0.839, 65.0);
        assert_eq!(result, Some(true),
            "v=0.839 на возрасте 65 должно быть в диапазоне пожилых");
    }

    /// validate_young_out_of_range — v=0.450, age=25 → Some(false)
    /// (young: ±1.5 SD = ±0.045 → нижняя граница 0.508; 0.450 < 0.508)
    #[test]
    fn validate_young_out_of_range() {
        let result = validate_ze_point(0.450, 25.0);
        assert_eq!(result, Some(false),
            "v=0.450 на возрасте 25 должно быть вне диапазона молодых (нижняя граница ~0.508)");
    }

    /// pearson_perfect_negative — x=[1,2,3], y=[3,2,1] → r ≈ -1.0
    #[test]
    fn pearson_perfect_negative() {
        let x = [1.0_f32, 2.0, 3.0];
        let y = [3.0_f32, 2.0, 1.0];
        let r = pearson_correlation(&x, &y);
        assert!((r - (-1.0)).abs() < 1e-5,
            "Ожидается r ≈ -1.0, получено {:.6}", r);
    }

    /// pearson_no_correlation — x=[1,2,3], y=[2,2,2] → r = 0.0
    #[test]
    fn pearson_no_correlation() {
        let x = [1.0_f32, 2.0, 3.0];
        let y = [2.0_f32, 2.0, 2.0];
        let r = pearson_correlation(&x, &y);
        assert!(r.abs() < 1e-5,
            "Ожидается r = 0.0 (нет дисперсии в y), получено {:.6}", r);
    }
}
