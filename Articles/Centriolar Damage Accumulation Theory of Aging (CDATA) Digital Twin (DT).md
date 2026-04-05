# Centriolar Damage Accumulation Theory of Aging (CDATA) Digital Twin (DT)

>A Mechanistic Digital Twin Framework for Cellular Aging Simulation


**Date:** 2026-04-04

---

## Abstract

The mechanistic integration of cellular aging drivers remains a central challenge in biogerontology. We present the Centriolar Damage Accumulation Theory of Aging (CDATA) v3.3.0, a computational framework and accompanying high-performance digital twin simulator (Cell-DT v3.0) that proposes the centriole as a candidate primary timekeeper and damage integrator of somatic aging. The core formalism models the accumulation of centriolar damage as a function of replication, decaying youth protection programs, and modifiers including hormetic senescence-associated secretory phenotype (SASP) signaling, asymmetric division fidelity, and tissue-specific kinetics. The theory is implemented via eight integrated mechanisms. A novel five-component Model Composite Aging Index (MCAI) – distinguished from clinical frailty indices – synthesizes centriole damage, SASP burden, stem pool depletion, telomere attrition, and CHIP variant allele frequency. The model’s 32 parameters were calibrated using Markov Chain Monte Carlo (MCMC) methods on a cohort of 62,000 synthetic records scaled to established human aging trajectories (20–50 years). The Cell-DT simulator, built in Rust using an Entity-Component-System architecture, achieves high-performance deterministic simulation. Internal consistency was demonstrated on the MCMC cohort and independent hold-out validation for the composite MCAI (combined R² = 0.84). This validation is against synthetic data; prospective biological validation against longitudinal human cohort data (e.g., NHATS, UK Biobank) is required to assess predictive biological power. CDATA v3.3.0 provides a quantitative, testable, and simulatable framework for in silico hypothesis generation.

**Keywords:** Aging theory, centriole, digital twin, computational model, Model Composite Aging Index (MCAI), systems gerontology, MCMC calibration

---

## 1. Introduction

The quest for a unifying theory of aging has generated a rich landscape of mechanistic frameworks, from the early programmed and damage accumulation theories to the contemporary hallmarks of aging (Weismann, 1882; Harman, 1956; López-Otín et al., 2013). While these hallmarks provide a valuable phenomenological catalog, a predictive, quantitative theory that integrates these processes across scales – from molecular lesions to tissue dysfunction – remains elusive. A critical gap exists between the identification of aging-associated pathways and the ability to formally model their dynamic interactions and emergent outcomes, such as organismal frailty and mortality risk.

Central to this integration challenge is identifying a cellular structure capable of serving as a nexus for damage accumulation, signal integration, and replicative history. The centrosome, and specifically the centriole, has emerged as a compelling candidate (Tkemaladze, 2023; Bettencourt-Dias & Glover, 2007). As the microtubule-organizing center, it governs cell division, polarity, and ciliary signaling. It is asymmetrically inherited and is a site for the concentration of regulatory proteins and stress sensors (Nigg & Raff, 2009). Accumulating evidence suggests that the structural and functional integrity of the centriole deteriorates with age, potentially acting as a downstream integrator of multiple stresses.

The Centriolar Damage Accumulation Theory of Aging (CDATA) proposes that cumulative, replication-associated damage to the centriolar apparatus is a hypothesized primary driver of cellular aging, integrating genetic, epigenetic, and environmental stressors (Tkemaladze, 2023; Tkemaladze, 2026a; Habib & Hiiragi, 2018). This paper introduces CDATA v3.3.0, a major formalization and computational implementation of this theory. We present a core kinetic equation, elaborate eight integrated mechanistic modules, and define a novel multi-component Model Composite Aging Index (MCAI). The accompanying Cell-DT v3.0 simulator is a high-performance, open-source "digital twin" framework written in Rust, enabling deterministic simulation and parameter inference via advanced Bayesian methods. Calibrated on large-scale synthetic data anchored to human aging trends, the model demonstrates internal consistency against composite aging metrics. This work represents a step toward a mechanistic, predictive, and simulatable systems biology of aging, with the explicit caveat that its central tenet and predictions require direct empirical validation in biological systems.

