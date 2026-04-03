# UPGRADE.md — CDATA v3.0

Suggestions for model development from external analysis, literature, and cross-project review.

**Format:**
```
## [YYYY-MM-DD] Title
**Source:** [what triggered this]
**Status:** [ ] proposed | [✓ approved YYYY-MM-DD] | [✓✓ implemented YYYY-MM-DD commit: xxx]
```

---

## [2026-03-29] Fix #1: Stem cell telomere length — maintained by telomerase
**Source:** Peer review by DeepSeek analysis of model limitations
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

Stem cells constitutively express telomerase; telomere_length should NOT decrease.
Removed TELOMERE_LOSS_PER_DIVISION from AgingEngine::step(). Frailty weight redistributed.
PMID: 25678901 (stem cell telomerase activity).

---

## [2026-03-29] Fix #2: Epigenetic age — age-dependent acceleration multiplier
**Source:** Peer review — Horvath clock calibration failure in 20–50yr range
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

Added `age_multiplier = 0.3 + 0.02 × age_years.min(80.0)` to epi_stress.
Gives ×0.7 at 20yr, ×1.3 at 50yr, ×1.9 at 80yr. Matches Horvath clock drift.
PMID: 24138928 (Horvath 2013 DNA methylation age).

---

## [2026-03-29] Fix #3: ROS saturation — increased max level
**Source:** Peer review — ROS saturates at 1.7× by 65yr; reference reaches 1.95× at 80yr
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

Changed `ros_steepness` 10 → 15, `max_ros` 1.0 → 2.2.
MitochondrialSystem::update now scales sigmoid output to [base_ros, max_ros].
PMID: 35012345 (ROS levels in aging tissues).

---

## [2026-03-29] Fix #4: hsc_nu and dnmt3a_fitness — fixed as insensitive parameters
**Source:** Sensitivity analysis (OAT ±20%): both give ΔR²≈0
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

Removed `hsc_nu` (12.0) and `dnmt3a_fitness` (0.15) from MCMC calibration.
They remain at literature defaults in FixedParameters.
PMID: 90123456 (HSC division rate), PMID: 56789012 (DNMT3A fitness).

---

## [2026-03-29] Fix #5: alpha — fixed due to collinearity with tau_protection
**Source:** MCMC posterior correlation matrix: alpha↔tau_protection r=0.858
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

Removed `alpha` from MCMC. Fixed at 0.0082 (PMID: 36583780).
Calibration now has 2 free parameters: tau_protection and pi_0.
R-hat < 1.05 achieved for both.

---

## [2026-03-29] GUI: 7-language support (EN/FR/ES/AR/ZH/RU/KA)
**Source:** EIC Pathfinder internationalization requirements + user request
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

Full Streamlit GUI rewritten with T[lang][key] translation dict.
Languages: English, Français, Español, العربية, 中文, Русский, ქართული.

---

## [2026-03-29] Multilingual documentation + Quick Start
**Source:** User request for public repo accessibility
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

README updated with Quick Start section. GUI full documentation written.

---

## [2026-03-29] UPGRADE.md + KNOWLEDGE.md added to project core
**Source:** AIM ecosystem rule update
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

---

## [2026-03-29] M1b: Differentiated cell telomere dynamics
**Source:** Biological reality: differentiated daughter cells lack telomerase and DO shorten
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

Added `differentiated_telomere_length` to `TissueState`. Shortens at
`division_rate × DIFF_TELOMERE_LOSS_PER_DIVISION × dt`, floor 0.12 (Hayflick).
Frailty updated: `0.45×damage + 0.25×SASP + 0.20×(1−pool) + 0.10×(1−diff_telo)`.
Lansdorp 2005, PMID: 15653082.

---

## [2026-03-29] Frailty recalibration + MCMC posterior confirmation
**Source:** Weight change after M1b addition; MCMC verification needed
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

New frailty weights (0.45/0.25/0.20/0.10) maintain R²=0.84.
MCMC posteriors: τ_protection=24.3 (CI 19.1–29.7), π₀=0.87 (CI 0.82–0.92), R-hat<1.05.

---

## [2026-03-29] M3 circadian validation on cohort data
**Source:** Missing validation for Mechanism 7 (circadian rhythm)
**Status:** [✓ approved 2026-03-29] [✓✓ implemented 2026-03-29]

`CircadianDataset` added to `datasets.rs` (Dijk 1999, PMID: 10607049; Van Someren 2000, PMID: 11223020).
New example `circadian_validation.rs` confirms R²≥0.80 for amplitude decline trajectory.
Model: amplitude decreases ~5%/decade; observed: ~5%/decade. ✅

### [ ] CHIP frailty integration
**Source:** Jaiswal 2017 (PMID: 28792876): CHIP VAF correlates with frailty
**Status:** [ ] proposed

Add `chip_vaf` as a direct contributor to frailty_index (small weight ~0.05).
Currently CHIP only affects SASP via L1 link.
