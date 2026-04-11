# PARAMETERS.md — CDATA v3.4: 32 Model Parameters

**Source of truth:** `crates/cell_dt_core/src/fixed_params.rs` → `FixedParameters::default()`
**Total parameters:** 32 (reduced from 120 after 5 rounds of peer review)
**Calibration:** Metropolis-Hastings MCMC (pilot=1000, main=5000 iter; R-hat<1.05) on **5 biomarker trajectories × 7 age points** derived from published population studies (NHANES, Jaiswal 2017, Horvath 2013, Franceschi 2000). **2 free parameters** (τ_protection, Π₀); 30 fixed by biological justification. No individual patient records used.
**Validation R²:** 0.84 (independent cross-sectional validation: frailty + mortality + epigenetic clock) — **use this in all publications and grant materials**
**Calibration fit R²:** 0.89 (2 parameters fit to 35 literature-derived data points — not an independent validation metric; do NOT cite as validation)

---

## Group 1: Базовые (Base) — 3 parameters

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| α | `alpha` | 0.0082 | Normal(0.008, 0.002) | damage/division | Base centriolar damage per stem cell division | PMID: 36583780 |
| — | `hayflick_limit` | 50 | Fixed | divisions | Replicative senescence limit (Hayflick, 1961) | PMID: 13905658 |
| — | `base_ros_young` | 0.12 | Fixed | a.u. | Baseline ROS level in young (20-yr) cells | PMID: 16565722 |

---

## Group 2: Защита молодости (Youth Protection) — 3 parameters

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| Π₀ | `pi_0` | 0.87 | Beta(8, 2) | fraction | Initial protection factor at birth (centrosomal quality control) | PMID: 29363672 |
| τ_prot | `tau_protection` | 24.3 | Gamma(25, 2) | years | Half-life of youth protection decay | PMID: 15967997 |
| Π_base | `pi_baseline` | 0.10 | Fixed | fraction | Residual protection at old age (basal repair activity) | PMID: 23746838 |

**Constraint:** Π₀ + Π_baseline must be ≤ 1.0 (validated in `FixedParameters::validate()`)

**Age-dependent protection:**
```
Π(t) = Π_baseline + (Π₀ - Π_baseline) × exp(-t / τ_protection)
```

---

## Group 3: Асимметрия деления (Asymmetric Division) — 3 parameters

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| P₀ | `p0_inheritance` | 0.94 | Beta(18, 1) | probability | P(daughter inherits undamaged centriole) at age 0 | PMID: 17255513 |
| — | `beta_a_fidelity` | 0.15 | Fixed | 1/D_unit | β_A: exponential decay rate of P_A(D) = p0·exp(−β_A·D) | PMID: 29363672 |
| — | `fidelity_loss` | 0.10 | Fixed | fraction | Total fidelity loss at maximum age | PMID: 17255513 |

**Range validation:** P(inherit_maternal) ∈ [0.60, 0.98] across all ages

---

## Group 4: Тканевые параметры (Tissue-Specific) — 12 parameters

### 4a. Hematopoietic Stem Cells (HSC)

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| ν_HSC | `hsc_nu` | 12 | Fixed | div/year | Division rate (most active stem cell pool) | PMID: 19062086 |
| β_HSC | `hsc_beta` | 1.0 | Fixed | multiplier | Damage accumulation multiplier (reference = 1.0) | PMID: 36583780 |
| τ_HSC | `hsc_tau` | 0.3 | Beta(3, 7) | fraction | Damage tolerance (low — HSC very sensitive to damage) | PMID: 16565722 |

### 4b. Intestinal Stem Cells (ISC)

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| ν_ISC | `isc_nu` | 70 | Fixed | div/year | Highest division rate — crypt turnover ~5 days | PMID: 17934449 |
| β_ISC | `isc_beta` | 0.3 | Fixed | multiplier | Lower damage/division (strong repair mechanisms) | PMID: 17934449 |
| τ_ISC | `isc_tau` | 0.8 | Beta(8, 2) | fraction | High tolerance (redundant crypt cell pool) | PMID: 17934449 |

### 4c. Muscle Stem Cells (Satellite cells)

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| ν_Muscle | `muscle_nu` | 4 | Fixed | div/year | Low basal division rate | PMID: 24522534 |
| β_Muscle | `muscle_beta` | 1.2 | Fixed | multiplier | Higher damage per division than HSC | PMID: 24522534 |
| τ_Muscle | `muscle_tau` | 0.5 | Beta(5, 5) | fraction | Moderate tolerance | PMID: 24522534 |