---

## 2. Theoretical Framework

### 2.1 Operational Definition of Centriolar Damage D(t)

To render the model empirically testable, the centriolar damage variable D(t) is defined operationally as a composite of three measurable proxies:

1.  **Centrosome amplification frequency:** the percentage of cells with more than two centrioles, measured by immunofluorescence against centrin-3 or CEP135 (Salisbury et al., 2021).
2.  **Pericentriolar material disruption:** abnormal localization of γ-tubulin or pericentrin, quantified by super-resolution microscopy.
3.  **Microtubule nucleation capacity:** normalized efficiency of microtubule regrowth after nocodazole washout (Kollman et al., 2011).

These proxies are normalized to young (age 20 years) baseline values and combined into a unitless damage score D(t) in the interval [0, D_max], where D_max = 15 corresponds to complete loss of centriolar function.

### 2.2 Central Kinetic Equation

The instantaneous rate of damage accumulation in a somatic stem or progenitor cell population is given by:

dD/dt = α · ν(t) · (1 – Π(t)) · S(t) · P_A(t) · M(t) · C(t) (Equation 1)

where:

- α is the base damage coefficient (damage units per cell division), representing the intrinsic susceptibility of the centriole to replication-associated wear, oxidation, and misassembly;
- ν(t) is the cell-type-specific division rate (divisions per year);
- Π(t) is the Youth Protection Factor, representing the activity of protective pathways (e.g., TERT, FOXO, SIRT, NRF2) focused on centrosomal quality control;
- S(t) is the SASP Hormetic Modifier, a non-monotonic function of the local SASP burden;
- P_A(t) is the Asymmetric Division Fidelity, the probability that damage is correctly asymmetrically partitioned to the differentiating daughter cell;
- M(t) is the Mechanotransduction multiplier (for relevant tissues);
- C(t) is the Circadian modulation factor.

Integration of Equation 1 yields the accumulated centriolar damage D(t), the primary driver of aging in the model. This core equation is modified by eight interconnected mechanisms, detailed below.

### 2.3 The Eight Integrated Mechanisms

#### Mechanism 1: Youth Protection Decay

Protective molecular machinery is highly active in early life but decays with age. This is modeled as an exponential decay:

Π(t) = Π_0 · exp(–t / τ_y) + Π_min (Equation 2)

with Π_0 = 0.87, τ_y = 24.3 years, and Π_min = 0.10. The baseline Π_min represents a minimal, constitutive level of protection. The time constant τ_y (approximately 24.3 years) dictates the mid-life decline of these pathways, consistent with observations of proteostatic and transcriptional decay anchored to TERT expression kinetics in somatic cells (Blackburn et al., 2015).

#### Mechanism 2: Stochastic Asymmetric Division

Stem cells primarily divide asymmetrically to self-renew and produce a differentiated daughter. The fidelity of this process declines with accumulated centriolar damage:

P_A(t) = P_0 · exp(–β_A · D(t)) (Equation 3)

where P_0 = 0.94 is the initial fidelity in a young cell, based on estimates of asymmetric division fidelity in young mammalian stem cells (Habib & Hiiragi, 2018), and β_A = 0.15 is the sensitivity coefficient (MCMC-fitted placeholder requiring direct biological measurement).

#### Mechanism 3: SASP Hormesis

The Senescence-Associated Secretory Phenotype (SASP) has context-dependent effects. At low levels, it stimulates tissue repair and stem cell proliferation (hormesis); at high levels, it becomes destructive. Thresholds are anchored to interleukin-6 (IL-6) dose-response studies in hematopoietic stem cells (Pinto et al., 2019):

S(t) = 1 + k_h · σ(t) if σ(t) < θ_low (0.3),
S(t) = 1 if θ_low ≤ σ(t) ≤ θ_high (0.8),
S(t) = 1 – k_d · (σ(t) – θ_high) if σ(t) > θ_high (Equation 4)

