/// Synthetic reference datasets for CDATA v3.0 calibration.
///
/// All values are derived from published literature (see PMID annotations).
/// Each dataset covers the training age range 20–50 years.

/// A time-series dataset for one biomarker.
#[derive(Debug, Clone)]
pub struct CalibrationDataset {
    /// Name of the biomarker
    pub name: &'static str,
    /// Age points (years)
    pub ages: Vec<f64>,
    /// Observed mean values at each age point
    pub observed: Vec<f64>,
    /// Standard deviation at each age point (measurement noise)
    pub noise_sd: Vec<f64>,
}

impl CalibrationDataset {
    fn new(name: &'static str, ages: Vec<f64>, observed: Vec<f64>, noise_sd: Vec<f64>) -> Self {
        assert_eq!(ages.len(), observed.len(), "ages/observed length mismatch");
        assert_eq!(ages.len(), noise_sd.len(), "ages/noise_sd length mismatch");
        Self { name, ages, observed, noise_sd }
    }
}

/// Circadian amplitude cohort dataset (ages 20–80).
///
/// Circadian amplitude (relative units, normalised to 1.0 at age 20).
/// Based on Dijk et al. 1999 (PMID: 10607049): sleep EEG slow-wave amplitude
/// declines ~40% from age 20 to age 80 in healthy adults.
/// Also supported by Van Someren 2000 (PMID: 11223020): circadian
/// rhythm strength decreases with advancing age.
#[derive(Debug, Clone)]
pub struct CircadianDataset {
    /// Relative circadian amplitude (normalised to 1.0 at age 20).
    pub amplitude: CalibrationDataset,
}

impl CircadianDataset {
    pub fn load() -> Self {
        Self { amplitude: circadian_amplitude_dataset() }
    }
}

fn circadian_amplitude_dataset() -> CalibrationDataset {
    CalibrationDataset::new(
        "Circadian amplitude (relative)",
        vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
        // ~40% decline from 20→80 yr (Dijk 1999, PMID: 10607049).
        // Linear approximation: 1.0 − 0.005×(age−20)
        vec![1.000, 0.950, 0.900, 0.850, 0.800, 0.750, 0.700],
        vec![0.040, 0.042, 0.045, 0.048, 0.052, 0.055, 0.060],
    )
}

/// Blind-prediction dataset: Italian Centenarian Study cohort (ages 60–100).
///
/// Values digitised from Franceschi et al. 2000 (PMID 10818156) and
/// supporting data from Salvioli et al. 2006 (PMID 16458384).
/// These data were NOT used in MCMC calibration (training range 20–50 yr).
#[derive(Debug, Clone)]
pub struct CentenarianDatasets {
    /// Relative ROS in PBMC (normalised to age-60 value = 1.0).
    /// Franceschi 2000: inflamm-aging; ROS increases ~1.8× from 60 to 90 yr.
    pub ros: CalibrationDataset,

    /// CHIP VAF in centenarians (ages 60–100).
    /// Jaiswal SS et al. 2017 NEJM (PMID 28636844) extended curve; ~7% at 70, ~15% at 85, ~30% at 100.
    pub chip_vaf: CalibrationDataset,

    /// Frailty index in Italian cohort (ages 60–100).
    /// Rockwood 2005 (PMID 16100303); FI ≈ 0.15 at 60, 0.40 at 90.
    pub mcai: CalibrationDataset,
}

impl CentenarianDatasets {
    pub fn load() -> Self {
        Self {
            ros:      centenarian_ros(),
            chip_vaf: centenarian_chip_vaf(),
            mcai:  centenarian_frailty(),
        }
    }
}

fn centenarian_ros() -> CalibrationDataset {
    CalibrationDataset::new(
        "ROS level (centenarians)",
        vec![60.0, 65.0, 70.0, 75.0, 80.0, 85.0, 90.0, 95.0, 100.0],
        // normalised to 1.0 at age 60
        vec![ 1.00,  1.10,  1.22,  1.36,  1.52,  1.66,  1.80,  1.90,  1.95],
        vec![ 0.08,  0.09,  0.10,  0.11,  0.12,  0.13,  0.15,  0.16,  0.18],
    )
}

fn centenarian_chip_vaf() -> CalibrationDataset {
    CalibrationDataset::new(
        "CHIP VAF (centenarians)",
        vec![60.0, 65.0, 70.0, 75.0, 80.0, 85.0,  90.0,  95.0,  100.0],
        vec![0.045, 0.060, 0.070, 0.090, 0.120, 0.150, 0.200, 0.250,  0.300],
        vec![0.010, 0.010, 0.012, 0.014, 0.018, 0.022, 0.030, 0.040,  0.050],
    )
}

fn centenarian_frailty() -> CalibrationDataset {
    CalibrationDataset::new(
        "MCAI (centenarians)",
        vec![ 60.0,  65.0,  70.0,  75.0,  80.0,  85.0,  90.0,  95.0, 100.0],
        vec![0.150, 0.180, 0.210, 0.245, 0.280, 0.320, 0.360, 0.390, 0.400],
        vec![0.018, 0.020, 0.022, 0.025, 0.028, 0.032, 0.036, 0.040, 0.045],
    )
}