### 4d. Neural Stem Cells (NSC)

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| ν_Neural | `neural_nu` | 2 | Fixed | div/year | Near-quiescent pool | PMID: 21295697 |
| β_Neural | `neural_beta` | 1.5 | Fixed | multiplier | Highest damage/division — minimal repair | PMID: 21295697 |
| τ_Neural | `neural_tau` | 0.2 | Beta(2, 8) | fraction | Lowest tolerance (post-mitotic context) | PMID: 21295697 |

**Ordering (validated by test):** HSC_τ < Neural_τ < Muscle_τ < ISC_τ

---

## Group 5: SASP (Hormetic Response) — 4 parameters

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| θ_stim | `stim_threshold` | 0.3 | Uniform(0.2, 0.4) | fraction | SASP level below which effect is stimulatory (pro-regenerative) | PMID: 19053174 |
| θ_inhib | `inhib_threshold` | 0.8 | Uniform(0.6, 1.0) | fraction | SASP level above which effect becomes fully inhibitory | PMID: 19053174 |
| — | `max_stimulation` | 1.5 | Fixed | multiplier | Maximum pro-regenerative boost (low SASP → +50% repair) | PMID: 19053174 |
| — | `max_inhibition` | 0.3 | Fixed | multiplier | Minimum inhibitory factor (high SASP → 70% suppression of repair) | PMID: 19053174 |

**Non-monotonic response:**
```
SASP effect:
  [0, θ_stim]         → stimulatory (max_stimulation)
  [θ_stim, θ_inhib]   → linear transition stimulation → inhibition
  [θ_inhib, 1]        → inhibitory (max_inhibition)
```
**Continuity constraint:** Response must be continuous at both threshold points (tested by `test_sasp_continuity`).

---

## Group 6: CHIP Drift — 4 parameters

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| s_DNMT3A | `dnmt3a_fitness` | 0.15 | Normal(0.15, 0.05) | selective advantage/yr | Fitness advantage per year for DNMT3A-mutant clones | PMID: 25426838 |
| — | `dnmt3a_age_slope` | 0.002 | Fixed | /year² | Rate at which DNMT3A clone advantage increases with age | PMID: 25426838 |
| s_TET2 | `tet2_fitness` | 0.12 | Normal(0.12, 0.04) | selective advantage/yr | Fitness advantage per year for TET2-mutant clones | PMID: 28792876 |
| — | `tet2_age_slope` | 0.0015 | Fixed | /year² | Rate at which TET2 clone advantage increases with age | PMID: 28792876 |

**Calibration target:** VAF at age 70 ≈ 0.07 (Jaiswal 2017, PMID: 28792876)

---

## Group 7: Фиксированные (Fixed System) — 4 parameters

| Symbol | Field | Value | Prior | Unit | Biological Meaning | Source |
|--------|-------|-------|-------|------|--------------------|--------|
| — | `mtor_activity` | 0.7 | Fixed | a.u. | Baseline mTOR activity (nutrient sensing, anabolic) | PMID: 19587680 |
| — | `circadian_amplitude` | 0.2 | Fixed | fraction | Amplitude of circadian modulation on repair efficiency | PMID: 22056141 |
| — | `meiotic_reset` | 0.8 | Fixed | fraction | Degree of centriole damage reset in germline (Bowness et al.) | PMID: 29363672 |
| — | `yap_taz_sensitivity` | 0.5 | Fixed | a.u. | YAP/TAZ mechanosensing sensitivity (mechanotransduction) | PMID: 21654799 |

---

## Group 8: Гипоксийный модуль (Hypoxia / O₂-Shield) — module-level constants, v3.4

> **Не входят в 32-параметрную структуру FixedParameters** — хранятся как `const` в
> `crates/cell_dt_modules/mitochondrial/src/system.rs`.
> Добавлены 2026-04-05 по результатам Peters-Hall et al. (2020).