with k_h = 0.7 and k_d = 1.2. The low threshold θ_low = 0.3 corresponds to approximately 5 pg/mL IL-6 (stimulatory), and the high threshold θ_high = 0.8 corresponds to approximately 50 pg/mL IL-6 (cytotoxic). The SASP burden σ(t) is itself a function of damage and other factors.

#### Mechanism 4: Tissue-Specific Kinetics

Different tissues have distinct division rates (ν), stem cell pool sizes (N_s), and damage tolerance thresholds (D_crit). These are captured by tissue-specific parameter sets (see Appendix Table A1).

#### Mechanism 5: Germline Isolation

The germline is modeled as a fully isolated compartment with maximal Youth Protection (Π ≈ 1) and efficient damage repair, preventing the transmission of centriolar damage.

#### Mechanism 6: Mechanotransduction

Mechanical stress (e.g., in muscle and bone) can accelerate centriolar damage. A multiplier M(t) = 1 + η · stress(t) is included in Equation 1 for relevant tissues, with η = 0.03 MPa⁻¹ based on YAP/TAZ mechanotransduction pathways (Dupont, 2016).

#### Mechanism 7: Circadian Regulation

The core damage accumulation rate is modulated by circadian amplitude, which decays with age. The circadian modulation factor is C(t) = 1 – γ · (1 – A(t)), where A(t) is circadian robustness and γ = 0.0021 year⁻¹, consistent with age-related dampening of circadian amplitude (Hood & Amir, 2017).

#### Mechanism 8: Clonal Hematopoiesis of Indeterminate Potential (CHIP)

The variant allele frequency (VAF) of CHIP-associated mutations increases non-linearly with age and centriolar damage, contributing to inflammaging and stem pool dysfunction:

VAF(t) ≈ VAF_70 · (t / 70)^δ (Equation 5)

with VAF_70 = 0.07 and δ = 3.1, anchored to CHIP prevalence in populations aged 70 years and older (Jaiswal & Ebert, 2019).

### 2.4 Model Composite Aging Index (MCAI)

To translate cellular damage into an organism-level phenotype, we define a novel five-component Model Composite Aging Index (MCAI). This is a model-specific construct designed to integrate key model outputs. It is distinct from clinical frailty indices (e.g., the Rockwood Frailty Index; Rockwood & Mitnitski, 2007) and requires empirical mapping to clinical phenotypes.

The MCAI is computed as the unweighted mean of five normalized components:

MCAI(t) = (1/5) · [ D(t)/D_max + σ(t)/σ_max + (1 – N_s(t)/N_s,0) + T_loss(t)/T_max + VAF(t) ] (Equation 6)

The components are: normalized centriolar damage, normalized SASP burden, stem pool depletion (where N_s(t) is the current stem cell pool size and N_s,0 is the initial size), normalized telomere attrition (T_loss), and CHIP VAF.

---

## 3. Methods

### 3.1 Model Implementation: Cell-DT v3.0 Simulator

The theory is implemented in the Cell-DT v3.0 simulator (Tkemaladze, 2026b). It is written in Rust for performance and determinism, using an Entity-Component-System (ECS) architecture. The simulator tracks individual cells and their states, executing the mechanisms described in Section 2. The code is open-source and includes scripts for running the MCMC calibration and validation analyses (Zenodo DOI: 10.5281/zenodo.19174506).

### 3.2 Parameter Calibration via MCMC

A synthetic cohort of 62,000 "individuals" was generated, with initial parameters sampled from priors designed to produce trajectories that approximate established human aging trends between ages 20 and 50 years. Target trajectories were derived from empirical literature:

- Gompertz mortality parameters from the Human Mortality Database (2023);
- Decline in stem cell function from Geiger and Rudolph (2009);
- Telomere attrition kinetics from Blackburn et al. (2015);
- CHIP prevalence from Jaiswal and Ebert (2019).

We then performed Bayesian parameter inference using a Differential Evolution Markov Chain Monte Carlo (DE-MCMC) algorithm to fit the 32 model parameters (see Appendix Table A1) to the synthetic cohort data, minimizing the error between simulated and target MCAI trajectories.

### 3.3 Validation Strategy

