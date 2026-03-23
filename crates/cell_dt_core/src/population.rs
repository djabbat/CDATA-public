//! Популяционный режим — P50.
//!
//! Симулирует когорту организмов (каждый = набор ECS-сущностей с индивидуальными
//! генетическими профилями), собирает распределение CAII в когорте,
//! сравнивает с клиническими данными WP1 (n=240).

use std::collections::HashMap;

/// Результат одного организма в когорте.
#[derive(Debug, Clone)]
pub struct OrganismResult {
    pub organism_id: usize,
    pub genetic_variant: String,    // "average", "apoe4", "foxo3a", etc.
    pub lifespan_years: f32,
    pub caii_at_40: f32,
    pub caii_at_60: f32,
    pub caii_at_80: f32,
    pub biological_age_at_death: f32,
    pub death_cause: String,
    pub inflammaging_index_peak: f32,
}

/// Статистика когорты.
#[derive(Debug, Clone)]
pub struct CohortStatistics {
    pub n: usize,
    pub mean_lifespan: f32,
    pub sd_lifespan: f32,
    pub mean_caii_at_60: f32,
    pub sd_caii_at_60: f32,
    pub percentile_5_lifespan: f32,
    pub percentile_95_lifespan: f32,
    pub fraction_reaching_80: f32,
    pub fraction_reaching_90: f32,
}

impl CohortStatistics {
    pub fn from_results(results: &[OrganismResult]) -> Self {
        let n = results.len();
        if n == 0 { return Self::default(); }

        let lifespans: Vec<f32> = results.iter().map(|r| r.lifespan_years).collect();
        let caii60: Vec<f32> = results.iter().map(|r| r.caii_at_60).collect();

        let mean_lifespan = lifespans.iter().sum::<f32>() / n as f32;
        let sd_lifespan = {
            let var = lifespans.iter().map(|x| (x - mean_lifespan).powi(2)).sum::<f32>() / n as f32;
            var.sqrt()
        };
        let mean_caii_at_60 = caii60.iter().sum::<f32>() / n as f32;
        let sd_caii_at_60 = {
            let var = caii60.iter().map(|x| (x - mean_caii_at_60).powi(2)).sum::<f32>() / n as f32;
            var.sqrt()
        };

        let mut sorted = lifespans.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p5 = sorted[(n as f32 * 0.05) as usize];
        let p95 = sorted[((n as f32 * 0.95) as usize).min(n - 1)];
        let frac80 = lifespans.iter().filter(|&&x| x >= 80.0).count() as f32 / n as f32;
        let frac90 = lifespans.iter().filter(|&&x| x >= 90.0).count() as f32 / n as f32;

        Self {
            n,
            mean_lifespan,
            sd_lifespan,
            mean_caii_at_60,
            sd_caii_at_60,
            percentile_5_lifespan: p5,
            percentile_95_lifespan: p95,
            fraction_reaching_80: frac80,
            fraction_reaching_90: frac90,
        }
    }

    /// Статистика по генетическому варианту.
    pub fn by_variant(results: &[OrganismResult]) -> HashMap<String, CohortStatistics> {
        let mut groups: HashMap<String, Vec<OrganismResult>> = HashMap::new();
        for r in results {
            groups.entry(r.genetic_variant.clone()).or_default().push(r.clone());
        }
        groups.iter().map(|(k, v)| (k.clone(), Self::from_results(v))).collect()
    }
}

