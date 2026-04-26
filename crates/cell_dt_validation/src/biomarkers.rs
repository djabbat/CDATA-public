use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomarkerType {
    RosLevel,
    MtdnaMutations,
    ChipFrequency,
    EpigeneticClock,
    StemCellPool,
    TelomereLength,
    FrailtyIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomarkerDataPoint {
    pub age: f64,
    pub value: f64,
    pub std_dev: f64,
    pub n_samples: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomarkerDataset {
    pub name: String,
    pub biomarker_type: BiomarkerType,
    pub values: Vec<BiomarkerDataPoint>,
    pub source_pmid: Option<u32>,
}

impl BiomarkerDataset {
    pub fn new(name: &str, biomarker_type: BiomarkerType) -> Self {
        Self { name: name.to_string(), biomarker_type, values: Vec::new(), source_pmid: None }
    }

    pub fn add_point(&mut self, age: f64, value: f64, std_dev: f64, n_samples: u32) {
        self.values.push(BiomarkerDataPoint { age, value, std_dev, n_samples });
    }

    pub fn max_age(&self) -> f64 {
        self.values.iter().map(|p| p.age).fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn min_age(&self) -> f64 {
        self.values.iter().map(|p| p.age).fold(f64::INFINITY, f64::min)
    }

    pub fn synthetic_chip_frequency() -> Self {
        let mut ds = Self::new("Synthetic CHIP Frequency", BiomarkerType::ChipFrequency);
        // FIX Round 7 (B3): Recalibrated VAF to match Jaiswal SS et al. 2017 (PMID: 28636844)
        // NEJM 2017: VAF>0.02 in ~2% at age 40, ~10% at 65, rare >0.10 at age 70
        // Previous values were 2–4× too high (70yo: 0.20 → corrected 0.07)
        // PMID correction 2026-04-21: prior 28792876 → correct 28636844 (Jaiswal NEJM "Clonal Hematopoiesis and Risk of Atherosclerotic CVD")
        ds.source_pmid = Some(28636844);  // Jaiswal 2017 NEJM 377(2):111-121
        for (age, val, std, n) in [(40.0, 0.005, 0.002, 500u32), (50.0, 0.015, 0.005, 600),
                                    (60.0, 0.040, 0.012, 700), (70.0, 0.070, 0.020, 500),
                                    (80.0, 0.120, 0.035, 300)] {
            ds.add_point(age, val, std, n);
        }
        ds
    }

    pub fn synthetic_ros() -> Self {
        let mut ds = Self::new("Synthetic ROS by Age", BiomarkerType::RosLevel);
        for (age, val, std, n) in [(20.0, 0.15, 0.03, 100u32), (40.0, 0.25, 0.05, 150),
                                    (60.0, 0.45, 0.08, 200), (80.0, 0.65, 0.10, 180)] {
            ds.add_point(age, val, std, n);
        }
        ds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_chip() {
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        assert_eq!(ds.values.len(), 5);
        assert!(ds.min_age() < ds.max_age());
    }

    // ── BiomarkerDataset construction ─────────────────────────────────────────

    #[test]
    fn test_new_empty_dataset() {
        let ds = BiomarkerDataset::new("TestBiomarker", BiomarkerType::RosLevel);
        assert_eq!(ds.name, "TestBiomarker");
        assert!(ds.values.is_empty());
        assert!(ds.source_pmid.is_none());
    }

    #[test]
    fn test_add_point_increments_values() {
        let mut ds = BiomarkerDataset::new("Test", BiomarkerType::RosLevel);
        ds.add_point(30.0, 0.2, 0.05, 100);
        ds.add_point(60.0, 0.4, 0.08, 150);
        assert_eq!(ds.values.len(), 2);
    }

    #[test]
    fn test_add_point_stores_correct_data() {
        let mut ds = BiomarkerDataset::new("Test", BiomarkerType::FrailtyIndex);
        ds.add_point(40.0, 0.3, 0.06, 200);
        let p = &ds.values[0];
        assert!((p.age - 40.0).abs() < 1e-9);
        assert!((p.value - 0.3).abs() < 1e-9);
        assert!((p.std_dev - 0.06).abs() < 1e-9);
        assert_eq!(p.n_samples, 200);
    }

    // ── max_age / min_age ─────────────────────────────────────────────────────

    #[test]
    fn test_max_age_single_point() {
        let mut ds = BiomarkerDataset::new("Test", BiomarkerType::RosLevel);
        ds.add_point(55.0, 0.3, 0.05, 100);
        assert!((ds.max_age() - 55.0).abs() < 1e-9);
    }

    #[test]
    fn test_min_age_single_point() {
        let mut ds = BiomarkerDataset::new("Test", BiomarkerType::RosLevel);
        ds.add_point(55.0, 0.3, 0.05, 100);
        assert!((ds.min_age() - 55.0).abs() < 1e-9);
    }

    #[test]
    fn test_max_age_multiple_points() {
        let mut ds = BiomarkerDataset::new("Test", BiomarkerType::RosLevel);
        ds.add_point(20.0, 0.1, 0.01, 50);
        ds.add_point(60.0, 0.3, 0.05, 100);
        ds.add_point(45.0, 0.2, 0.03, 80);
        assert!((ds.max_age() - 60.0).abs() < 1e-9);
    }

    #[test]
    fn test_min_age_multiple_points() {
        let mut ds = BiomarkerDataset::new("Test", BiomarkerType::RosLevel);
        ds.add_point(20.0, 0.1, 0.01, 50);
        ds.add_point(60.0, 0.3, 0.05, 100);
        ds.add_point(45.0, 0.2, 0.03, 80);
        assert!((ds.min_age() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn test_max_age_empty_is_neg_infinity() {
        let ds = BiomarkerDataset::new("Empty", BiomarkerType::RosLevel);
        assert!(ds.max_age().is_infinite() && ds.max_age() < 0.0);
    }

    #[test]
    fn test_min_age_empty_is_pos_infinity() {
        let ds = BiomarkerDataset::new("Empty", BiomarkerType::RosLevel);
        assert!(ds.min_age().is_infinite() && ds.min_age() > 0.0);
    }

    // ── synthetic_chip_frequency ──────────────────────────────────────────────

    #[test]
    fn test_synthetic_chip_has_pmid() {
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        assert_eq!(ds.source_pmid, Some(28636844), "CHIP should cite Jaiswal 2017 NEJM PMID 28636844");
    }

    #[test]
    fn test_synthetic_chip_ages_increasing() {
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        for w in ds.values.windows(2) {
            assert!(w[1].age > w[0].age, "Ages should be strictly increasing");
        }
    }

    #[test]
    fn test_synthetic_chip_values_increasing_with_age() {
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        for w in ds.values.windows(2) {
            assert!(w[1].value > w[0].value,
                "CHIP frequency should increase with age: {} -> {}", w[0].value, w[1].value);
        }
    }

    #[test]
    fn test_synthetic_chip_values_in_range() {
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        for p in &ds.values {
            assert!(p.value >= 0.0 && p.value <= 1.0,
                "CHIP value {} out of [0,1] at age {}", p.value, p.age);
        }
    }

    #[test]
    fn test_synthetic_chip_calibrated_below_02_at_70() {
        // B3 fix: at age 70 VAF should be ~0.07, not 0.20
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        let age_70 = ds.values.iter().find(|p| (p.age - 70.0).abs() < 1.0);
        if let Some(p) = age_70 {
            assert!(p.value < 0.15,
                "CHIP at age 70 should be recalibrated below 0.15 (B3 fix), got {}", p.value);
        }
    }

    #[test]
    fn test_synthetic_chip_min_age_40() {
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        assert!((ds.min_age() - 40.0).abs() < 1.0);
    }

    #[test]
    fn test_synthetic_chip_max_age_80() {
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        assert!((ds.max_age() - 80.0).abs() < 1.0);
    }

    // ── synthetic_ros ─────────────────────────────────────────────────────────

    #[test]
    fn test_synthetic_ros_has_four_points() {
        let ds = BiomarkerDataset::synthetic_ros();
        assert_eq!(ds.values.len(), 4);
    }

    #[test]
    fn test_synthetic_ros_values_increasing() {
        let ds = BiomarkerDataset::synthetic_ros();
        for w in ds.values.windows(2) {
            assert!(w[1].value >= w[0].value,
                "ROS should increase with age: {} -> {}", w[0].value, w[1].value);
        }
    }

    #[test]
    fn test_synthetic_ros_values_in_range() {
        let ds = BiomarkerDataset::synthetic_ros();
        for p in &ds.values {
            assert!(p.value >= 0.0 && p.value <= 1.0,
                "ROS value {} out of [0,1]", p.value);
        }
    }

    #[test]
    fn test_synthetic_ros_min_max_age() {
        let ds = BiomarkerDataset::synthetic_ros();
        assert!(ds.min_age() < ds.max_age());
        assert!((ds.min_age() - 20.0).abs() < 1.0);
        assert!((ds.max_age() - 80.0).abs() < 1.0);
    }

    #[test]
    fn test_synthetic_ros_has_no_pmid() {
        // Synthetic data — no PMID set
        let ds = BiomarkerDataset::synthetic_ros();
        assert!(ds.source_pmid.is_none());
    }

    // ── BiomarkerDataPoint ────────────────────────────────────────────────────

    #[test]
    fn test_data_point_std_dev_non_negative() {
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        for p in &ds.values {
            assert!(p.std_dev >= 0.0, "std_dev must be non-negative at age {}", p.age);
        }
    }

    #[test]
    fn test_data_point_n_samples_positive() {
        let ds = BiomarkerDataset::synthetic_chip_frequency();
        for p in &ds.values {
            assert!(p.n_samples > 0, "n_samples must be > 0");
        }
    }
}