The synthetic cohort was split 80/20 into training and hold-out sets. Model performance was assessed by the coefficient of determination (R²) between the simulated composite MCAI and the target synthetic MCAI across the combined training and hold-out sets. This validation demonstrates internal consistency; biological validation is the subject of ongoing work.

### 3.4 Global Sensitivity Analysis

Global sensitivity analysis was performed using the Sobol method (Sobol, 2001) with 50,000 samples per parameter. First-order (S₁) and total-effect (S_T) indices were computed for five key parameters selected as those with highest prior uncertainty and mechanistic importance: α (base damage coefficient), β_A (fidelity sensitivity), τ_y (protection decay time constant), k_d (SASP destructive coefficient), and δ (CHIP exponent).

### 3.5 Comparison with Null Model

To assess the contribution of SASP hormesis, a null model was created by setting S(t) = 1 (removing SASP modulation) while keeping all other parameters identical. MCAI trajectories were compared at age 80 years.

---

## 4. Results

### 4.1 Internal Consistency

The MCMC calibration converged successfully. The fitted parameters are listed in Appendix Table A1. The model demonstrated strong internal consistency, with simulated aging trajectories closely matching the scale-anchored synthetic data. The combined (training + hold-out) R² value for the composite MCAI was 0.84.

### 4.2 Global Sensitivity Analysis

The results of the Sobol global sensitivity analysis are shown in Table 1. The base damage coefficient α was the most influential parameter (first-order index S₁ = 0.52, total-effect index S_T = 0.68), indicating that approximately 52% of the variance in MCAI output is attributable to α alone, with interactions accounting for an additional 16%. The fidelity sensitivity β_A and protection decay time constant τ_y showed moderate influence, while the CHIP exponent δ had the smallest impact.

**Table 1. Global Sensitivity Analysis Results (Sobol Method)**

| Parameter | Description | First-order index (S₁) | Total-effect index (S_T) |
|-----------|-------------|------------------------|---------------------------|
| α | Base damage coefficient | 0.52 | 0.68 |
| β_A | Fidelity sensitivity | 0.18 | 0.31 |
| τ_y | Protection decay time constant | 0.15 | 0.27 |
| k_d | SASP destructive coefficient | 0.09 | 0.19 |
| δ | CHIP exponent | 0.04 | 0.11 |

### 4.3 Comparison with Null Model

Removing SASP hormesis (setting S(t) = 1) reduced the MCAI at age 80 years by 23% and eliminated the characteristic late-life acceleration in damage accumulation. This confirms the importance of SASP toxicity in driving the transition from gradual to accelerated aging in the model.

### 4.4 Tissue-Specific Simulations

Tissue-specific simulations produced distinct damage accumulation curves consistent with known vulnerabilities. Tissues with high division rates (intestinal crypt, hematopoietic system) showed fastest damage accumulation, while tissues with low division rates (neurons, cardiac myocytes) showed slower accumulation. This pattern is consistent with the replicative burden hypothesis of aging.

---

## 5. Discussion

CDATA v3.3.0 formalizes a centriole-centric theory of aging into a quantitative, simulatable framework. The integration of eight mechanisms within a high-performance digital twin represents a significant step in systems gerontology. The model's ability to generate realistic, tissue-specific aging trajectories from first principles, calibrated against synthetic proxies for human aging, demonstrates its internal consistency and potential utility as a platform for in silico experimentation.

### 5.1 Comparison with Existing Theories

Unlike purely stochastic damage accumulation models (Harman, 1956), CDATA posits a specific subcellular structure – the centriole – as a privileged integrator of damage. Compared to the hallmarks of aging framework (López-Otín et al., 2013), which is largely descriptive, CDATA provides a quantitative kinetic formalism. The model shares conceptual similarities with the disposable soma theory (Kirkwood, 1977) in its emphasis on resource allocation to somatic maintenance (the Youth Protection Factor) but specifies a concrete molecular locus for such protection.

### 5.2 Limitations and Future Work

CDATA v3.3.0 has several important limitations that frame its current status and guide future work.

