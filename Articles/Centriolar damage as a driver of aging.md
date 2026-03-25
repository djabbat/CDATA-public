# Cell DT: A Multi-Track Rust-Based Simulator of the Centriolar Damage Accumulation Theory of Aging

**Author:** Jaba Tkemaladze

---

## Abstract

The Centriolar Damage Accumulation Theory of Aging (CDATA) proposes that the structural and functional decline of centrioles and associated structures acts as a candidate integrator and driver of cellular aging, contributing to stem cell exhaustion and tissue dysfunction. To test this hypothesis computationally and explore its systemic consequences, we developed Cell DT, a high-performance, multi-track simulator built in Rust using an Entity Component System (ECS) architecture. Cell DT models seven parallel, interacting aging tracks: (A) ciliary dysfunction from appendage damage, (B) mitotic spindle infidelity, (C) telomere erosion, (D) epigenetic drift, (E) mitochondrial ROS-driven centriolar damage, (F) integrated division rate regulation, and (G) myeloid shift and inflammaging feedback. We further incorporate a thermodynamic layer based on Arrhenius kinetics, a multi-tissue model with systemic signaling (SASP, IGF-1), and clonal hematopoiesis dynamics. Simulations from a baseline human model (5 stem cell niches) recapitulate key aging phenomena: progressive accumulation of centriolar damage (0.006 to 0.577 over 70 years), decline in ciliary (0.991 to 0.215) and spindle (0.999 to 0.713) functions, increasing Systemic Degradation Index (SDI: 0.206 to 0.914), and death at ~80 years. Model outputs were compared to empirical benchmarks, showing consistency in trends for myeloid bias and epigenetic acceleration, while identifying areas for refinement. Multi-tissue simulations (25 niches across five tissues) predict synchronized systemic collapse near age 75, driven by stem cell pool depletion (~7% remaining). We successfully model clonal hematopoiesis (CHIP), with complete clonal takeover by age 79, and a progressive myeloid shift (bias ≈ 0.45 at age 70). Sensitivity analysis identified the midlife damage multiplier as the most critical parameter governing lifespan. Intervention modules (e.g., senolytics, centrosome transplant) demonstrate the platform's utility for *in silico* therapeutic screening, with quantified effects (e.g., calorie restriction extended lifespan by ~15%). Cell DT provides a rigorous, extensible, and open-source framework for simulating the CDATA and exploring its implications for geroscience, offering a "digital twin" approach to understanding the integrated mechanics of aging.

**Keywords:** Aging, Centriole, Digital Twin, Computational Model, Stem Cell Exhaustion, Clonal Hematopoiesis, Rust, ECS.

---

## 1. Introduction

Aging is a complex, systemic process characterized by a time-dependent decline in physiological function and increased vulnerability to disease (López-Otín et al., 2013). While numerous hallmarks of aging have been identified—including genomic instability, telomere attrition, epigenetic alterations, and stem cell exhaustion—a unifying, mechanistic theory that integrates these disparate elements into a coherent causal framework remains elusive (Kennedy et al., 2014). The Centriolar Damage Accumulation Theory of Aging (CDATA) proposes that the centriole, a microtubule-based organelle critical for cell division (via the mitotic spindle) and sensing (via the primary cilium), acts as a candidate integrator of age-related damage (Tkemaladze, 2023). According to CDATA, accumulated damage to centriolar appendage proteins (e.g., CEP164, Ninein) impairs ciliary signaling and spindle fidelity, leading to asymmetric stem cell division, niche exhaustion, and tissue degeneration. This damage is exacerbated by secondary hallmarks, such as mitochondrial ROS and epigenetic drift, creating a self-reinforcing cycle of cellular dysfunction.

Computational modeling has become an indispensable tool in aging research, allowing for the integration of multi-scale data and the testing of dynamical hypotheses that are intractable via experimentation alone (Zhdanov, 2011; Mooney et al., 2016). Agent-based models and systems biology approaches have been used to simulate telomere dynamics (Proellochs & Bischof, 2020), epigenetic clocks (Hannum et al., 2013), and stem cell population kinetics (Rockwood, K., & Mitnitski, A. (2007). Frailty in relation to the accumulation of deficits. *The Journals of Gerontology Series A: Biological Sciences and Medical Sciences, 62*(7), 722-727. https://doi.org/10.1093/gerona/62.7.722

Rodriguez-Brenes et al., 2013). However, few simulators explicitly model organelle-level dynamics as a primary driver of aging, and none, to our knowledge, are built around the centriole as a core component.

