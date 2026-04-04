# CDATA Cell-DT v3.2.3

## Centriolar Damage Accumulation Theory of Aging — Digital Twin Simulator

[![Rust](https://img.shields.io/badge/rust-1.77%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-483-brightgreen.svg)](https://github.com/djabbat/CDATA-public)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19174506.svg)](https://doi.org/10.5281/zenodo.19174506)

**CDATA (Centriolar Damage Accumulation Theory of Aging)** — a theory of aging that explains organismal degradation as an inevitable consequence of the stem cell differentiation program. The maternal centriole of stem cells is the **only biological structure that irreversibly accumulates molecular damage throughout the lifespan** because it replicates via a template mechanism and is always inherited by the daughter cell that maintains stemness.

---

## ⚡ Quick Start

**Prerequisites:** [Rust 1.77+](https://rustup.rs) · Python 3.9+ · Git

```bash
# 1. Clone
git clone https://github.com/djabbat/CDATA-public.git
cd CDATA-public

# 2. Launch interactive GUI (7 languages: EN/FR/ES/AR/ZH/RU/KA)
bash run.sh gui
# → opens http://localhost:8501 in your browser

# 3. Run simulation (all 4 tissues, terminal output)
bash run.sh sim

# 4. Run tests (472 tests)
bash run.sh test

# 5. Build release binary
bash run.sh build
```

**All commands:**

| Command | Description |
|---------|-------------|
| `bash run.sh gui` | Streamlit GUI (browser, 7 languages) |
| `bash run.sh sim` | Basic aging simulation (4 tissues) |
| `bash run.sh test` | Full test suite (472 tests) |
| `bash run.sh build` | Release build |
| `bash run.sh check` | Fast syntax check |
| `bash run.sh bench` | Benchmarks |
| `bash run.sh docs` | Rustdoc documentation |
| `bash run.sh python` | Build PyO3 Python bindings |

---

## 📖 Table of Contents

- [Theory in Brief](#-theory-in-brief)
- [Key Mechanisms](#-key-mechanisms)
- [Key Results](#-key-results)
- [GUI Usage](#-gui-usage)
- [Architecture](#-architecture)
- [Interventions](#-interventions)
- [Validation](#-validation)
- [Publications](#-publications)
- [Citation](#-citation)

---

## 🧬 Theory in Brief

### Central Concepts

The theory focuses on the maternal centriole of stem cells as a structure that accumulates molecular changes over time. Key observations:

1. **Centriole replication is predominantly template-dependent** — in somatic stem cells, the existing centriole serves as a template for new centriole formation.

2. **Early embryonic development offers a reset** — during the first 2–3 divisions of the zygote, centrioles are formed *de novo*, establishing a fresh pool. After this window, replication transitions to template-dependent mechanisms.

3. **Asymmetric inheritance** — the daughter cell maintaining stemness tends to inherit the older (maternal) centriole; the differentiating daughter receives the newly formed one.

4. **Cumulative damage** — molecular modifications (oxidative, structural, PTM) accumulate in retained centrioles over successive divisions.

### The Paradox of Aging

Tissues are continuously renewed by stem cells, yet the organism still ages — precisely because stem cells carry an increasing burden of damage in their maternal centrioles.

### Mathematical Formulation

```
d(Damage)/dt = α × ν(t) × (1 − Π(t)) × S(t) × A(t)
```

| Symbol | Description | Value/Range |
|--------|-------------|-------------|
| α | Baseline damage per division | 0.0082 |
| ν(t) | Stem cell division rate (tissue-specific) | 2–70 /year |
| Π(t) | Youth protection (exponential decay) | 0.87 → 0.10 |
| S(t) | SASP hormetic modifier (non-monotonic) | 0.3–1.5× |
| A(t) | Asymmetric division fidelity | 0.60–0.98 |

---

## ⚙️ Key Mechanisms

| # | Mechanism | Description | R² |
|---|-----------|-------------|-----|
| 1 | **Youth Protection** | TERT, FOXO, SIRT, NRF2 protect centrioles in early life; exponential decay (τ = 24.3 yr) | 0.91 |
| 2 | **Stochastic Asymmetric Inheritance** | P(maternal centriole → stem daughter) declines 0.94 → 0.60 with age | 0.79 |
| 3 | **Hormetic SASP Response** | Low SASP: +50% regeneration; high SASP: −70% repair. Non-monotonic Arndt–Schulz effect | 0.82 |
| 4 | **Tissue-Specific Tolerance** | HSC low tolerance (τ=0.3); ISC high tolerance (τ=0.8) despite 6× higher division rate | 0.84 |
| 5 | **Germline Reset** | D-complex: 3.5× higher repair; 80% damage reset at meiosis; explains paternal age effect | 0.76 |
| 6 | **Mechanotransduction** | Physical activity → YAP/TAZ → mitochondrial biogenesis → +3.2 years lifespan | — |
| 7 | **Circadian Rhythms** | Circadian amplitude modulates repair efficiency (±20%); shift work → −2.1 years | — |
| 8 | **CHIP Drift** | DNMT3A/TET2 clonal expansion; VAF predicted 0.07 at age 70 (Jaiswal 2017) | 0.79 |

---

## 📊 Key Results

| Metric | Value |
|--------|-------|
| **Parameters** | 32 (reduced from 120 after peer review) |
| **Mechanisms** | 8 validated |
| **Tissues** | 4 (HSC, ISC, Muscle, Neural) |
| **Tests** | 483 unit tests |
| **Frailty Index** | 5-component: centriole×0.40 + SASP×0.25 + stem pool×0.20 + telomere×0.10 + CHIP×0.05 |
| **Validation R²** | 0.84 independent (frailty, mortality, CHIP, epigenetic clock) |
| **Blind Prediction Δ** | 1.6 years (Italian Centenarians, n=500) |
| **CHIP Prediction R²** | 0.79 |
| **Epigenetic Clock R²** | 0.91 |

### Tissue-Specific Parameters

| Tissue | ν (div/yr) | β (damage/div) | τ (tolerance) | Effective aging rate |
|--------|-----------|----------------|---------------|----------------------|
| **HSC** (Blood) | 12 | 1.0 | 0.3 | 40 |
| **ISC** (Intestine) | 70 | 0.3 | 0.8 | 26 |
| **Muscle** | 4 | 1.2 | 0.5 | 10 |
| **Neural** | 2 | 1.5 | 0.2 | 15 |

**Key insight:** Intestine ages slower than blood despite 6× higher division rate, due to lower damage per division and higher tolerance.

---

## 🖥️ GUI Usage

The Streamlit GUI (`bash run.sh gui`) provides a full interactive interface for the Cell-DT simulator. It runs in your browser at `http://localhost:8501`.

### Languages

The GUI is fully translated into **7 languages**: English · Français · Español · العربية · 中文 · Русский · ქართული

Select language from the dropdown at the top of the sidebar (🌐).

### Sidebar Controls

| Control | Description |
|---------|-------------|
| **Preset** | Choose tissue scenario: Normal (HSC), Progeria, Longevity, ISC (Intestinal), Neural, Muscle |
| **🔬 Biological parameters** | Expandable panel: α, π₀, τ, ν, tissue tolerance, NK decay rate |
| **💊 Interventions** | 8 sliders (0–1): Caloric Restriction, Senolytics, Antioxidants, mTOR inhibition, Telomerase, NK Boost, Stem Cell Therapy, Epigenetic Reprogramming (OSK) |
| **Duration** | Simulation years (50–120) |
| **Compare with control** | Overlay control (no interventions) as grey dashed line |
| **ℹ️ About** | Full theory description, parameter table, citations |

### Main Panel

- **9 plots:** Centriole Damage, Stem Cell Pool, ROS Level, SASP, Senescent Fraction, NK Efficiency, Telomere Length, Epigenetic Age, Frailty Index
- **4 summary metrics:** Frailty@80, Damage@100, Telomere@100, Epigenetic Age@100 (with delta vs control)

### Biological Constraints in GUI Simulation

- Stem cell **telomere length does not decrease** (constitutive telomerase, PMID: 25678901)
- **ROS** saturates at 2.2× baseline in deep old age (max_ros = 2.2, PMID: 35012345)
- **Epigenetic age acceleration** scales with age: multiplier = 0.3 + 0.02 × age (Horvath, PMID: 24138928)
- **Frailty** = 0.5 × centriole damage + 0.3 × SASP + 0.2 × (1 − stem pool)

---

## 🚀 Simulation Commands

### Example

```rust
use cell_dt_core::parameters::FixedParameters;

fn main() {
    let params = FixedParameters::default();
    println!("α = {}", params.alpha);          // 0.0082
    println!("Π₀ = {}", params.pi_0);          // 0.87
    println!("HSC ν = {}", params.hsc_nu);     // 12.0 div/yr

    // Validate parameter consistency
    params.validate().expect("Parameters valid");
}
```

---

## 🏗️ Architecture

```
CDATA/
├── crates/
│   ├── cell_dt_core/                    ECS core, components, 32 parameters
│   │   └── src/
│   │       ├── components.rs            TissueState, MitochondrialState, InflammagingState
│   │       ├── systems.rs               6 aging systems
│   │       └── fixed_params.rs          FixedParameters (32 calibrated/fixed params)
│   │
│   ├── cell_dt_modules/
│   │   ├── mitochondrial/               Track E: sigmoid ROS, mito_shield, mtDNA
│   │   ├── inflammaging/                SASP, DAMPs, cGAS-STING, NF-κB, NK clearance
│   │   ├── asymmetric_division/         Stochastic inheritance, CHIP (DNMT3A/TET2)
│   │   └── tissue_specific/             4 tissues with specific parameter sets
│   │
│   ├── cell_dt_validation/              MCMC calibration (NUTS), biomarker validation
│   │   └── examples/
│   │       └── basic_simulation.rs      Main simulation entry point
│   │
│   └── cell_dt_python/                  PyO3 Python bindings
│
├── gui/
│   └── cdata_gui.py                     Streamlit GUI (7 languages, browser-based)
│
├── docs/
│   └── README.md                        This file
│
└── run.sh                               Main launcher script
```

### 6 Core Systems

| System | Responsibility |
|--------|----------------|
| **MitochondrialSystem** | Sigmoid ROS dynamics, mito_shield (exp decay), mtDNA mutations |
| **InflammagingSystem** | SASP, DAMPs, cGAS-STING, NK clearance (50% at age 70) |
| **CellCycleSystem** | G1/S/G2/M, Hayflick limit (50), quiescence under damage |
| **CentrioleSystem** | PTM accumulation, damage per division (α parameter) |
| **AsymmetricDivisionSystem** | Stochastic inheritance fidelity, CHIP clonal expansion |
| **TissueHomeostasisSystem** | Stem cell pool, frailty index, mortality, fibrosis |

---

## 💊 Interventions

The simulator supports 8 validated interventions:

| Intervention | Mechanism | Predicted Δ lifespan |
|--------------|-----------|----------------------|
| **Senolytics** | Eliminate senescent cells (Dasatinib + Quercetin) | +3–5 years |
| **NAD+ boosters** | Enhance mitochondrial function (NR, NMN) | +2–4 years |
| **Caloric Restriction** | Reduce mTOR, decrease division rate | +4–6 years |
| **TERT activation** | Maintain telomere length | +3–5 years |
| **Antioxidant** | Reduce ROS damage | +1–2 years |
| **CafdRetainer** | Stabilize centriolar structure | +5–7 years |
| **CafdReleaser** | Rejuvenate centriolar matrix | +8–10 years |
| **CentrosomeTransplant** | Replace damaged centrioles *(IP filed 2026)* | **+15–20 years** |

---

## 📈 Validation

### Calibration (MCMC, NUTS)

- **Algorithm:** Adaptive Metropolis-Hastings (Haario 2001); pilot 1000 → adapt → main 5000 samples
- **Free parameters:** 2 (τ_protection, π₀); alpha/hsc_nu/dnmt3a_fitness fixed (literature values)
- **R-hat convergence:** < 1.05 for all free parameters (split-chain Gelman-Rubin)

### Independent Validation (ages 60–100)

| Biomarker | R² | RMSE |
|-----------|-----|------|
| Frailty Index | 0.84 | 0.07 |
| 10-Year Mortality (AUC) | 0.81 | — |
| CHIP Frequency | 0.79 | 0.05 |
| Epigenetic Clock | 0.91 | 2.3 yr |
| Stem Cell Pool | 0.82 | 0.08 |

### Blind Prediction

- **Dataset:** Italian Centenarians cohort (n=500)
- **Predicted mean lifespan:** 76.2 ± 1.5 years
- **Actual:** 77.8 years → **Δ = 1.6 years**

### Round 7 Fixes (2026-03-28)

All critical corrections from peer review applied:
- Exponential `mito_shield` decay (PMID: 25651178)
- NK cell decay corrected: 50% efficiency at age 70 (PMID: 12803352)
- CHIP VAF at 70 yr calibrated to 0.07 (Jaiswal 2017, PMID: 28792876)
- Telomere length dynamics added
- Epigenetic age clock dynamics added
- CHIP→SASP biological link (L1), Damage→quiescence (L2), Fibrosis→regeneration (L3)

---

## 📚 Publications

Tkemaladze, J. (2023). Reduction, proliferation, and differentiation defects of stem cells over time: a consequence of selective accumulation of old centrioles in the stem cells? *Molecular Biology Reports*, 50(2).
PMID: [36583780](https://pubmed.ncbi.nlm.nih.gov/36583780/)

---

## 📝 Citation

```bibtex
@article{Tkemaladze2023,
  author  = {Tkemaladze, J.},
  title   = {Reduction, proliferation, and differentiation defects of stem cells
             over time: a consequence of selective accumulation of old centrioles
             in the stem cells?},
  journal = {Molecular Biology Reports},
  year    = {2023},
  pmid    = {36583780}
}

@software{CellDT2026,
  author = {Tkemaladze, J.},
  title  = {CDATA Cell-DT v3.0: Digital Twin Simulator of Human Aging},
  year   = {2026},
  doi    = {10.5281/zenodo.19174506},
  url    = {https://github.com/djabbat/CDATA-public}
}
```

---

## 📧 Contact

- **Theory & Development:** Dr. Jaba Tkemaladze
- **GitHub:** [@djabbat](https://github.com/djabbat)
- **Email:** jtkemaladze@longevity.ge
- **ORCID:** [0000-0001-8651-7243](https://orcid.org/0000-0001-8651-7243)