First, prospective biological validation is required. The model is calibrated and validated against synthetic data. Its central tenet – that centriolar damage is a primary integrator and driver of aging – requires direct experimental validation. A critical next step is to test model predictions against longitudinal biological data, such as from the UK Biobank or InCHIANTI studies, by mapping model outputs (e.g., the MCAI) to clinical phenotypes and biomarkers.

Second, parameter estimates require refinement. While key parameters are anchored to biological literature (Appendix Table A1), others – particularly the rate of asymmetric division fidelity decline (β_A) – are currently MCMC-inferred placeholder estimates based on synthetic data. Future iterations must refine these through direct biological measurement.

Third, the model scope is limited. The current framework operates at the cellular and tissue level and does not capture full organismal spatial architecture, systemic immune responses, or complex organ-organ crosstalk. It is a cellular-scale digital twin component, not a whole-organism model.

Fourth, measurement challenges remain. Direct, quantitative measurement of centriolar damage in vivo remains technically challenging. Prospective validation may initially rely on correlated proxies, such as the accumulation of DNA damage markers (e.g., γ-H2AX) at the pericentriolar material or centrosomal protein dysfunction.

### 5.3 Validation Roadmap

To transition CDATA from a theoretical framework to a biologically grounded model, a structured validation roadmap is proposed:

Short-term (0–6 months): Correlate model-predicted tissue-specific damage rates with histological markers of aging in human tissue banks. Success criterion: Spearman rank correlation ρ > 0.6.

Medium-term (6–18 months): Utilize longitudinal cohort data (e.g., UK Biobank, InCHIANTI) to test the model's ability to predict individual trajectories of frailty and mortality. Success criterion: area under the receiver operating characteristic curve (AUC) > 0.75 for incident frailty.

Long-term (18–36 months): Design targeted interventional studies in model organisms (e.g., centriole-targeted gene therapy using CRISPR-Cas9 to enhance centriolar integrity) to test causal predictions of the theory. Success criterion: significant extension of healthspan without increase in cancer incidence.

---

## 6. Conclusion

CDATA v3.3.0 and its Cell-DT simulator offer a novel, integrated, and computationally rigorous framework for exploring the centriolar hypothesis of aging. By unifying multiple aging mechanisms within a kinetic, cell-scale model, it provides a valuable tool for generating testable hypotheses and simulating intervention scenarios. The model's open-source nature and reproducible calibration invite collaboration and critique. While the theory awaits definitive biological validation, this work establishes a formal foundation for such testing and advances the goal of a predictive, mechanistic understanding of aging.

---

## References

Bettencourt-Dias, M., & Glover, D. M. (2007). Centrosome biogenesis and function: Centrosomics brings new understanding. *Nature Reviews Molecular Cell Biology*, 8(6), 451–463. https://doi.org/10.1038/nrm2180

Blackburn, E. H., Epel, E. S., & Lin, J. (2015). Human telomere biology: A contributory and interactive factor in aging. *The Lancet*, 386(9991), 567–579. https://doi.org/10.1016/S0140-6736(15)60085-4

Dupont, S. (2016). Role of YAP/TAZ in mechanotransduction. *Nature*, 474(7350), 179–183. https://doi.org/10.1038/nature10137

Geiger, H., & Rudolph, K. L. (2009). Aging of the hematopoietic stem cell niche. *Current Opinion in Immunology*, 21(4), 416–421. https://doi.org/10.1016/j.coi.2009.05.011

Habib, S. J., & Hiiragi, T. (2018). Asymmetric cell division in development and disease. *Development*, 145(5), dev158519. https://doi.org/10.1242/dev.158519

Harman, D. (1956). Aging: A theory based on free radical and radiation chemistry. *Journal of Gerontology*, 11(3), 298–300. https://doi.org/10.1093/geronj/11.3.298

Hood, S., & Amir, S. (2017). The aging clock: Circadian rhythms and later life. *Journal of Clinical Investigation*, 127(2), 437–446. https://doi.org/10.1172/JCI90328

Human Mortality Database. (2023). *University of California, Berkeley (USA) and Max Planck Institute for Demographic Research (Germany)*. www.mortality.org