Here, we present Cell DT, a novel, high-performance simulation platform written in Rust, designed to implement and test the CDATA. Leveraging an Entity Component System (ECS) architecture for efficiency and modularity, Cell DT simulates seven concurrent aging "tracks," a thermodynamic damage layer, multi-tissue interactions, and clonal dynamics. The platform generates testable predictions regarding the trajectory of centriolar damage, its impact on tissue-specific stem cell pools, and the emergence of aging-associated phenomena like clonal hematopoiesis of indeterminate potential (CHIP) and myeloid-biased hematopoiesis (Yamamoto et al., 2018). We validate model outputs against established empirical benchmarks where possible and conduct a rigorous sensitivity analysis to identify key drivers of simulated aging. This paper details the architecture of Cell DT, presents baseline and multi-tissue simulation results, explores emergent dynamics like CHIP, and demonstrates its utility for evaluating hypothetical anti-aging interventions with quantified outcomes.

---

## 2. Methods

### 2.1. Platform Architecture: Rust ECS Core

Cell DT is implemented in Rust, chosen for its performance, memory safety, and concurrency features. The core simulation engine uses the `hecs` ECS crate, which provides a data-oriented architecture where entities (e.g., cells, niches) are defined by collections of components (e.g., `DamageState`, `CiliaryFunction`), and systems (e.g., `update_damage`, `apply_division`) act on these components. This design decouples data from logic, enabling efficient simulation of thousands of entities with complex, interacting state variables and facilitating modular code development. The ECS pattern is particularly beneficial for biological modeling as it allows for the straightforward addition, removal, or modification of aging tracks, interventions, and entity types without major refactoring. The platform is organized into 14 crates: `cell_dt_core` (ECS engine and track logic), `cell_dt_modules` (specific aging tracks and interventions), `cell_dt_gui` (real-time visualization), `cell_dt_python` (PyO3 bindings), and `cell_dt_io` (data persistence). The codebase includes 439 unit and integration tests to ensure correctness.

The primary results reported in this study are from deterministic simulations (`noise_scale` parameter = 0.0) to ensure reproducibility and clarity in identifying causal chains within the model logic. The deterministic trajectory presented is representative of the mean behavior observed in stochastic runs. However, the framework includes a stochastic Langevin noise term (`noise_scale` in `DamageParams`) to model biological variability. When enabled (e.g., `noise_scale=0.1`), this introduces run-to-run variation, producing an interquartile range (IQR) for lifespan of approximately ±3.5 years (mean ≈ 79.6 years) around the deterministic value. Key phenomena such as CHIP emergence, synchronized tissue collapse, and the sequence of tissue failure remain qualitatively consistent across stochastic replicates (verified over 50 independent runs at noise_scale=0.1).

### 2.2. Simulation Parameters & Aging Tracks

The baseline simulation models a human organism with 5 stem cell niches, a time step (dt) of 1 simulated day, and parameters calibrated to approximate a 75–80 year lifespan, serving as a proof-of-concept for the CDATA framework. Calibration was performed manually through iterative exploratory simulation rather than automated optimization, and parameter values should be interpreted as proof-of-concept estimates pending empirical validation. Seven primary aging tracks are simulated in parallel each time step, with cross-talk between tracks.

- **Track A (Ciliary Dysfunction):** Models damage to centriolar appendage proteins (CEP164, CEP89, Ninein, CEP170). Damage accumulates stochastically with a base rate modified by ROS (from Track E) and thermal acceleration (Arrhenius layer). Ciliary function drives regeneration capacity in tissues.
- **Track B (Spindle Fidelity):** Models the precision of mitotic spindle assembly, dependent on centriole integrity. Decreased spindle fidelity promotes asymmetric divisions, depleting the stem cell pool.
- **Track C (Telomere & Cell Cycle):** Telomere length shortens per division: `telomere -= division_cost × (1 − spindle × 0.3) × ros_factor`. Upon reaching a critical threshold, the cell enters Hayflick-like G1 arrest.
- **Track D (Epigenetic Clock):** A methylation-based age accumulator: `methylation_age += dt × (1 + total_damage × 0.5)`. The `clock_acceleration` factor, rising to 1.33×, modulates the rate of other tracks.
- **Track E (Mitochondrial ROS):** mtDNA mutation load increases with age, elevating ROS production and reducing the `mito_shield` factor, allowing deeper oxygen penetration to centriolar proteins.
- **Track F (Integrated Division Rate):** `division_rate = base_rate × cilia_drive × spindle_drive × age_factor / (1 + ros_brake + mtor_brake)`.
- **Track G (Myeloid Shift & Inflammaging):** `myeloid_bias = (1 − spindle)^1.5 × 0.45 + (1 − cilia) × 0.30 + ros × 0.15 + aggregates × 0.10`. Rising myeloid bias drives `inflammaging_index`, which feeds back to boost systemic ROS and impair niche maintenance, creating a self-amplifying loop between centriolar dysfunction and systemic inflammation.
- **Frailty Index:** The Frailty Index in the model, renamed as the **Systemic Degradation Index (SDI)** for clarity, is a weighted composite metric of systemic decline. It is computed as: `SDI = 0.5 × (1 – average_niche_capacity) + 0.3 × SASP_level + 0.2 × total_systemic_damage`. Each component (average_niche_capacity, SASP_level, total_systemic_damage) is normalized to a [0,1] scale prior to weighting, ensuring the SDI itself remains bounded in [0,1]. This index reflects the progressive exhaustion of stem cell niche functional capacity and the buildup of systemic damage signals, rather than a clinical accumulation of deficits.

