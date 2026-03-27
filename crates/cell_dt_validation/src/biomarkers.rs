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
        // FIX Round 7 (B3): Recalibrated VAF to match Jaiswal et al. 2017 (PMID: 28792876)
        // NEJM 2017: VAF>0.02 in ~2% at age 40, ~10% at 65, rare >0.10 at age 70
        // Previous values were 2–4× too high (70yo: 0.20 → corrected 0.07)
        ds.source_pmid = Some(28792876);  // Jaiswal 2017 NEJM
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
}