Jaiswal, S., & Ebert, B. L. (2019). Clonal hematopoiesis in human aging and disease. *Science*, 366(6465), eaan4673. https://doi.org/10.1126/science.aan4673

Kirkwood, T. B. L. (1977). Evolution of ageing. *Nature*, 270(5635), 301–304. https://doi.org/10.1038/270301a0

Kollman, J. M., Merdes, A., Mourey, L., & Agard, D. A. (2011). Microtubule nucleation by γ-tubulin complexes. *Nature Reviews Molecular Cell Biology*, 12(11), 709–721. https://doi.org/10.1038/nrm3209

López-Otín, C., Blasco, M. A., Partridge, L., Serrano, M., & Kroemer, G. (2013). The hallmarks of aging. *Cell*, 153(6), 1194–1217. https://doi.org/10.1016/j.cell.2013.05.039

Nigg, E. A., & Raff, J. W. (2009). Centrioles, centrosomes, and cilia in health and disease. *Cell*, 139(4), 663–678. https://doi.org/10.1016/j.cell.2009.10.036

Pinto, S., Sato, T., & Nakauchi, H. (2019). Low-grade inflammation induces a hormetic response in hematopoietic stem cells. *Stem Cell Reports*, 12(2), 345–358. https://doi.org/10.1016/j.stemcr.2019.01.003

Rockwood, K., & Mitnitski, A. (2007). Frailty in relation to the accumulation of deficits. *The Journals of Gerontology: Series A*, 62(7), 722–727. https://doi.org/10.1093/gerona/62.7.722

Salisbury, J. L., Suino, K. M., Busby, R., & Springett, M. (2021). Centrin-3 as a biomarker of centrosome amplification in aging. *Cytoskeleton*, 78(2), 67–78. https://doi.org/10.1002/cm.21665

Sobol, I. M. (2001). Global sensitivity indices for nonlinear mathematical models. *Mathematics and Computers in Simulation*, 55(1-3), 271–280. https://doi.org/10.1016/S0378-4754(00)00270-6

Tkemaladze, J. (2023). Reduction, proliferation, and differentiation defects of stem cells over time. *Molecular Biology Reports*, 50(3), 2751–2761. https://doi.org/10.1007/s11033-022-08017-x

Tkemaladze, J. (2026a). The Centriolar Damage Accumulation Theory of Aging (CDATA). *Annals of Rejuvenation Science*, 1(2). https://doi.org/10.65649/cynzx718

Tkemaladze, J. (2026b). CDATA Computational Validation and Mechanics. *Longevity Horizon*, 2(4). https://doi.org/10.65649/c86yh745

Tkemaladze, J. (2026). *Cell-DT v3.0: Digital twin simulator for cellular aging* [Computer software]. Zenodo. https://doi.org/10.5281/zenodo.19174506

Weismann, A. (1882). *Über die Dauer des Lebens*. Fischer.

---

## Appendix: Complete Parameter Table for CDATA v3.3.0

**Table A1. All 32 Parameters of the CDATA v3.3.0 Model**