Phenotype Count in Table 1 refers to the number of distinct AgingPhenotype flags active simultaneously in the simulation (e.g., ImmuneDecline, CiliaryDysfunction, MyeloidShift, ReplicativeSenescence, OrganicFailure, ApoptosisElevated, NicheExhaustion); each flag is activated by crossing a defined threshold in its corresponding track metric.

**Systemic Layers:**

- **Multi-Tissue Model:** 25 niches across five tissues (Blood, Neural, Gut, Muscle, Skin; 5 niches each). Tissues share a systemic SASP pool and an IGF-1/GH axis that declines with average tissue damage.
- **CHIP/Clonal Drift Module:** For hematopoietic niches, a `NichePool` component tracks clones. Fitness variation arising from stochastic damage accumulation can lead to clonal expansion (Watson et al., 2020).
- **Thermodynamic (Arrhenius) Layer:** Reaction rates are scaled by temperature via the Arrhenius equation `k = A × exp(−Ea / (R × T))`. Baseline temperature: 36.6°C. To model the effect of chronic low-grade inflammation, the local tissue temperature is elevated by up to +2.4°C (a calibrated assumption; see Franceschi et al., 2018 for the inflammaging framework). Activation energies (Ea) for damage reactions were estimated from literature on protein stability and post-translational modification kinetics: carbonylation 50 kJ/mol (Dalle-Donne et al., 2003), aggregation 80 kJ/mol (Dobson, 2003), acetylation 40 kJ/mol, phosphorylation 45 kJ/mol (Johnson & Lewis, 2001), appendage loss 55 kJ/mol. These values reflect the relative thermal sensitivity of different degradation pathways.

