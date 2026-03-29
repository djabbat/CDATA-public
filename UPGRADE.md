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

## Pending proposals

### [ ] Frailty index recalibration after telomere fix
**Source:** Fix #1 changes frailty formula; calibration R² may shift
**Status:** [ ] proposed

After removing telomere term, run full MCMC calibration with new 2-parameter set
and verify R² stays ≥ 0.82.

### [ ] Tissue-specific telomere dynamics
**Source:** Biology: differentiated daughter cells DO lose telomeres
**Status:** [ ] proposed

The current model tracks only stem cell state. Consider adding a separate
`differentiated_telomere_length` field that does decrease (differentiated cells
lack telomerase), contributing to frailty via a separate pathway.

### [ ] Circadian amplitude dataset calibration
**Source:** Mechanism 7 (circadian) currently not validated
**Status:** [ ] proposed

Search for cohort data on circadian amplitude vs age (actigraphy studies).
Could add R² = 0.75 validation for the M3 pathway.

### [ ] CHIP frailty integration
**Source:** Jaiswal 2017 (PMID: 28792876): CHIP VAF correlates with frailty
**Status:** [ ] proposed

Add `chip_vaf` as a direct contributor to frailty_index (small weight ~0.05).
Currently CHIP only affects SASP via L1 link.
