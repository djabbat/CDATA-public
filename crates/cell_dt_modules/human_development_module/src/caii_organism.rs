//! CAII-индекс организма — уровень +3.
//!
//! Центриолярный Индекс Накопления Повреждений (CAII = Centriolar Accumulation
//! Index of Integrity) агрегируется по всем нишам организма.
//! Биологический возраст = хронологический × (1 + (1 − CAII) × 0.50).
//!
//! # Формула
//!
//! ```text
//! caii_organism = mean(niches_caii)   ; 1.0 если пусто
//! bio_age       = age_years × (1 + (1 − caii) × 0.50)
//! ```
//!
//! При CAII = 1.0 → bio_age = age (идеальное здоровье).
//! При CAII = 0.0 → bio_age = 1.5 × age (максимальное старение).

/// Вычислить CAII-индекс организма и биологический возраст.
///
/// # Аргументы
/// * `niches_caii` — CAII-значения всех живых ниш [0..1].
/// * `age_years`   — хронологический возраст [лет].
///
/// # Возврат
/// `(caii_organism, biological_age)`
pub fn compute_organism_caii(niches_caii: &[f32], age_years: f32) -> (f32, f32) {
    let caii = if niches_caii.is_empty() {
        1.0
    } else {
        niches_caii.iter().sum::<f32>() / niches_caii.len() as f32
    };
    let bio_age = age_years * (1.0 + (1.0 - caii) * 0.50);
    (caii, bio_age)
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// CAII = 1.0 (идеальный) → биологический возраст = хронологическому
    #[test]
    fn test_perfect_caii_biological_equals_chronological() {
        let niches = vec![1.0_f32; 10];
        let age = 40.0_f32;
        let (caii, bio_age) = compute_organism_caii(&niches, age);
        assert!((caii - 1.0).abs() < 1e-5, "CAII должен быть 1.0: {:.4}", caii);
        assert!((bio_age - age).abs() < 1e-4,
            "Bio age при CAII=1.0 должен = хронологическому: {:.2} vs {:.2}",
            bio_age, age);
    }

    /// CAII = 0.0 → биологический возраст = 1.5 × хронологическому
    #[test]
    fn test_zero_caii_biological_age_1_5x() {
        let niches = vec![0.0_f32; 5];
        let age = 60.0_f32;
        let (caii, bio_age) = compute_organism_caii(&niches, age);
        assert!((caii - 0.0).abs() < 1e-5, "CAII должен быть 0.0: {:.4}", caii);
        let expected = 60.0_f32 * 1.5;
        assert!((bio_age - expected).abs() < 1e-4,
            "Bio age при CAII=0: {:.2} vs {:.2}", bio_age, expected);
    }

    /// CAII = 0.5 → биологический возраст = 1.25 × хронологическому
    #[test]
    fn test_mid_caii_biological_age_1_25x() {
        let niches = vec![0.5_f32; 4];
        let age = 50.0_f32;
        let (caii, bio_age) = compute_organism_caii(&niches, age);
        assert!((caii - 0.5).abs() < 1e-5, "CAII должен быть 0.5: {:.4}", caii);
        let expected = 50.0_f32 * 1.25;
        assert!((bio_age - expected).abs() < 1e-4,
            "Bio age при CAII=0.5: {:.2} vs {:.2}", bio_age, expected);
    }

    /// Пустой список ниш → CAII = 1.0
    #[test]
    fn test_empty_niches_caii_one() {
        let niches: Vec<f32> = vec![];
        let age = 30.0_f32;
        let (caii, bio_age) = compute_organism_caii(&niches, age);
        assert!((caii - 1.0).abs() < 1e-5, "Пустые ниши → CAII = 1.0: {:.4}", caii);
        assert!((bio_age - age).abs() < 1e-4,
            "Пустые ниши → bio_age = age: {:.2}", bio_age);
    }
}