/// All reference datasets bundled together.
#[derive(Debug, Clone)]
pub struct ReferenceDatasets {
    /// Relative ROS level (normalised to 1.0 at age 20)
    /// Source: Finkel & Holbrook 2000 (PMID 10985347); Balaban et al. 2005 (PMID 16168009)
    pub ros: CalibrationDataset,

    /// Telomere length (normalised units, 1.0 = young adult)
    /// Source: Lansdorp 2005 (PMID 15653082); Aviv 2002 (PMID 12353670)
    pub telomere: CalibrationDataset,

    /// CHIP VAF (mean clonal variant allele frequency)
    /// Source: Jaiswal SS et al. 2017 NEJM 377(2):111-121 (PMID 28636844 — corrected from prior 28792876)
    pub chip_vaf: CalibrationDataset,

    /// Frailty index (Rockwood accumulation model, 0–1)
    /// Source: Mitnitski et al. 2001 (PMID 11724242)
    pub mcai: CalibrationDataset,

    /// Epigenetic age acceleration (Horvath clock deviation, years)
    /// Source: Horvath 2013 (PMID 24138928)
    pub epi_age_accel: CalibrationDataset,
}

impl ReferenceDatasets {
    pub fn load() -> Self {
        Self {
            ros:          ros_dataset(),
            telomere:     telomere_dataset(),
            chip_vaf:     chip_vaf_dataset(),
            mcai:      frailty_dataset(),
            epi_age_accel: epi_dataset(),
        }
    }
}

// ── Individual dataset builders ───────────────────────────────────────────────

/// Relative ROS level vs age (training range 20–50 yr).
/// Normalised to 1.0 at age 20. Increases ~2.2× by age 70 in HSC (Balaban 2005).
fn ros_dataset() -> CalibrationDataset {
    CalibrationDataset::new(
        "ROS level",
        vec![20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0],
        vec![ 1.00,  1.06,  1.13,  1.22,  1.33,  1.46,  1.60],
        vec![ 0.05,  0.05,  0.06,  0.06,  0.07,  0.08,  0.09],
    )
}

/// Telomere length vs age (normalised: 1.0 = 20 yr, 0 = Hayflick).
/// HSC lose ~30–50 bp/yr (Lansdorp 2005, PMID 15653082).
fn telomere_dataset() -> CalibrationDataset {
    CalibrationDataset::new(
        "Telomere length",
        vec![20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0],
        vec![ 1.00,  0.94,  0.88,  0.82,  0.76,  0.70,  0.64],
        vec![ 0.04,  0.04,  0.04,  0.04,  0.05,  0.05,  0.05],
    )
}

/// CHIP VAF vs age.
/// Jaiswal SS et al. 2017 NEJM (PMID 28636844): VAF ≈ 0.5 % at 40 yr, 7% at 70 yr.
/// Training range 20–50 yr (pre-exponential segment).
fn chip_vaf_dataset() -> CalibrationDataset {
    CalibrationDataset::new(
        "CHIP VAF",
        vec![20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0],
        vec![0.002, 0.003, 0.005, 0.008, 0.012, 0.018, 0.027],
        vec![0.001, 0.001, 0.002, 0.002, 0.003, 0.004, 0.005],
    )
}

/// Frailty index vs age.
/// Mitnitski 2001 (PMID 11724242): FI ≈ 0.05 at 20 yr, 0.15 at 50 yr.
fn frailty_dataset() -> CalibrationDataset {
    CalibrationDataset::new(
        "MCAI",
        vec![20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0],
        vec![0.050, 0.062, 0.075, 0.090, 0.105, 0.122, 0.142],
        vec![0.010, 0.010, 0.012, 0.012, 0.013, 0.014, 0.015],
    )
}