| Symbol | Const | Value | Unit | Biological Meaning | Source |
|--------|-------|-------|------|--------------------|--------|
| s_max | `MITO_SHIELD_MAX` | 0.99 | fraction | Максимальный щит при [O₂]→0; рекалибровано против >200 PD Peters-Hall | DOI: 10.1096/fj.201901415R |
| K_O₂ | `K_O2` | 0.2 | (%O₂)⁻¹ | Скорость экспоненциального спада щита с ростом [O₂] | Ito et al. 2006, PMID: 16474382 |
| D_crit | `D_CRIT` | 1000 | a.u. | Порог повреждения → сенесценция (для N_Hayflick) | Калибровка: N≈50 при 21% O₂ |
| α·ν·β | `ALPHA_NU_BETA` | 20 | a.u./div | Составной параметр скорости повреждения | Hayflick & Moorhead 1961 |

**Клеточные модификаторы φ_cell (enum CellTypeShield):**

| Вариант | φ | Обоснование |
|---------|---|------------|
| `EpithelialProgenitor` | 1.00 | Базальные бронхиальные прогениторы (Peters-Hall 2020) |
| `HematopoieticStem` | 0.96 | HSC с нишевой антиоксидантной защитой (Ito 2006) |
| `Fibroblast` | 0.91 | Дифференцированные, меньший PCM (Hayflick 1961) |

**Функции модуля:**
```rust
pub fn mito_shield_for_o2(o2_percent: f64, cell_type: CellTypeShield) -> f64
pub fn predicted_hayflick(o2_percent: f64, cell_type: CellTypeShield) -> f64
```

**Верификационные предсказания:**
| Условия | Предсказание | Наблюдение | Статус |
|---------|------------|------------|--------|
| Fibroblast, 21% O₂ | N ≈ 50 | 50 ± 8 (Hayflick 1961) | ✅ |
| Progenitor, 2% O₂ | N ≈ 148 | >200 (Peters-Hall 2020) | ⚠️ Гипотезы §5.1 |
| Progenitor, 2% O₂ + ROCKi | N ≈ 526 | >200 | ✅ |

---

## Summary Table

| Group | Count | Estimated | Fixed | Notes |
|-------|-------|-----------|-------|-------|
| Base | 3 | 1 (α) | 2 | α is the primary calibrated parameter |
| Youth protection | 3 | 2 (Π₀, τ) | 1 | |
| Asymmetry | 3 | 1 (P₀) | 2 | |
| Tissue × 4 | 12 | 4 (τ_HSC, τ_ISC, τ_Muscle, τ_Neural) | 8 | ν and β fixed from literature |
| SASP | 4 | 2 (θ_stim, θ_inhib) | 2 | |
| CHIP | 4 | 2 (s_DNMT3A, s_TET2) | 2 | |
| Fixed system | 4 | 0 | 4 | All fixed from literature |
| **Total (FixedParameters)** | **32** | **12** | **20** | **Matches FixedParameters struct (32 fields)** |
| Hypoxia module (Group 8) | 4+3φ | 0 | All | module-level const, NOT in FixedParameters |

---

## Core Equation Reference

```
d(Damage)/dt = α × ν(t) × (1 − Π(t)) × S(t) × A(t) × (1 − mito_shield_total(t, [O₂]))

where:
  α      = 0.0082             base centriolar damage per division
  ν(t)   = tissue-specific    division rate (div/year)
  Π(t)   = Π_baseline + (Π₀ − Π_baseline) × exp(−t / τ_protection)
  S(t)   = SASP hormetic modifier (non-monotonic, see Group 5)
  A(t)   = P(correct inheritance) (declines with age per Group 3)
  mito_shield_total(t,[O₂]) = exp(−k_age×t) × MITO_SHIELD_MAX × φ_cell × exp(−K_O2×[O₂])
```

**Derived: Hayflick limit as function of O₂ (Group 8):**
```
N_Hayflick([O₂]) = D_crit / (ALPHA_NU_BETA × (1 − mito_shield_for_o2([O₂], cell_type)))

  D_crit        = 1000 a.u.
  ALPHA_NU_BETA = 20 a.u./div
```

---

## Calibration Protocol

```bash
# Run MCMC calibration (NUTS sampler)
cargo run --example basic_simulation --release

# Calibration targets (from literature):
#   Frailty Index at 80 yrs: ~0.3 (Rockwood 2005)
#   CHIP prevalence at 70 yrs: ~7% (Jaiswal 2017)
#   Epigenetic age acceleration: ~3 yrs/decade post-40 (Horvath 2013)
#   NK cell efficiency at 70 yrs: ~50% of young (PMID: 12803352)
```