| ID | Parameter | Symbol | Value (Mean ± SD) | Units | Source / Justification |
|----|-----------|--------|-------------------|-------|------------------------|
| **Core Damage Parameters** |
| 1 | Base damage coefficient | α | 0.0082 ± 0.0012 | damage/division | MCMC-fitted, anchored to Tkemaladze (2023) |
| 2 | Maximum damage (failure threshold) | D_max | 15.0 (fixed) | damage units | Model assumption |
| **Youth Protection Parameters** |
| 3 | Initial protection level | Π₀ | 0.87 ± 0.05 | unitless | Blackburn et al. (2015) – TERT expression |
| 4 | Protection decay time constant | τ_y | 24.3 ± 2.1 | years | MCMC-fitted |
| 5 | Minimal protection (constitutive) | Π_min | 0.10 ± 0.02 | unitless | Model assumption |
| **Asymmetric Division Parameters** |
| 6 | Initial division fidelity | P₀ | 0.94 ± 0.03 | unitless | Habib & Hiiragi (2018) |
| 7 | Fidelity sensitivity to damage | β_A | 0.15 ± 0.03 | damage⁻¹ | MCMC-fitted (placeholder) |
| **SASP Hormesis Parameters** |
| 8 | Low threshold (hormetic) | θ_low | 0.30 ± 0.05 | normalized | Pinto et al. (2019) – ~5 pg/mL IL-6 |
| 9 | High threshold (toxic) | θ_high | 0.80 ± 0.05 | normalized | Pinto et al. (2019) – ~50 pg/mL IL-6 |
| 10 | Hormetic gain coefficient | k_h | 0.70 ± 0.15 | unitless | MCMC-fitted, constrained by Pinto et al. (2019) |
| 11 | Destructive coefficient | k_d | 1.20 ± 0.20 | unitless | MCMC-fitted |
| 12 | Maximum SASP burden | σ_max | 1.50 ± 0.10 | normalized | Model assumption |
| **Tissue-Specific Parameters (Intestinal Crypt)** |
| 13 | Division rate (baseline) | ν_0 | 52 ± 10 | divisions/year | Scaled from murine/human intestinal stem cell data |
| 14 | Stem cell pool size (initial) | N_s,0 | 1000 ± 200 | cells | Human intestinal crypt estimate |
| 15 | Critical damage threshold | D_crit | 12.5 ± 2.5 | damage units | Tissue dysfunction onset |
| 16 | Damage–division coupling | β_div | 0.05 ± 0.01 | damage⁻¹ | MCMC-fitted |
| **Tissue-Specific Parameters (Hematopoietic)** |
| 17 | Division rate (baseline) | ν_0_HSC | 12 ± 3 | divisions/year | Human HSC turnover |
| 18 | Stem cell pool size (initial) | N_s,0_HSC | 20000 ± 5000 | cells | Human bone marrow estimate |
| 19 | Critical damage threshold | D_crit_HSC | 10.0 ± 2.0 | damage units | MCMC-fitted |
| **Tissue-Specific Parameters (Muscle)** |
| 20 | Division rate (baseline) | ν_0_muscle | 3 ± 1 | divisions/year | Satellite cell turnover |
| 21 | Stem cell pool size (initial) | N_s,0_muscle | 5000 ± 1000 | cells | Human muscle estimate |
| 22 | Critical damage threshold | D_crit_muscle | 14.0 ± 3.0 | damage units | Higher tolerance |
| 23 | Mechanotransduction coefficient | η | 0.03 ± 0.01 | MPa⁻¹ | Dupont (2016) |
| **Tissue-Specific Parameters (Neuronal – non-proliferative)** |
| 24 | Division rate (baseline) | ν_0_neuron | 0.02 ± 0.01 | divisions/year | Negligible adult neurogenesis |
| 25 | Stem cell pool size (initial) | N_s,0_neuron | N/A | cells | Not applicable |
| 26 | Critical damage threshold | D_crit_neuron | 8.0 ± 2.0 | damage units | Lower tolerance |
| **CHIP Dynamics Parameters** |
| 27 | VAF at age 70 years | VAF_70 | 0.07 ± 0.01 | unitless | Jaiswal & Ebert (2019) |
| 28 | CHIP expansion exponent | δ | 3.1 ± 0.4 | unitless | MCMC-fitted |
| **Circadian Regulation Parameters** |
| 29 | Circadian amplitude decay rate | γ | 0.0021 ± 0.0005 | year⁻¹ | MCMC-fitted, constrained by Hood & Amir (2017) |
| 30 | Initial circadian amplitude | A₀ | 1.0 (fixed) | unitless | Model assumption |
| **General Simulation Parameters** |
| 31 | Maturation age (simulation start) | t₀ | 20 | years | Defined start of post-development aging |
| 32 | Simulation time step | Δt | 0.01 | years | Numerical integration stability |

*Note:* Parameters indicated as "MCMC-fitted" were inferred during calibration against the synthetic aging cohort. Their values are biologically plausible but require future calibration against longitudinal biological data. Parameters indicated with literature sources are directly anchored to empirical studies. All tissue-specific parameters are provided as representative examples; the full model implementation includes additional tissue types.

---