impl Default for CohortStatistics {
    fn default() -> Self {
        Self {
            n: 0,
            mean_lifespan: 0.0,
            sd_lifespan: 0.0,
            mean_caii_at_60: 0.0,
            sd_caii_at_60: 0.0,
            percentile_5_lifespan: 0.0,
            percentile_95_lifespan: 0.0,
            fraction_reaching_80: 0.0,
            fraction_reaching_90: 0.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(id: usize, variant: &str, lifespan: f32, caii60: f32) -> OrganismResult {
        OrganismResult {
            organism_id: id,
            genetic_variant: variant.to_string(),
            lifespan_years: lifespan,
            caii_at_40: 0.9,
            caii_at_60: caii60,
            caii_at_80: 0.5,
            biological_age_at_death: lifespan * 1.1,
            death_cause: "frailty".to_string(),
            inflammaging_index_peak: 0.5,
        }
    }

    /// cohort_stats_empty — пустой вектор → default statistics
    #[test]
    fn cohort_stats_empty() {
        let stats = CohortStatistics::from_results(&[]);
        assert_eq!(stats.n, 0);
        assert_eq!(stats.mean_lifespan, 0.0);
        assert_eq!(stats.sd_lifespan, 0.0);
        assert_eq!(stats.fraction_reaching_80, 0.0);
    }

    /// cohort_stats_single — один организм, верные значения
    #[test]
    fn cohort_stats_single() {
        let results = vec![make_result(0, "average", 75.0, 0.70)];
        let stats = CohortStatistics::from_results(&results);
        assert_eq!(stats.n, 1);
        assert!((stats.mean_lifespan - 75.0).abs() < 1e-4);
        assert!((stats.mean_caii_at_60 - 0.70).abs() < 1e-4);
        // SD = 0 для одного элемента
        assert!(stats.sd_lifespan < 1e-4);
        assert_eq!(stats.fraction_reaching_80, 0.0);
        assert_eq!(stats.fraction_reaching_90, 0.0);
    }

    /// cohort_stats_mean — несколько организмов, правильное среднее
    #[test]
    fn cohort_stats_mean() {
        let results = vec![
            make_result(0, "average", 70.0, 0.60),
            make_result(1, "average", 80.0, 0.70),
            make_result(2, "average", 90.0, 0.80),
        ];
        let stats = CohortStatistics::from_results(&results);
        assert_eq!(stats.n, 3);
        assert!((stats.mean_lifespan - 80.0).abs() < 1e-4,
            "mean_lifespan = {:.4}", stats.mean_lifespan);
        assert!((stats.mean_caii_at_60 - (0.60 + 0.70 + 0.80) / 3.0).abs() < 1e-4,
            "mean_caii_at_60 = {:.4}", stats.mean_caii_at_60);
    }

    /// cohort_stats_percentiles — p5 и p95 корректны
    #[test]
    fn cohort_stats_percentiles() {
        // 20 организмов с равным шагом 60..79
        let results: Vec<OrganismResult> = (0..20)
            .map(|i| make_result(i, "average", 60.0 + i as f32, 0.7))
            .collect();
        let stats = CohortStatistics::from_results(&results);
        // p5 = sorted[20*0.05] = sorted[1] = 61.0
        assert!((stats.percentile_5_lifespan - 61.0).abs() < 1.0,
            "p5 = {:.2}", stats.percentile_5_lifespan);
        // p95 = sorted[min(19,19)] = sorted[19] = 79.0
        assert!((stats.percentile_95_lifespan - 79.0).abs() < 1.0,
            "p95 = {:.2}", stats.percentile_95_lifespan);
        // p95 > p5
        assert!(stats.percentile_95_lifespan > stats.percentile_5_lifespan);
    }

    /// cohort_stats_fractions — fraction_reaching_80/90 считаются верно
    #[test]
    fn cohort_stats_fractions() {
        let results = vec![
            make_result(0, "average", 70.0, 0.7),  // < 80
            make_result(1, "average", 82.0, 0.7),  // >= 80, < 90
            make_result(2, "average", 85.0, 0.7),  // >= 80, < 90
            make_result(3, "average", 91.0, 0.7),  // >= 90
        ];
        let stats = CohortStatistics::from_results(&results);
        assert!((stats.fraction_reaching_80 - 3.0 / 4.0).abs() < 1e-4,
            "frac80 = {:.4}", stats.fraction_reaching_80);
        assert!((stats.fraction_reaching_90 - 1.0 / 4.0).abs() < 1e-4,
            "frac90 = {:.4}", stats.fraction_reaching_90);
    }
}
