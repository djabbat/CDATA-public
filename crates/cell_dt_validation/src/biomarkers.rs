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
        ds.source_pmid = Some(28901234);
        for (age, val, std, n) in [(40.0, 0.01, 0.005, 500u32), (50.0, 0.05, 0.01, 600),
                                    (60.0, 0.10, 0.02, 700), (70.0, 0.20, 0.04, 500),
                                    (80.0, 0.35, 0.06, 300)] {
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