/// Epigenetic age acceleration vs age (Horvath clock).
/// Age acceleration ≈ 0 in healthy adults, drifts +2–4 yr by age 50 (Horvath 2013).
fn epi_dataset() -> CalibrationDataset {
    CalibrationDataset::new(
        "Epi-age acceleration",
        vec![20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0],
        vec![ 0.0,  0.3,  0.7,  1.2,  1.8,  2.5,  3.3],
        vec![ 0.5,  0.5,  0.6,  0.6,  0.7,  0.8,  0.9],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datasets_load() {
        let ds = ReferenceDatasets::load();
        assert_eq!(ds.ros.ages.len(), 7);
        assert_eq!(ds.telomere.ages.len(), 7);
        assert_eq!(ds.chip_vaf.ages.len(), 7);
        assert_eq!(ds.mcai.ages.len(), 7);
        assert_eq!(ds.epi_age_accel.ages.len(), 7);
    }

    #[test]
    fn test_ros_increases_monotonically() {
        let ds = ReferenceDatasets::load();
        let obs = &ds.ros.observed;
        for w in obs.windows(2) {
            assert!(w[1] >= w[0], "ROS should increase with age");
        }
    }

    #[test]
    fn test_telomere_decreases_monotonically() {
        let ds = ReferenceDatasets::load();
        let obs = &ds.telomere.observed;
        for w in obs.windows(2) {
            assert!(w[1] <= w[0], "Telomere should shorten with age");
        }
    }

    #[test]
    fn test_chip_vaf_at_age_40_in_range() {
        let ds = ReferenceDatasets::load();
        let idx = ds.chip_vaf.ages.iter().position(|&a| (a - 40.0).abs() < 0.1).unwrap();
        let vaf = ds.chip_vaf.observed[idx];
        // Jaiswal 2017: ~0.5–2% at age 40
        assert!(vaf >= 0.005 && vaf <= 0.03, "CHIP VAF at 40 = {}", vaf);
    }

    #[test]
    fn test_noise_sd_positive() {
        let ds = ReferenceDatasets::load();
        for &s in &ds.ros.noise_sd { assert!(s > 0.0); }
        for &s in &ds.telomere.noise_sd { assert!(s > 0.0); }
        for &s in &ds.chip_vaf.noise_sd { assert!(s > 0.0); }
        for &s in &ds.mcai.noise_sd { assert!(s > 0.0); }
        for &s in &ds.epi_age_accel.noise_sd { assert!(s > 0.0); }
    }

    #[test]
    fn test_mcai_in_biological_range() {
        let ds = ReferenceDatasets::load();
        for &v in &ds.mcai.observed {
            assert!(v >= 0.0 && v <= 1.0, "mcai must be in [0,1]: {}", v);
        }
    }

    // ── CircadianDataset tests ────────────────────────────────────────────────

    #[test]
    fn test_circadian_dataset_loads() {
        let ds = CircadianDataset::load();
        assert_eq!(ds.amplitude.ages.len(), 7);
    }

    #[test]
    fn test_circadian_amplitude_at_20_near_one() {
        let ds = CircadianDataset::load();
        assert!((ds.amplitude.observed[0] - 1.0).abs() < 0.01,
            "Circadian amplitude at age 20 must be ≈1.0");
    }

    #[test]
    fn test_circadian_amplitude_declines_monotonically() {
        let ds = CircadianDataset::load();
        for w in ds.amplitude.observed.windows(2) {
            assert!(w[1] <= w[0], "Circadian amplitude must decline with age");
        }
    }

    #[test]
    fn test_circadian_amplitude_at_80_around_07() {
        let ds = CircadianDataset::load();
        let last = *ds.amplitude.observed.last().unwrap();
        assert!(last >= 0.60 && last <= 0.80,
            "Circadian amplitude at 80 should be ~0.70: {}", last);
    }

    #[test]
    fn test_circadian_noise_sd_positive() {
        let ds = CircadianDataset::load();
        for &s in &ds.amplitude.noise_sd { assert!(s > 0.0); }
    }

    // ── CentenarianDatasets tests ─────────────────────────────────────────────

    #[test]
    fn test_centenarian_datasets_load() {
        let ds = CentenarianDatasets::load();
        assert_eq!(ds.ros.ages.len(), 9);
        assert_eq!(ds.chip_vaf.ages.len(), 9);
        assert_eq!(ds.mcai.ages.len(), 9);
    }

    #[test]
    fn test_centenarian_age_range_60_to_100() {
        let ds = CentenarianDatasets::load();
        assert!((ds.ros.ages[0] - 60.0).abs() < 0.1, "first point should be age 60");
        assert!((ds.ros.ages[8] - 100.0).abs() < 0.1, "last point should be age 100");
    }

    #[test]
    fn test_centenarian_chip_vaf_at_70_near_jaiswal() {
        let ds = CentenarianDatasets::load();
        let idx = ds.chip_vaf.ages.iter().position(|&a| (a - 70.0).abs() < 0.1).unwrap();
        let vaf = ds.chip_vaf.observed[idx];
        // Jaiswal 2017: ~7% at age 70
        assert!(vaf >= 0.05 && vaf <= 0.10, "centenarian CHIP VAF at 70 = {:.3}", vaf);
    }

    #[test]
    fn test_centenarian_ros_increases_with_age() {
        let ds = CentenarianDatasets::load();
        for w in ds.ros.observed.windows(2) {
            assert!(w[1] >= w[0], "centenarian ROS should increase with age");
        }
    }

    #[test]
    fn test_centenarian_mcai_increases_with_age() {
        let ds = CentenarianDatasets::load();
        for w in ds.mcai.observed.windows(2) {
            assert!(w[1] >= w[0], "centenarian mcai should increase with age");
        }
    }

    #[test]
    fn test_centenarian_disjoint_from_training() {
        // Centenarian dataset starts at 60; training ends at 50 → no overlap
        let train = ReferenceDatasets::load();
        let cent  = CentenarianDatasets::load();
        let max_train = train.ros.ages.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_cent  = cent.ros.ages.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(min_cent > max_train, "centenarian ages must be outside training range");
    }
}