**Table S1. Parameter Sources and Justification**
| Parameter | Default Value | Biological Basis / Source | Notes |
| :--- | :--- | :--- | :--- |
| `base_detach_probability` | 0.0003 | Calibrated to produce ~78-yr lifespan in single-niche deterministic run; no direct empirical counterpart (centriolar O₂-detachment is not directly measurable in vivo) | Proof-of-concept estimate; sensitivity: ±50% → ±0.3 yr (low) |
| `midlife_multiplier` | 1.60 | Reflects antagonistic pleiotropy — post-reproductive acceleration of damage; calibrated to match the known acceleration of aging-related dysfunction after age 40 (López-Otín et al., 2013; Kennedy et al., 2014) | Most sensitive parameter (±50% → ∓13/+38 yr); future calibration should target longitudinal proteomics data |
| `senescence_threshold` | 0.75 | Threshold-based arrest analogous to Hayflick limit; value chosen to allow ~50 divisions before senescence, consistent with reported replicative limits (Campisi & d'Adda di Fagagna, 2007) | Floor effect: reducing threshold → immediate collapse |
| `cep164_loss_rate`, `cep89_loss_rate`, `ninein_loss_rate`, `cep170_loss_rate` | Derived from relative OH·-sensitivity: CEP164: 1.50; CEP89: 1.00; Ninein: 0.75; CEP170: 0.55 | Based on relative OH·-sensitivity of each protein derived from oxidative fragility of coiled-coil domains; no direct quantitative data available | Ranked by sensitivity: CEP170 > CEP89/Ninein > CEP164 |
| `ros_feedback_coefficient` | 0.12 | Phenomenological coupling; calibrated so ROS feedback contributes ~1 yr lifespan change per ±50% variation | Low sensitivity; future work: calibrate against mtDNA mutation rates in aging tissues |
| `mito_shield` parameters | — | Fusion/fission balance derived qualitatively from mitochondrial dynamics literature (Chen & Dorn, 2013; Youle & van der Bliek, 2012) | Track E parameter; indirectly calibrated via lifespan output |

*Note:* Parameters lacking empirical counterparts are explicitly identified as proof-of-concept calibrations. A full parameter justification summary is provided in Table S1. Systematic parameter inference against longitudinal omics data is a planned future direction.

### 2.3. Intervention Modules

Eight intervention types are implemented as plug-in systems:

1.  **Senolytics** — removes entities with a `Senescent` tag.
2.  **NAD+ Booster** — reduces ROS and improves spindle fidelity.
3.  **Calorie Restriction** — reduces mTOR signaling and basal damage rate.
4.  **TERT Activation** — slows telomere shortening (Track C).
5.  **Antioxidant** — scavenges ROS (Tracks A and E).
6.  **CafdRetainer** — stabilizes centriolar appendage proteins. (CAFD = Centriolar Appendage Functional Determinants — the molecular complexes anchoring CEP164, CEP89, Ninein, and CEP170 to the distal appendages of the mother centriole)
7.  **CafdReleaser** — promotes turnover of damaged appendages.
8.  **CentrosomeTransplant** — replaces centriolar damage values with donor (young) levels at a specified time point.

### 2.4. Sensitivity Analysis Protocol

To assess the robustness of the model and identify parameters with the greatest influence on the primary output (lifespan), we performed a local, one-at-a-time sensitivity analysis. Starting from the calibrated baseline parameter set (producing a lifespan of 80.5 years), each parameter was independently varied by ±50%. All other parameters were held at their baseline values, and the simulation was run to determine the new lifespan. The change in lifespan (Δ years) was recorded for each perturbation. This analysis focused on core damage accumulation and signaling parameters, including base loss rates for appendage proteins (`cep164_loss_rate`, `cep89_loss_rate`, `ninein_loss_rate`, `cep170_loss_rate`), feedback coefficients (`ros_feedback_coeff`), and key scaling factors such as the `midlife_multiplier` (which scales damage rates after age 40 to model antagonistic pleiotropy) and the `senescence_threshold`. Parameters were ranked by the maximum absolute change in lifespan across the +50% and -50% perturbations, a method chosen to capture asymmetric threshold effects (e.g., senescence_threshold shows no effect at +50% but a large effect at -50%).

The one-at-a-time (OAT) local sensitivity analysis employed here has inherent limitations: it cannot detect parameter interactions in this coupled, non-linear system. For example, the low sensitivity of ros_feedback_coeff may increase substantially when co-varied with midlife_multiplier. Global sensitivity methods (e.g., Sobol indices, Morris screening) are planned for future versions to properly rank parameter importance and quantify interaction effects; implementing these is the nearest-term planned priority, as it will reveal interactions — particularly between midlife_multiplier and ros_feedback_coeff — and sharpen experimental target prioritization. The current OAT results are presented as an initial guide for experimental prioritization.

---

## 3. Results

### 3.1. Baseline Aging Trajectory and Empirical Comparison

The baseline simulation (5 niches, dt = 1 day, deterministic) recapitulates a human aging trajectory over ~80 years (Table 1). Centriolar damage accumulates non-linearly, with an accelerating rate after middle age. Ciliary and spindle functions decline concordantly but at different rates, with ciliary function decaying more rapidly. The Systemic Degradation Index (SDI, formerly Frailty Index) increases steadily. Phenotypic abnormalities emerge after year 20. The simulated organism dies at age **79.6 years**, when all five niches reach exhaustion with inducer state M = 0, D = 8 (Oligopotent). Here M and D denote the count of intact inducers on the mother and daughter centrioles, respectively; Oligopotent indicates that only one centriole retains functional inducers, restricting the stem cell to asymmetric division toward differentiation with no self-renewal capacity.

**Table 1.** Baseline Simulation Output (5 stem cell niches, dt = 1 day)

| Age (years) | Damage Index | Ciliary Function | Spindle Fidelity | Systemic Degradation Index (SDI) | Phenotype Count |
|-------------|:------------:|:----------------:|:----------------:|:-------------:|:---------------:|
| 0           | 0.006        | 0.991            | 0.999            | 0.206         | 0               |
| 10          | 0.065        | 0.904            | 0.979            | 0.270         | 0               |
| 20          | 0.129        | 0.814            | 0.951            | 0.342         | 2               |
| 30          | 0.199        | 0.720            | 0.915            | 0.426         | 3               |
| 40          | 0.278        | 0.614            | 0.873            | 0.525         | 5               |
| 50          | 0.370        | 0.491            | 0.825            | 0.649         | 6               |
| 60          | 0.472        | 0.353            | 0.771            | 0.789         | 6               |
| 70          | 0.577        | 0.215            | 0.713            | 0.914         | 7               |
| **~80**     | —            | —                | —                | —             | Death (M=0, D=8) |

> **Table 1 note:** *Phenotype Count* = number of active AgingPhenotype flags (ImmuneDecline, CiliaryDysfunction, MyeloidShift, ReplicativeSenescence, OrganicFailure, ApoptosisElevated, NicheExhaustion), each triggered by crossing a defined threshold. *SDI* = 0.5x(1-niche_capacity) + 0.3xSASP_level + 0.2xsystemic_damage; all components [0,1].

To contextualize the model's outputs, we compared key metrics against established empirical benchmarks from the literature (Table 2). The model's qualitative trends align with several hallmarks of aging. The computed myeloid bias progression is consistent with murine data. The epigenetic clock acceleration factor (1.33x) aligns with estimates of biological age acceleration in humans. The SDI in the model operates on a different scale (reflecting stem cell niche exhaustion and systemic damage) than clinical frailty indices, a distinction noted in the Limitations. The model predicts a more aggressive depletion of the stem cell pool by late age (~93% loss) than some human estimates, which is discussed as a point for future calibration.

**Table 2.** Comparison of Simulated Outputs to Empirical Benchmarks
| Metric | Simulated Output (Age 70) | Empirical Benchmark (Approx. Age 70) | Source & Notes |
| :--- | :--- | :--- | :--- |
| **Myeloid Bias** | ~0.45 (Moderate Shift) | ~0.45-0.60 in elderly mice | Yamamoto et al., 2018 (*Cell Stem Cell*). Trend consistent. |
| **Epigenetic Acceleration** | Clock acceleration = 1.33x | ~3-4 years per decade after 40 (~1.3-1.4x acceleration) | Hannum et al., 2013. Magnitude consistent. |
| **Stem Cell Pool Depletion** | ~7% of youthful level remaining | Human HSC function estimated 2-5 fold reduction (~20-50% remaining); murine data shows ~10-fold decline. | Rossi et al., 2007 (*Cell Stem Cell*). *Note: Model measures functional niche exhaustion; empirical data often uses phenotypic HSC counts. True functional decline may be more severe than phenotypic data suggest.* |
| **Systemic Degradation Index (SDI)** | 0.914 (model scale: 0-1) | Population mean ~0.25 (clinical frailty index scale: 0-1) | Rockwood & Mitnitski, 2007. *Note: Model SDI reflects niche-driven systemic decline, not accumulation of clinical deficits.* |
| **Telomere Length** | Normalized decline tracked | ~35% reduction from birth (11kb to ~7kb) | Blackburn, 2000; Slagboom et al., 1994. Model tracks normalized trend. |

### 3.2. Multi-Tissue Model with Systemic Signaling

Expanding to a 25-niche, five-tissue model with systemic SASP and a declining IGF-1 axis resulted in a lifespan of **74.8 years** (death cause: frailty). A striking result was the synchronized collapse of all five tissues initiating at approximately year 40. This synchronization was driven by the shared SASP feedback loop and global IGF-1 decline (from 1.000 to 0.490 over 70 years), which uniformly reduced tissue maintenance signals. By age 70, the total stem cell pool across all tissues was depleted to **~7%** of its youthful level. The sequence of final tissue failure was: Gut → Muscle → Blood → Neural → Skin, all within a narrow temporal window. It is important to note that this synchronization is, in part, a consequence of the current model architecture: all five tissue types are represented by homogeneous niches sharing identical intrinsic parameters and a common SASP/IGF-1 axis. *Note: all five tissue types currently use identical division and damage parameters; synchronized collapse is therefore an architectural consequence, not a biological prediction. Tissue-specific parameterization is a planned extension.* Under these conditions, synchronized collapse is an expected outcome rather than an emergent biological finding. Incorporating tissue-specific parameters (e.g., gut epithelium's high division rate vs. neural stem cell quiescence) is a necessary next step to test whether systemic factors genuinely dominate over intrinsic tissue resilience, as the current model assumes.

### 3.3. Modeling Clonal Hematopoiesis (CHIP)

Using the `NichePool` module with 20 HSC niches, CHIP emerged spontaneously. A detectable dominant clone appeared at year 40. The fitness variation driving clonal expansion is an **emergent property** of the model: it arises from stochastic variation in centriolar damage accumulation between individual niches (when `noise_scale` > 0). This differential damage leads to variations in spindle error-induced apoptosis rates, conferring a slight selective advantage to clones with less damage. No fitness parameter is pre-specified. Through this neutral drift and emergent selection, a clone expanded monotonically. By year 79, a single clone occupied **100%** of the simulated HSC pool. This result confirms that the CDATA framework—by generating variation in stem cell fitness through stochastic damage accumulation—can spontaneously produce CHIP, consistent with epidemiological data (Jaiswal & Ebert, 2019).

### 3.4. Myeloid Shift and Inflammaging Dynamics

The computed `myeloid_bias` increased from ~0.1 at baseline to approximately **0.45 at age 70** (Moderate Shift). This shift was driven primarily by declining spindle fidelity (~50% weight) and ciliary function (~30%), with additional contributions from ROS and protein aggregates. The rising myeloid bias created a positive feedback loop via `inflammaging_index`, boosting systemic ROS (`ros_boost = inflammaging_index × 0.15`), linking centriolar dysfunction to systemic inflammation — a recognized hallmark of aging (Franceschi et al., 2018).

### 3.5. Thermodynamic (Arrhenius) Layer Effects

The ze_velocity analog is a dimensionless metric derived from the thermodynamic layer: ze_velocity = entropy_production / (entropy_production + 2.0), where entropy_production accumulates from post-translational modifications (PTMs) on centriolar proteins; it serves as a proxy for the system's rate of disorder and is inspired by Ze-theory's formalization of biological state transitions (Tkemaladze, 2023). The ze_velocity analog peaked at v* ≈ **0.456** around age 20, reflecting maximal entropy production during the period of high mitotic and metabolic activity.

Chronic low-grade inflammation (modeled as a +2.4°C local temperature increase) increased damage accumulation by **14–22%** across tracks, with aggregation being the most thermally sensitive (Ea = 80 kJ/mol). This translated to a lifespan reduction of approximately 3–5 years in otherwise identical simulations.

### 3.6. Quantified *In Silico* Intervention Screening

Preliminary tests of the eight intervention modules demonstrated diverse and quantifiable outcomes (Table 3). Interventions were applied continuously starting at birth unless otherwise noted. **Senolytics** (applied from age 50) reduced SASP burden, modestly slowed frailty progression, and extended lifespan by ~2 years. **Calorie Restriction** was highly effective, extending median lifespan by ~15% to 91.5 years. The hypothetical **CentrosomeTransplant** at age 50 was the most effective within the CDATA framework: by resetting centriolar damage and appendage function to donor levels, it caused a dramatic reversal of functional decline and extended lifespan beyond 100 years (the simulation did not terminate within the 100-year window). **Antioxidants** and **NAD+ Boosters** had modest effects, extending lifespan by 3-5 years, slowing early damage but failing to fully counteract late-stage, aggregation-driven dysfunction.

**Table 3.** Quantified Effects of Simulated Interventions on Lifespan
| Intervention (Start Time) | Lifespan (Years) | Δ vs. Baseline (Years) | Δ vs. Baseline (%) |
| :--- | :--- | :--- | :--- |
| **Baseline (No Intervention)** | 79.6 | — | — |
| **Senolytics (Age 50)** | 81.5 | +1.9 | +2.4% |
| **Calorie Restriction (Birth)** | 91.5 | +11.9 | +14.9% |
| **CentrosomeTransplant (Age 50)** | >100 years (simulation terminated at 100-year cutoff with no death event) | >+20.4 | >+25.6% |
| **Antioxidant (Birth)** | 82.5 | +2.9 | +3.6% |
| **NAD+ Booster (Birth)** | 84.5 | +4.9 | +6.2% |
| **TERT Activation (Birth)** | 86.0 | +6.4 | +8.0% |

### 3.7. Sensitivity Analysis Identifies Key Model Drivers

The one-at-a-time sensitivity analysis (±50% parameter variation) revealed the parameters to which the model's output (lifespan) is most sensitive (Table 4). The `midlife_multiplier` (which scales damage rates after age 40) was by far the most sensitive parameter: a +50% increase shortened lifespan by 12.75 years, while a -50% decrease extended it by 37.73 years. This highlights the critical role of late-life damage acceleration in determining lifespan within the model. The `senescence_threshold` and the base loss rates of key appendage proteins (CEP170, CEP89, Ninein) also showed high sensitivity. In contrast, parameters like `ros_feedback_coeff` and `aggregation_rate` had minimal to no effect on lifespan at ±50% variation, suggesting the model's dynamics are less dependent on these specific feedback loops or that their effects are buffered by other pathways. The asymmetry in the response for some parameters (e.g., `senescence_threshold`: +50% = 0 yr, -50% = -25.68 yr) is biologically meaningful; it reflects a floor effect where increasing the threshold beyond the baseline has no additional impact, while lowering it allows cells to survive with higher damage, significantly extending lifespan.

**Table 4.** Sensitivity Analysis of Key Parameters on Simulated Lifespan (Baseline = 80.5 years)
| Parameter (Baseline Value) | +50% Change: Δ Lifespan (Years) | -50% Change: Δ Lifespan (Years) | Relative Sensitivity* |
| :--- | :--- | :--- | :--- |
| `midlife_multiplier` (1.60) | **-12.75** | **+37.73** | **HIGHEST** |
| `senescence_threshold` (0.75) | 0.00 | -25.68 | High |
| `cep170_loss_rate` (0.0067) | -7.73 | +21.18 | High |
| `cep89_loss_rate` (0.0084) | -3.86 | +17.34 | High |
| `ninein_loss_rate` (0.0084) | -3.86 | +17.34 | High |
| `cep164_loss_rate` (0.0113) | -0.13 | +8.80 | Moderate |
| `ros_feedback_coeff` (0.12) | -1.24 | +1.22 | Low |
| `phospho_dysreg_rate` (0.005) | -0.51 | +0.32 | Low |
| `acetylation_rate` (0.004) | -0.45 | +0.48 | Low |
| `aggregation_rate` (0.002) | 0.00 | 0.00 | None |
***Note:*** *Relative Sensitivity is ranked based on the maximum absolute change in lifespan (years) observed across the +50% and -50% perturbations.*

---

## 4. Discussion

Cell DT provides the first computational instantiation of the CDATA, offering a platform to explore aging from an organelle-centric perspective. By modeling centriolar damage as a core driver intersecting with established hallmarks, the simulator generates trajectories consistent with several observed biological phenomena: stem cell pool depletion, CHIP, myeloid shift, and systemic synchronization of tissue failure. The sensitivity analysis and comparison to empirical benchmarks strengthen the model's utility as a tool for generating hypotheses and identifying critical leverage points within the proposed aging framework.

**Comparison with existing simulators.** Most computational aging models focus on a single hallmark, such as telomere erosion (Proellochs & Bischof, 2020) or epigenetic clocks (Hannum et al., 2013). Agent-based models capture cell–cell interactions but typically lack organelle-level mechanics (Mooney et al., 2016). Cell DT's key innovation is its explicit, multi-track modeling of centriolar function integrated with systemic physiology. The Rust ECS architecture provides superior performance and modularity over many Python-based platforms, enabling efficient simulation of the complex interactions relevant to whole-organism digital twin applications.

**Translational Potential.** The sensitivity analysis revealed that the `midlife_multiplier` (modeling antagonistic pleiotropic acceleration of damage after age 40) is the dominant driver of lifespan within the model (+50% → -13 years; -50% → +38 years). This suggests that interventions targeting the mechanism of this late-life damage amplification—particularly the stabilization of the most sensitive centriolar appendage proteins (CEP170, CEP89, Ninein)—could be substantially more impactful than strategies targeting reactive oxygen species alone (`ros_feedback_coeff` ±50% → only ±1.2 years). This generates a testable experimental hypothesis: targeted stabilization of CEP170 or related proteins should extend functional lifespan in model organisms more effectively than broad antioxidant treatments. This prediction is amenable to experimental testing using available tools, such as knock-in mice with engineered stabilization of centriolar appendage proteins, or targeted siRNA knockdown of appendage-degrading enzymes; pharmacological small-molecule screens for appendage stabilizers represent a further avenue.

**Limitations and Future Directions.** The current study has several limitations that guide future work. First, while parameters were calibrated to produce a plausible lifespan, precise rate constants require experimental validation against longitudinal measures of centriolar protein integrity, ciliary function, and spindle fidelity with age. The sensitivity analysis provides a roadmap for prioritizing which parameters need the most accurate estimation. Second, the multi-tissue model employs a major simplification: representing tissues as collections of identical stem cell niches. The synchronized collapse observed may be an artifact of this homogeneity; introducing tissue-specific damage rates, repair capacities, and niche architectures is a necessary next step to model differential tissue aging. Third, the intervention modules are highly abstracted and do not capture pharmacokinetics, side effects, or combinatorial interactions. Finally, the CDATA itself remains a theoretical framework; Cell DT demonstrates its internal consistency and ability to produce aging-like phenotypes but does not prove causality. The platform is designed to generate testable predictions (e.g., specific sequences of centriolar protein loss, correlations between ciliary dysfunction and myeloid bias) that can guide targeted experiments.

**Future directions.** Planned modules include a detailed immune simulator for immunosenescence, a spatial metabolism engine, and integration with single-cell RNA-seq data to inform cell state transitions. A WebAssembly build of the egui interface is planned for browser-accessible demonstrations. The existing Python bindings (`cell_dt_python`) enable community extension and integration with bioinformatics pipelines. The platform could also be adapted to model progeroid syndromes and cancer evolution, where centriolar abnormalities are well documented (Godinho & Pellman, 2014). Furthermore, the modular ECS architecture means individual aging tracks can be replaced or supplemented with other organelle-centric modules without restructuring the simulation core. This positions Cell DT as a general digital twin toolkit for testing competing theories of aging, not limited to CDATA. For instance, a mitochondrial-centric aging model could be implemented by replacing the centriole damage accumulation track (Track A/B) with a mitochondrial fission/fusion dynamics module, while retaining the systemic SASP, IGF-1, and multi-tissue layers unchanged — demonstrating the architectural benefit of ECS component isolation.

---

## 5. Conclusion

We have developed Cell DT, a high-performance, multi-track simulation platform for the Centriolar Damage Accumulation Theory of Aging. Built in Rust with an ECS architecture and 439 passing tests, it models seven interacting aging processes centered on centriolar integrity, alongside systemic signaling, clonal dynamics, and thermodynamic effects. Baseline simulations recapitulate key features of human aging, and outputs show consistency with empirical data for myeloid shift and epigenetic acceleration. The platform generates emergent age-related phenomena—clonal hematopoiesis, myeloid shift, synchronized tissue collapse—and enables quantified *in silico* screening of hypothetical interventions. Sensitivity analysis identified the post-midlife damage acceleration rate as the most critical parameter governing lifespan within the model. Cell DT provides a concrete, computable framework for CDATA, generating testable predictions to guide future experimental research in geroscience and serving as an extensible foundation for more detailed digital twin models of aging.

---

## References

Blackburn, E. H. (2000). Telomere states and cell fates. *Nature, 408*(6808), 53–56. https://doi.org/10.1038/35040500

Campisi, J., & d'Adda di Fagagna, F. (2007). Cellular senescence: when bad things happen to good cells. *Nature Reviews Molecular Cell Biology, 8*(9), 729–740. https://doi.org/10.1038/nrm2233

Dalle-Donne, I., Rossi, R., Giustarini, D., Milzani, A., & Colombo, R. (2003). Protein carbonyl groups as biomarkers of oxidative stress. *Clinica Chimica Acta, 329*(1-2), 23–38. https://doi.org/10.1016/S0009-8981(03)00003-2

Dobson, C. M. (2003). Protein folding and misfolding. *Nature, 426*(6968), 884–890. https://doi.org/10.1038/nature02261

Franceschi, C., Garagnani, P., Parini, P., Giuliani, C., & Santoro, A. (2018). Inflammaging: a new immune–metabolic viewpoint for age-related diseases. *Nature Reviews Endocrinology, 14*(10), 576–590. https://doi.org/10.1038/s41574-018-0059-4

Godinho, S. A., & Pellman, D. (2014). Causes and consequences of centrosome abnormalities in cancer. *Philosophical Transactions of the Royal Society B: Biological Sciences, 369*(1650), 20130467. https://doi.org/10.1098/rstb.2013.0467

Hannum, G., Guinney, J., Zhao, L., Zhang, L., Hughes, G., Sadda, S., & Zhang, K. (2013). Genome-wide methylation profiles reveal quantitative views of human aging rates. *Molecular Cell, 49*(2), 359–367. https://doi.org/10.1016/j.molcel.2012.10.016

Jaiswal, S., & Ebert, B. L. (2019). Clonal hematopoiesis in human aging and disease. *Science, 366*(6465), eaan4673. https://doi.org/10.1126/science.aan4673

Johnson, L. N., & Lewis, R. J. (2001). Structural basis for control by phosphorylation. *Chemical Reviews, 101*(8), 2209–2242. https://doi.org/10.1021/cr000225s

Kennedy, B. K., Berger, S. L., Brunet, A., Campisi, J., Cuervo, A. M., Epel, E. S., & Sierra, F. (2014). Geroscience: linking aging to chronic disease. *Cell, 159*(4), 709–713. https://doi.org/10.1016/j.cell.2014.10.039

Lezhava, T., Monaselidze, J., Kadotani, T., Dzhokhadze, T., Tkemaladze, J., Kiguradze, T., & Sigua, T. (2011). Two types of chromatin remodeling in aging cells. *Biogerontology, 12*(3), 195–204. https://doi.org/10.1007/s10522-010-9312-5

López-Otín, C., Blasco, M. A., Partridge, L., Serrano, M., & Kroemer, G. (2013). The hallmarks of aging. *Cell, 153*(6), 1194–1217. https://doi.org/10.1016/j.cell.2013.05.039

Mooney, K. M., Morgan, A. E., & Mc Auley, M. T. (2016). Aging and computational systems biology. *Wiley Interdisciplinary Reviews: Systems Biology and Medicine, 8*(2), 123–139. https://doi.org/10.1002/wsbm.1328

Proellochs, N., & Bischof, J. (2020). A reaction network approach to modeling the human telomere length maintenance system. *Bioinformatics, 36*(1), 78–86. https://doi.org/10.1093/bioinformatics/btz499

Rodriguez-Brenes, I. A., Komarova, N. L., & Wodarz, D. (2013). Evolutionary dynamics of feedback escape and the development of stem-cell–driven cancers. *Proceedings of the National Academy of Sciences, 110*(52), 20906–20911. https://doi.org/10.1073/pnas.1311312110

Rossi, D. J., Bryder, D., & Weissman, I. L. (2007). Hematopoietic stem cell aging: mechanism and consequence. *Cell Stem Cell, 1*(1), 9–11. https://pubmed.ncbi.nlm.nih.gov/17909015/

Schultz, M. B., & Sinclair, D. A. (2016). When stem cells grow old: phenotypes and mechanisms of stem cell aging. *Development, 143*(1), 3–14. https://doi.org/10.1242/dev.130633

Slagboom, P. E., Droog, S., & Boomsma, D. I. (1994). Genetic determination of telomere size in humans: a twin study of three age groups. *American Journal of Human Genetics, 55*(5), 876–882.

Tkemaladze, J. (2023). The centriolar damage accumulation theory of aging (CDATA). *Molecular Biology Reports, 50*, 2167–2175. https://doi.org/10.1007/s11033-022-08045-z

Tkemaladze, J. (2025). Cell DT — CDATA simulation platform [Data set]. Zenodo. https://doi.org/10.5281/zenodo.19174506

Watson, C. J., Papula, A. L., Poon, G. Y. P., Wong, W. H., Young, A. L., Druley, T. E., & Blundell, J. R. (2020). The evolutionary dynamics and fitness landscape of clonal hematopoiesis. *Science, 367*(6485), 1449–1454. https://doi.org/10.1126/science.aay9333

Yamamoto, R., Wilkinson, A. C., Ooehara, J., Lan, X., Lai, C.-Y., Nakauchi, Y., & Nakauchi, H. (2018). Large-scale clonal analysis resolves aging of the mouse hematopoietic stem cell compartment. *Cell Stem Cell, 22*(4), 600–607.e4. https://doi.org/10.1016/j.stem.2018.03.013

Zhdanov, V. P. (2011). Kinetic models of gene expression including non-coding RNAs. *Physics of Life Reviews, 8*(1), 88–104. https://doi.org/10.1016/j.plrev.2011.01.002