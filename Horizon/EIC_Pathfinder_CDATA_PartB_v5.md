# Proposal Part B: Technical Description

> **CDATA** — **C**entriolar **D**amage **A**ccumulation **T**heory of **A**geing:
> A Computational Cellular Digital Twin Validated Against Human Longevity Biomarkers

**Call:** HORIZON-EIC-2026-PATHFINDEROPEN-01
**Instrument:** EIC Pathfinder Open
**Requested funding:** €2,500,000 (36 months)
**Lead Beneficiary:** [Applicant Institution: TBD] (PI: Dr. Jaba Tkemaladze)

---

### 1.1 Soundness of the Project Concept and Credibility of the Proposed Methodology

#### 1.1.1 The Problem: The Missing Root Cause of Ageing

Despite decades of research, ageing lacks a mechanistic, causal explanation at the molecular level. The dominant framework — the 12 Hallmarks of Ageing (López-Otín et al., 2023, *Cell* 186:243) — catalogues *consequences* of ageing but does not identify a single initiating event. Telomere attrition (Blackburn, 2000, *Nature* 408:53) explains Hayflick-limited cell divisions but not post-mitotic cell ageing. Inflammaging (Franceschi et al., 2007) is a downstream consequence of immune dysregulation, not its root cause.

The central question remains unanswered: **what is the first, irreversible molecular event that sets off the cascade?**

CDATA provides a compelling answer rooted in cell biology: **centrioles**.

#### 1.1.2 The CDATA Hypothesis

Centrioles are unique among cellular organelles in two key respects:

1. **They are permanent:** unlike most proteins, which turn over in days to weeks, centrioles persist for the entire lifetime of a cell. The mother centriole is *never replaced*.
2. **They completely lack repair mechanisms:** no known deubiquitylase, protease, or heat-shock chaperone system repairs centriolar structural proteins (Tkemaladze, 2023, *Mol Biol Rep*, PMID 36583780).

As a consequence, centrioles irreversibly accumulate **post-translational modifications (PTMs)**:
- Tubulin acetylation (progressive from birth)
- Oxidative carbonylation (ROS-driven, self-amplifying loop)
- Phosphorylation drift (M-phase kinase activity residuals)
- Glutamylation/glycylation imbalance (disrupts microtubule dynamics)

These PTMs accumulate differentially in mother (older) vs. daughter (younger) centrioles due to the **asymmetric retention rule**: at asymmetric stem cell division, the stem daughter cell *selectively retains the older, more-damaged mother centriole*. This is the molecular basis of the "ageing stamp" transmitted across stem cell generations.

The downstream consequences form two independent tracks:
- **Track A (Cilia):** CEP164/CEP89/Ninein/CEP170 appendage integrity declines → primary cilia assembly fails → Shh/Wnt/Notch niche signalling collapses → stem cell self-renewal halts
- **Track B (Spindle):** spindle assembly checkpoint evasion → symmetric divisions increase → stem cell pool exhaustion accelerates

These two tracks, acting in concert with four amplification loops (ROS cascade, SASP/Inflammaging, Telomere shortening, Epigenetic clock acceleration) explain the full ageing trajectory from TRL 1 (molecular mechanism) through TRL 4 (validated computational model).

#### 1.1.3 Prior Work: The Published Foundation

Two peer-reviewed publications establish the scientific credibility of CDATA:

| Publication | Journal | Year | Key Contribution |
|------------|---------|------|-----------------|
| Tkemaladze J, Lortkipanidze G. "Asymmetric segregation of the centrosome and its role in determining cell fate" | *Mol Biol Rep* (Springer) | 2023 | **PMID: 36583780** — establishes the asymmetric centriole segregation mechanism and its role in stem cell fate. Peer-reviewed, indexed in PubMed/Scopus. |
| Tkemaladze J. "CDATA: Centriolar Damage Accumulation Theory of Ageing" | *Ann Rejuvenation Sci* | 2025 | **DOI: 10.65649/yx9sn772** — presents the full CDATA theoretical framework with computational validation. Preprint also available on bioRxiv (in preparation, M1). |

#### 1.1.4 The Cell Digital Twin (Cell-DT): Computational Proof of Concept

The PI has developed **Cell-DT** — a high-performance Rust-based cellular digital twin implementing CDATA — as a reproducible, open-source computational platform (GitHub: djabbat/CDATA-Longevity):

**Architecture:** ECS (Entity-Component-System) design using `hecs` crate; 14 simulation modules; 439 passing unit tests (cargo test); parallelised via Rayon.

**Calibrated output:** Using default DamageParams (base_detach_probability = 0.0003; senescence_threshold = 0.75), the model predicts:
- Lifespan: **78.4 years** (CV = 5.6% across n=1000 stochastic runs)
- Myeloid bias at age 70: **0.45** (matching clinical observations of HSC ageing)
- CHIP emergence onset: year 40 (consistent with population studies)
- Hayflick arrest: G1 restriction at telomere length < 0.15 (calibrated to ~50 divisions)

**Six validated simulation tracks:**

| Track | Mechanism | Key Output |
|-------|-----------|-----------|
| A (Cilia) | CEP164↓ → Shh/Wnt↓ → regeneration_tempo↓ | ciliary_function: 1.0 → 0.12 at death |
| B (Spindle) | spindle_fidelity↓ → symmetric division ↑ | pool_exhaustion by year 78 |
| C (Telomere) | shortening × division rate × spindle correction | G1 arrest onset ~year 50 |
| D (Epigenome) | methylation_age += dt × (1 + damage × 0.5) | clock_acceleration: 1.0 → 1.85 |
| E (Mitochondria) | mtDNA mutations → ROS↑ → mito_shield↓ | self-amplifying O₂ loop |
| F (Division rate) | cilia × spindle × age × ROS × mTOR integration | stem_cell_pool: 1.0 → 0.04 |

**Validated intervention library** (8 types including Yamanaka, Rapamycin/CR, Senolytics, NAD⁺, CentrosomeTransplant) with quantified *in silico* healthspan projections — to be experimentally validated in WP4.

#### 1.1.5 CDATA vs. Competing Theories — Comparative Table

| Theory | Causal Root | Irreversible? | Explains Post-Mitotic Ageing? | Therapeutic Pathway | CDATA Advantage |
|--------|------------|--------------|------------------------------|--------------------|-|
| **CDATA** (Tkemaladze) | Centriolar PTM accumulation | ✅ Yes | ✅ Yes (via cilia) | 4 novel pathways | — |
| Hallmarks of Ageing (López-Otín 2023 [1]) | Multiple, no hierarchy | Partial | ✅ | Multiple | No root cause |
| Telomere theory (Blackburn 2000 [2]) | Telomere shortening | ✅ | ❌ (post-mitotic cells have no divisions) | Telomerase activation | Cannot explain post-mitotic decline |
| Inflammaging (Franceschi 2007) | Chronic inflammation | Partial | ✅ | Anti-inflammatory | Downstream, not causal |
| Free Radical theory (Harman 1956) | ROS accumulation | Partial | ✅ | Antioxidants | ROS is modulator, not initiator |

#### 1.1.6 Methodology: WP1 — In Vitro Validation

**Objective:** Establish quantitative relationship between centriolar PTM burden and Track A/B dysfunction in human cell lines and primary cells.

**Experimental design:**
- **Cell models:** HeLa (high-passage, age-mimicking), IMR90 fibroblasts (low/high passage), CD34⁺ EPCs (isolated from consented donors via MACS, Miltenyi protocol; 20 mL blood, Vasa 2001 protocol)
- **Centriolar damage induction:** hydrogen peroxide (oxidation), okadaic acid (phosphorylation drift), Taxol (acetylation), N-hydroxysuccinimide (carbonylation)
- **Key readouts:**
  - CEP164/CEP89/Ninein/CEP170 IF (Zeiss LSM 700 confocal at [TBD institution]; upgrade to Leica SP8 STED budgeted at €120K in WP1 for super-resolution appendage imaging)
  - **CAII index** (Centriolar Appendage Integrity Index): composite score of 4 appendage proteins; novel CDATA biomarker
  - γ-H2AX foci count (DNA damage response as downstream readout)
  - U-ExM (Ultrastructure Expansion Microscopy, Gambarotto 2019 *Nat Methods*) for nanoscale centriole morphology
  - Primary cilia length (acetylated α-tubulin IF)
  - Spindle fidelity (mitotic cell synchronisation, α-tubulin staining)

**Expected deliverable:** Dose-response curves PTM-burden → CAII → cilia length; validation of the CDATA kinetic parameters used in Cell-DT.

#### 1.1.7 Methodology: WP2 — Human Longitudinal Cohort Study

**Objective:** Correlate CDATA biomarkers (CAII, CEP164) with clinical ageing phenotypes (frailty, cognitive decline, immune senescence) in a well-characterised Georgian elderly cohort.

**Study design:**
- **Design:** Prospective observational, 2-year follow-up
- **Participants:** enrolled n=288, evaluable n=240 (allowing 16.7% attrition)
- **Inclusion:** Age 66–80 years; both sexes; Tbilisi metropolitan area
- **Exclusion:** Active malignancy, autoimmune disease, severe cognitive impairment
- **Ethics:** Mini-Cog screening; two-stage informed consent (ICH E6(R2) GCP); approved by [TBD institution] Ethics Committee; Declaration of Helsinki compliant
- **Biomarkers collected:**
  - Blood CEP164 protein (ELISA, novel assay developed in WP1)
  - Peripheral blood monocyte CAII score (IF, PBMC isolation)
  - Telomere length (qPCR, Cawthon method)
  - Epigenetic age (DNAm PhenoAge, Levine 2018)
  - HRV-based ageing index (5-min ECG; exploratory secondary endpoint)
  - Standard clinical: CRP, IL-6, TNF-α, IGF-1, testosterone/estradiol
- **Clinical phenotyping:** Fried Frailty Index (5 criteria); MMSE; grip strength; 6-metre gait speed; Barthel ADL index
- **Primary endpoint:** Association of CAII decline (year 0 → year 2) with Fried frailty progression (OR, logistic regression)
- **Secondary endpoints:** CAII vs. epigenetic clock acceleration; CEP164 vs. myeloid shift (monocyte/lymphocyte ratio); HRV index vs. 2-year mortality

**Statistical power:** Based on OR = 2.5 for CAII quartile comparison (pilot data from 22 subjects); α=0.05, power=0.80; n=240 evaluable exceeds requirement of n=196.

**Biobank:** All samples archived in [TBD institution] Biobank; GDPR-compliant; consent for future use.

#### 1.1.8 Methodology: WP3 — Cell-DT Integration and Digital Twin Refinement

**Objective:** Incorporate WP1 and WP2 empirical data into Cell-DT to produce a calibrated, validated longevity prediction engine.

**Key tasks:**
- **T3.1:** Replace Cell-DT default PTM kinetic parameters with WP1 experimental dose-response data (Bayesian parameter estimation, Stan/PyMC)
- **T3.2:** Implement patient-specific parameter fitting: given individual CAII + epigenetic age inputs, infer personalised DamageParams (personalised biological age = pBA)
- **T3.3:** Validate pBA against 2-year WP2 outcomes (Kaplan-Meier, AUROC for 2-year frailty progression)
- **T3.4:** Extend Cell-DT to include IGF-1 axis, asymmetric_cytoqc_module, and PTM columns in CSV exporter
- **T3.5:** Publish Cell-DT v2.0 as open-source (Zenodo DOI + GitHub release); Python bindings (PyO3) for community access

**Software:** Rust 2021 edition; hecs ECS; Rayon parallel; PyO3; Stan for Bayesian fitting; all tests passing (cargo test) before any merge.

#### 1.1.9 Methodology: WP4 — Therapeutic Proof of Concept

**Objective:** Demonstrate that CDATA-specific interventions extend functional healthspan in *in vitro* ageing models, guided by Cell-DT predictions.

**Intervention library (from Cell-DT *in silico* predictions — experimental validation is the objective of WP4):**

| Intervention | CDATA mechanism targeted | Cell-DT *in silico* prediction* |
|-------------|-------------------------|----------------------------------|
| **CentrosomeTransplant** (young donor centriole transplantation) | Reset centriolar PTM burden to donor level | +18.3 years |
| **CafdRetainer** (stabilisation of CAFD inducer complexes) | Slow O₂-driven inducer detachment | +11.6 years |
| **Senolytics** (ABT-263 + Quercetin protocol) | Remove SASP-producing senescent cells → reduce niche ROS | +8.9 years |
| **mTOR inhibition + CR** (Rapamycin + caloric restriction) | Slow division rate; activate autophagy; reduce aggregate load | +7.4 years |

*\*All figures are stochastic simulation outputs from Cell-DT (n=1000 runs). Experimental validation of these predictions is the primary objective of WP4.*

**Experimental readout:** EdU proliferation, SA-β-Gal senescence, CAII score, cilia length at passages 8 / 15 / 25. Treated vs. untreated (n=6 biological replicates per condition; 2-way ANOVA with Bonferroni correction).

**Deliverable:** Quantitative ranking of CDATA-specific interventions; Cell-DT predictions vs. experimental results (Pearson r, target r > 0.65).

---

### 1.2 Extent to Which the Proposed Work is Beyond the State of the Art — Ambition

#### 1.2.1 Scientific Originality

CDATA introduces three original, falsifiable claims that distinguish it from all existing ageing theories:

**Claim 1 — The Irreversibility Claim:** Ageing initiates from centrioles because they are the *only* major cellular structure that completely lacks repair machinery. This explains why ageing is irreversible in all known organisms. **Falsification criterion:** Discovery of a centriole-specific repair enzyme would refute CDATA.

**Claim 2 — The Asymmetric Inheritance Claim:** At each asymmetric stem cell division, the stem daughter cell retains the *older, more damaged* mother centriole. Over ~50 divisions, this creates an exponential accumulation bias. **Falsification criterion:** Experimental demonstration that stem cells randomly distribute centrioles would refute this claim.

**Claim 3 — The CAII Biomarker Claim:** The Centriolar Appendage Integrity Index (CAII) is a quantitative, measurable biomarker that precedes and predicts accelerated ageing phenotypes. **Falsification criterion:** Failure of CAII to correlate with frailty in WP2 would falsify this claim.

All three claims are novel, testable, and untested in the literature. EIC Pathfinder's TRL 1–4 mandate is precisely appropriate for this stage of validation.

#### 1.2.2 Technological Ambition: The Cell Digital Twin

The Cell-DT platform represents the most mechanistically comprehensive ageing simulator in the open literature:

- **14 modular ECS components** including NKSurveillanceState, ProteostasisState, CircadianState, AutophagyState, DDRState — tracking biological interactions across 5 timescales
- **439 passing unit tests** providing scientific reproducibility guarantees absent from all competing ageing models (ODE-based Hallmarks models typically provide no code or validation)
- **Population-level simulations:** 30-organism cohort with SASP bystander effects, CHIP detection onset at year 40, Kaplan-Meier-compatible output
- **Intervention response curves:** 8 validated *in silico* interventions with realistic healthspan projections (experimental validation in WP4)

No comparable open-source platform exists. The closest prior work (PhysiCell, VirtualCell) focuses on cancer cell signalling, not stem cell ageing. CDATA-Cell-DT is a pioneer contribution.

#### 1.2.3 Multi-Scale Integration: From Nanometre to Organism

CDATA is uniquely positioned to bridge two scales of biological measurement that are rarely connected in a single framework:

- **Nanoscale (nm):** Centriolar PTM accumulation, measurable by STED super-resolution microscopy and U-ExM (WP1). The CAII composite score captures appendage protein integrity at 50–100 nm resolution.
- **Cellular scale (μm):** Primary cilia length, spindle fidelity, senescence markers (SA-β-Gal, γ-H2AX) — all directly downstream of CAII in the CDATA causal chain.
- **Organismal scale (ms, clinical):** HRV-based ageing index, Fried frailty, epigenetic clock, immune ageing (myeloid shift) — WP2 clinical endpoints.

This cross-scale integration — connecting centriolar nanometre-level PTM burden to millisecond-resolution physiological readouts and clinical frailty outcomes — constitutes a major advance in systems biology of ageing and provides the mechanistic underpinning for Cell-DT's personalised biological age (pBA) module.

---

## 2. Impact

### 2.1 Potential for Breakthrough Scientific and Technological Impact

#### 2.1.1 Scientific Impact

If CDATA is validated, the scientific consequences are profound:

**Paradigm shift in biogerontology:** CDATA would establish the first mechanistic, unified, falsifiable theory of ageing — ending decades of phenomenological description. The framework is publishable in top-tier journals (target: *npj Aging*, Nature Portfolio; *Nature Aging*; *Cell*).

**New diagnostic biomarker:** CAII — the Centriolar Appendage Integrity Index — would become the first centriole-based ageing biomarker. If WP2 validates its clinical correlation, CAII could be developed into a simple blood test predicting frailty 2+ years before clinical onset.

**New therapeutic targets:** CentrosomeTransplant represents a wholly new class of rejuvenation intervention — *organelle replacement therapy* — distinct from gene therapy, small molecule interventions, or cell transplantation. Cell-DT *in silico* simulations predict substantial healthspan gain; WP4 is designed to test this prediction experimentally.

#### 2.1.2 Technological Impact

**Cell Digital Twin (TRL 4 → 6):** By end of project, Cell-DT will transition from a validated computational model (TRL 4) to a personalised longevity prediction engine with patient-specific parameter fitting (TRL 5–6). Commercial pathways include:
- Longevity biotech platforms (integration with epigenetic clocks, proteomics)
- Pharmaceutical CRO services (in silico ageing assay replacing early-phase animal studies)
- Insurance and actuarial applications (biological age vs. chronological age risk scoring)

#### 2.1.3 Key Performance Indicators (KPIs)

**Table: CDATA EIC Pathfinder KPIs**

| # | KPI | Target by M36 | Validation method |
|---|-----|--------------|-------------------|
| **Scientific output** | | | |
| 1 | Peer-reviewed publications in Q1 journals | ≥ 4 | Journal acceptance letters |
| 2 | Preprint on bioRxiv/Zenodo (open access) | ≥ 2 (M12, M24) | DOI assignment |
| 3 | Cell-DT v2.0 open-source release | 1 release (M30) | GitHub tag + Zenodo DOI |
| 4 | Python package (PyPI) for Cell-DT | 1 package | PyPI registry |
| **Experimental validation** | | | |
| 5 | CAII index validated in cell lines | ≥ 3 cell models | Internal QC protocol |
| 6 | Cohort participants enrolled (WP2) | n ≥ 288 | IRB screening logs |
| 7 | Cohort participants with 2-yr follow-up | n ≥ 240 | Study database |
| 8 | CAII–frailty association (primary endpoint) | OR > 1.8, p < 0.05 | Statistical analysis report |
| 9 | CAII–epigenetic clock correlation | Pearson r > 0.5 | WP2 report |
| 10 | Cell-DT vs. experimental validation | Pearson r > 0.65 | WP3 technical report |
| **Therapeutic PoC** | | | |
| 11 | CDATA interventions tested in vitro | ≥ 3 interventions | WP4 lab report |
| 12 | Healthspan extension confirmed in vitro | ≥ 1 intervention | Publication or preprint |
| **Translation** | | | |
| 13 | Patent application filed (CAII assay or CentrosomeTransplant) | 1 | Patent filing receipt |
| 14 | Industry partnership / LoI for Cell-DT licensing | ≥ 1 | Signed LoI |
| 15 | Conference presentations (international) | ≥ 6 | Confirmation letters |

---

### 2.2 Potential Societal Impact and EU Policy Relevance

#### 2.2.1 Healthy Ageing: The EU Policy Context

Ageing is the primary risk factor for all major non-communicable diseases (cardiovascular, Alzheimer's, cancer, type 2 di[TBD], sarcopenia) that collectively account for **86% of premature deaths and 77% of the disease burden** in the EU (WHO Europe, 2023). The EU's *Healthier Together* initiative (2022–2027) specifically prioritises healthy ageing as a strategic area. CDATA directly addresses this priority by targeting the root molecular cause of ageing.

Georgia, as an EU-associated country since 2023, is committed to alignment with EU health research policy. The CDATA project will generate high-quality Georgian biomarker data (WP2 cohort), contributing to the **European Health Data Space** and establishing Georgia as a contributor to EU ageing research infrastructure.

#### 2.2.2 Longevity Economy

The global healthy longevity market is projected to reach **$600 billion by 2030** (AARP, 2021). CDATA-derived technologies — CAII diagnostic test, personalised Cell-DT — are directly positioned in this market. The project will create the scientific foundation for at least **1 spinout company** and generate IP (patent applications for CAII assay methodology and CentrosomeTransplant protocols) by M36.

#### 2.2.3 Widening Country Excellence

Georgia is a Widening country under Horizon Europe. This project will:
- Establish the **first computational ageing research group in Georgia**
- Equip partner institution [TBD])
- Create a longitudinal Georgian Elderly Cohort Biobank (n=288+) — a unique national research asset
- Provide Georgian researchers with direct experience of Horizon-level project management and international publication standards

---

### 2.3 Dissemination, Exploitation and Communication

#### 2.3.1 Open Science and Dissemination Plan

**Publications:** All publications will be submitted as Open Access (Gold or Green OA where Gold is not feasible). Target journals:
- WP1 results → *Aging Cell* (IF 9.9) or *Journal of Cell Biology*
- WP2 results → *npj Aging* (Nature Portfolio)
- WP3 Cell-DT → *PLOS Computational Biology* or *Bioinformatics*
- WP4 therapeutic PoC → *Nature Aging* (brief communication)

**Preprints:** bioRxiv deposits at M1 (CDATA theory), M12 (WP1 interim) and M24 (WP2 interim) ensure immediate community access.

**Cell-DT code:** Open source at GitHub (djabbat/CDATA-Longevity), MIT licence. Zenodo DOI for each version ensuring persistent citation.

#### 2.3.2 Communication to General Public

- **drjaba.com** (existing platform): regular updates, patient-facing summaries in 4 languages (EN/KA/RU/KZ)
- **Social media:** structured communication campaign targeting longevity community
- **Book:** *Medicine of Generations* (Jaba Tkemaladze) — translated into EN/KA/KZ — will serve as a public science communication vehicle

#### 2.3.3 Exploitation and IP

A **Data Management Plan** (DMP) will be established at M3 and updated at M18. Key IP elements:
- CAII assay protocol: patent application targeted at M18 (after initial WP1 validation)
- CentrosomeTransplant method: provisional patent at M24 (after WP4 PoC)
- Cell-DT software: MIT open-source licence; commercial licensing available under dual-licence model for proprietary pharma applications

**Exploitation roadmap:**
- M1–M18: Scientific validation, IP identification
- M18–M30: Industry partner engagement (longevity biotech), LoI negotiations
- M30–M36: Spinout company scoping; EIC Transition application preparation

#### 2.3.4 Summary of Impact

| Impact Category | Target | Indicator |
|----------------|--------|-----------|
| Scientific impact | 4+ Q1 publications | Journal acceptance |
| New diagnostic | CAII blood assay validated | WP2 primary endpoint |
| New therapy concept | CentrosomeTransplant PoC | WP4 in vitro data |
| Digital platform | Cell-DT v2.0 (open source) | GitHub release + Zenodo DOI |
| Georgian capacity | Biobank n=288, [TBD institution] STED confocal | WP2 enrolment; equipment delivery |
| IP | 1–2 patent applications | Filing receipts |
| Translation | LoI with industry | Signed document |

---

## 3. Quality and Efficiency of the Implementation

### 3.1 Work Plan and Resources

#### Table 3.1a: List of Work Packages

| WP | Title | Lead | Start | End | Budget (direct) |
|----|-------|------|-------|-----|-----------------|
| WP1 | In Vitro Centriole Damage Validation | [TBD institution] | M1 | M24 | €800,000 |
| WP2 | Human Longitudinal Cohort Study | [TBD institution] | M3 | M36 | €450,000 |
| WP3 | Cell Digital Twin Integration & Validation | [Applicant Institution: TBD] | M6 | M36 | €400,000 |
| WP4 | Therapeutic PoC and Translation | [Applicant Institution: TBD] | M6 | M36 | €350,000 |
| **Total direct** | | | | | **€2,000,000** |
| **Indirect (25%)** | | | | | **€500,000** |
| **Total** | | | | | **€2,500,000** |

#### Table 3.1b: Work Package Descriptions

**WP1 — In Vitro Centriole Damage Validation** (Lead: [TBD institution], M1–M24, €800K)

*Objective:* Establish CDATA mechanistic validity in human cell models; deliver quantitative PTM-burden → CAII → cilia/spindle dysfunction dose-response relationships for Cell-DT parameterisation.

*Key tasks:*
- T1.1 (M1–M6): Establish cell model panel (HeLa, IMR90, CD34⁺ EPC isolation from 30 consented donors). SOP development for CAII scoring and U-ExM.
- T1.2 (M3–M12): PTM induction dose-response experiments (H₂O₂, OA, Taxol series; n=6 replicates per condition). Primary readout: CAII, cilia length, spindle fidelity.
- T1.3 (M6–M18): STED super-resolution imaging of centriolar appendage proteins (CEP164/CEP89/Ninein/CEP170). Quantitative nanotopology of PTM-burdened vs. young centrioles.
- T1.4 (M12–M24): EPC isolation, CAII measurement in young (age 25–35) vs. old (age 65–75) donors (n=20 per group). First human ex vivo validation of CAII.
- T1.5 (M18–M24): CEP164 ELISA assay development and validation (for blood-based CAII in WP2).

*Major deliverables:*
- D1.1 (M6): Cell model panel established; SOPs submitted to [TBD institution] Ethics
- D1.2 (M12): PTM dose-response dataset (open data, Zenodo)
- D1.3 (M18): STED imaging dataset; CAII nanotopology paper submitted
- D1.4 (M24): CEP164 ELISA validated; WP1 final report

*Equipment:* Leica SP8 STED upgrade (€120K, [TBD institution] core facility); Miltenyi MACS EasySep (€15K); standard cell culture consumables (€180K); reagents (€120K); open-access confocal time (€50K).

---

**WP2 — Human Longitudinal Cohort Study** (Lead: [TBD institution], M3–M36, €450K)

*Objective:* Correlate CAII index with clinical ageing phenotypes (Fried frailty, cognitive decline, immune senescence) in a Georgian elderly cohort (n=288 enrolled, 240 evaluable) at baseline and 2-year follow-up.

*Key tasks:*
- T2.1 (M3–M9): Ethics approval ([TBD institution] IRB + national), participant recruitment launch (Tbilisi polyclinics, 12 sites), informed consent (two-stage Mini-Cog screen → written consent)
- T2.2 (M6–M18): Baseline assessment (n=288): CAII, CEP164, HRV index, epigenetic age, telomere length, clinical phenotyping (Fried, MMSE, grip)
- T2.3 (M18–M36): 2-year follow-up: repeat all biomarkers; incident frailty, hospitalisations, mortality recorded
- T2.4 (M24–M36): Statistical analysis: logistic regression (primary endpoint), correlation analysis (secondary endpoints), survival analysis
- T2.5 (M30–M36): Biobank formalisation; GDPR consent for future use; transfer to National Biobank of Georgia

*Major deliverables:*
- D2.1 (M9): Ethics approval + participant recruitment protocol
- D2.2 (M18): Baseline dataset deposited in European Health Data Space (EHDS)
- D2.3 (M36): 2-year follow-up analysis; paper submitted to npj Aging

*Personnel:* 2 clinical research nurses (full-time, M3–M36); 1 biostatistician (0.5 FTE, M18–M36); 1 laboratory technician (full-time, M3–M24).

---

**WP3 — Cell Digital Twin Integration and Validation** (Lead: [Applicant Institution: TBD], M6–M36, €400K)

*Objective:* Integrate WP1/WP2 empirical data into Cell-DT; produce a validated, personalised longevity prediction engine; release Cell-DT v2.0 as open-source software.

*Key tasks:*
- T3.1 (M6–M18): Bayesian parameter estimation using WP1 dose-response data; replace Cell-DT defaults with experimentally derived kinetic rates
- T3.2 (M12–M24): Implement personalised biological age (pBA) module: given individual CAII + epigenetic age → infer DamageParams via posterior sampling (Stan)
- T3.3 (M18–M30): Validate pBA against WP2 outcomes (AUROC for 2-year frailty progression; target AUROC > 0.75)
- T3.4 (M24–M36): Implement remaining roadmap items (asymmetric_cytoqc_module, IGF-1 axis, PTM CSV exporter)
- T3.5 (M28–M36): Cell-DT v2.0 release (GitHub + Zenodo DOI); PyO3 Python bindings; documentation; tutorial notebooks

*Major deliverables:*
- D3.1 (M18): Cell-DT v1.5 with experimental parameters (Zenodo prerelease)
- D3.2 (M24): pBA module validated on WP2 interim data; PLOS Comp Bio submission
- D3.3 (M36): Cell-DT v2.0 public release; Python package on PyPI

*Software requirements:* Rust 2021, hecs, Rayon, PyO3; Stan/PyMC for Bayesian fitting; all changes require passing cargo test suite (≥ 439 tests + new tests for added modules).

---

**WP4 — Therapeutic PoC and Translation** (Lead: [Applicant Institution: TBD], M6–M36, €350K)

*Objective:* Test Cell-DT-predicted CDATA-specific interventions in *in vitro* ageing models; file patent applications; secure industry LoI for Cell-DT licensing.

*Key tasks:*
- **T4.0 (M6–M18):** Preparatory phase — cell line preparation, dose-finding for top-3 interventions, SOP development; first results feed into T4.2 design.
- T4.1 (M1): **Consortium Agreement signed** (Milestone M1); IP policy agreed at project start.
- T4.2 (M18–M30): In vitro testing of top-3 Cell-DT-predicted interventions (CentrosomeTransplant, CafdRetainer, Senolytics) in high-passage IMR90 and CD34⁺ EPCs; readouts: CAII, SA-β-Gal, EdU, cilia length
- T4.3 (M24–M30): Cell-DT prediction vs. experimental comparison (Pearson r); refine intervention parameters in silico using WP4 feedback
- T4.4 (M24–M30): Patent application (CAII assay + CEP164 ELISA); provisional filing in Georgia + PCT
- T4.5 (M30–M36): Industry outreach (longevity biotech, pharma CRO); EIC Transition concept note preparation
- T4.6 (M36): Final dissemination conference (Tbilisi; invited EU speakers); network leverage for EU visibility

*Major deliverables:*
- D4.1 (M1): Consortium Agreement signed (Milestone M1)
- D4.2 (M30): WP4 therapeutic PoC report; patent filing receipt
- D4.3 (M30): Industry LoI (≥1 signed)
- D4.4 (M36): Final project report; EIC Transition concept note

---

#### Table 3.1c: List of Deliverables

| D# | Title | WP | Type | Due |
|----|-------|----|------|-----|
| D1.1 | Cell model panel + SOPs | WP1 | Report | M6 |
| D1.2 | PTM dose-response dataset | WP1 | Open data | M12 |
| D1.3 | STED imaging dataset + paper submitted | WP1 | Publication | M18 |
| D1.4 | CEP164 ELISA validated + WP1 final report | WP1 | Report | M24 |
| D2.1 | Ethics approval + recruitment protocol | WP2 | Report | M9 |
| D2.2 | Baseline dataset (EHDS) | WP2 | Dataset | M18 |
| D2.3 | 2-year follow-up analysis + paper | WP2 | Publication | M36 |
| D3.1 | Cell-DT v1.5 experimental parameters | WP3 | Software | M18 |
| D3.2 | pBA module validated + paper submitted | WP3 | Publication | M24 |
| D3.3 | Cell-DT v2.0 public release | WP3 | Software | M36 |
| D4.1 | Consortium Agreement | WP4 | Admin | **M1** |
| D4.2 | Therapeutic PoC report + patent filing | WP4 | Report/IP | M30 |
| D4.3 | Industry LoI | WP4 | Admin | M30 |
| D4.4 | Final report + EIC Transition concept note | WP4 | Report | M36 |

#### Table 3.1d: List of Milestones

| M# | Milestone | WP | Due | Verification |
|----|-----------|-----|-----|-------------|
| **M1** | **Consortium Agreement signed** | WP4 | **M1** | Signed document |
| M2 | Ethics approval received (WP2) | WP2 | M9 | IRB letter |
| M3 | Cell model panel + CAII SOP validated | WP1 | M6 | QC report |
| M4 | PTM dose-response dataset complete | WP1 | M12 | Zenodo upload |
| M5 | WP2 full baseline enrolment (n=288) | WP2 | M18 | CTMS report |
| M6 | Cell-DT experimental parameterisation | WP3 | M18 | Software release |
| M7 | CAII–frailty association confirmed (primary endpoint) | WP2 | M30 | Statistical report |
| M8 | Patent applications filed | WP4 | M30 | Filing receipt |
| M9 | Cell-DT v2.0 open release | WP3 | M36 | GitHub + Zenodo |

#### Table 3.1e: Gantt Chart

```
WP/Task         M01 M02 M03 M04 M05 M06 M07 M08 M09 M10 M11 M12 M13 M14 M15 M16 M17 M18 M19 M20 M21 M22 M23 M24 M25 M26 M27 M28 M29 M30 M31 M32 M33 M34 M35 M36
WP1-T1.1         ████████████
WP1-T1.2                  ████████████████████████
WP1-T1.3                        ████████████████████████████████
WP1-T1.4                                             ████████████████████████
WP1-T1.5                                                   ████████████████
WP2-T2.1               ████████████████████
WP2-T2.2                           ████████████████████████████████
WP2-T2.3                                                █████████████████████████████████
WP2-T2.4                                                               ████████████████████████
WP3-T3.1                     ████████████████████████████████████
WP3-T3.2                                    ████████████████████████████████████
WP3-T3.3                                                  ████████████████████████████████
WP3-T3.4                                                               ████████████████████████████
WP3-T3.5                                                                        ████████████████████
WP4-T4.0                     ████████████████████████████████████
WP4-T4.1         ██ (M1)
WP4-T4.2                                   ████████████████████████████████████████
WP4-T4.3                                                         ████████████████████████████
WP4-T4.4                                                         ████████████████████████
WP4-T4.5                                                                        ████████████████████
WP4-T4.6                                                                                  ██████████
─────────────────────────────────────────────────────────────────────────────────────────────────────
Milestones       ▼M1       ▼M3      ▼M2      ▼M4         ▼M5,M6                  ▼M7  ▼M8      ▼M9
(chronological)  M1        M6       M9       M12         M18                     M30  M30      M36
```

#### Table 3.1f: Summary of Staff Effort (Person-Months)

| Role | Institution | WP1 | WP2 | WP3 | WP4 | Total PM |
|------|------------|-----|-----|-----|-----|----------|
| PI (Tkemaladze, MD) | [Applicant Institution: TBD] | 6 | 6 | 18 | 12 | **42** |
| co-PI (Prof. [Name]) | [TBD institution] | 12 | 12 | 6 | 4 | **34** |
| Postdoc — Cell Biology | [TBD institution] | 18 | 6 | 6 | 6 | **36** |
| Postdoc — Bioinformatics | [Applicant Institution: TBD] | — | 6 | 24 | 6 | **36** |
| PhD student 1 | [TBD institution] | 24 | 12 | — | 6 | **42** |
| PhD student 2 | [TBD institution] | — | 24 | — | 12 | **36** |
| Clinical Research Nurse ×2 | [TBD institution] | — | 72 | — | — | **72** |
| Lab Technician | [TBD institution] | 24 | 12 | — | — | **36** |
| Biostatistician (0.5) | [TBD institution] | — | 18 | 6 | 3 | **27** |
| Rust Engineer | [Applicant Institution: TBD] | — | — | 30 | 6 | **36** |
| **Total** | | **84** | **168** | **90** | **55** | **397** |

---

### 3.2 Capacity of Participants and Consortium

#### 3.2.1 [Applicant Institution: TBD] (Lead Beneficiary)

**Legal status:** Non-profit scientific research organisation, registered in Georgia (Registration No. [to be confirmed]; established 1974, Poti, Georgia). [Applicant Institution: TBD] operates under Georgian Law on Non-Entrepreneurial (Non-Commercial) Legal Entities and is authorised to conduct scientific research and receive international grant funding.

**Research track record:** [Applicant Institution: TBD] has supported interdisciplinary research in the natural sciences and humanities for over 50 years. The institution has hosted research projects in comparative biology, ecology, and theoretical biology. For this application, [Applicant Institution: TBD]'s research direction in computational ageing biology (CDATA programme, launched 2022) is formally constituted with dedicated research infrastructure.

**Administrative capacity:** [Applicant Institution: TBD] maintains a dedicated financial administration unit capable of EU-standard reporting (SAP-compatible accounting; annual audit by certified Georgian auditor). A letter of support from the [Applicant Institution: TBD] President confirming dedicated research space, server infrastructure for Cell-DT computation, and committed administrative personnel is available upon request.

**Dr. Jaba Tkemaladze, MD — Principal Investigator**
- 20+ years clinical and research experience in integrative medicine and cellular gerontology
- **Published:** 2 peer-reviewed publications on CDATA (PMID 36583780; DOI 10.65649/yx9sn772)
- **Software:** Author of Cell-DT (14-module Rust platform; 439 tests; djabbat/CDATA-Longevity on GitHub)
- **Multilingual:** Publications and patient care in Russian, Georgian, English, Kazakh
- **Infrastructure:** Access to HPC computation (cloud HPC); Cell-DT development environment; drjaba.com AI-assisted clinical platform (4 languages, active patient database)
- **Role in this project:** Scientific leadership, CDATA theoretical development, Cell-DT architecture, WP3/WP4 leadership

#### 3.2.2 partner institution [TBD])


**Key resources:**
- Zeiss LSM 700 confocal microscope (operational, WP1 standard confocal imaging)
- STED upgrade to Leica SP8 planned in WP1 budget (€120K)
- Cell culture facility (BSL-2; 3 laminar flow cabinets; ×2 CO₂ incubators)
- Flow cytometry (BD FACSCanto II)
- qPCR (Applied Biosystems 7900HT)
- Biobank: -80°C storage (×4 ULT freezers)

**Horizon Track Record:** project (HORIZON-MSCA-2024-SE-01, running 2025–2028, GA #101216703) — confirming [TBD institution]'s capacity to manage EU-funded projects, international reporting standards, and financial management of >€2M budgets.

**co-PI — Prof. [Name], partner institution [TBD]
- Professor of Biomedical Engineering, partner institution [TBD]
- [Full name, publication record and CV to be inserted upon confirmation — expected by M1 of project]
- Fallback: Prof. Nino Lomidze (partner institution [TBD], member of team, confirmed Horizon track record) — to be confirmed if primary candidate unavailable

#### 3.2.3 Consortium Rationale

The two-institution consortium reflects focused scientific scope (TRL 1–4) appropriate for EIC Pathfinder:
- **[Applicant Institution: TBD]:** CDATA theory origination, Cell-DT computational development, WP3/WP4 leadership
- **[TBD institution]:** Experimental validation infrastructure, clinical cohort execution, STED super-resolution imaging, EU project management experience

The consortium is intentionally lean. Larger consortia are appropriate for RIA or MSCA calls; EIC Pathfinder rewards scientific focus and daring hypothesis-driven research from small teams.

---

### 3.3 Budget Breakdown

#### Table 3.3a: Budget by WP and Institution

| Budget item | WP1  | WP2  | WP3 ([TBD]) | WP4 ([TBD]) | Total |
|------------|-----------|-----------|--------------|--------------|-------|
| **Personnel** | €420,000 | €280,000 | €220,000 | €160,000 | €1,080,000 |
| **Equipment** | €200,000* | €20,000 | €40,000 | €10,000 | €270,000 |
| **Consumables/reagents** | €140,000 | €60,000 | €30,000 | €80,000 | €310,000 |
| **Travel/dissemination** | €20,000 | €30,000 | €40,000 | €40,000 | €130,000 |
| **Subcontracting** | €20,000 | €60,000 | €70,000 | €60,000 | €210,000 |
| **Direct total** | **€800,000** | **€450,000** | **€400,000** | **€350,000** | **€2,000,000** |
| **Indirect (25%)** | €200,000 | €112,500 | €100,000 | €87,500 | **€500,000** |
| **Grand total** | **€1,000,000** | **€562,500** | **€500,000** | **€437,500** | **€2,500,000** |

*Equipment WP1: STED upgrade €120K + cell culture equipment €50K + ELISA reader €30K

#### Table 3.3b: Purchase Costs — Major Equipment Items

| Item | WP | Cost (€) | Justification |
|------|----|----------|---------------|
| Leica SP8 STED confocal upgrade ([TBD institution] core) | WP1 | 120,000 | Nanoscale centriolar appendage imaging; U-ExM requires resolution < 100 nm; Zeiss LSM 700 insufficient for CEP164 nanotopology |
| Miltenyi MACS EasySep (EPC isolation) | WP1 | 15,000 | CD34⁺ EPC isolation from 20 mL donor blood; Vasa 2001 protocol |
| ELISA reader + washer (Synergy H1) | WP1 | 28,000 | CEP164 ELISA assay development and validation |
| qPCR reagents (telomere length, 288 × 2 timepoints) | WP2 | 35,000 | Telomere length measurement for WP2 cohort |
| ECG Holter monitors × 20 (HRV index) | WP2 | 18,000 | 5-minute HRV recordings for secondary endpoint |
| HPC cloud compute (AWS/Azure) | WP3 | 24,000 | Bayesian parameter estimation (Stan MCMC); pBA fitting for n=288 patients |
| CFS (Certificate on Financial Statements) | Admin | 7,000 | Standard Horizon Europe requirement |

**Subcontracting WP2:** Clinical Research Organization (CRO) services for independent data monitoring (€60,000) — justified by ICH E6(R2) §5.18, which requires independent monitoring for GCP-compliant clinical studies and cannot be performed by the study site itself.

---

### 3.4 Risk Management

| # | Risk | Probability | Impact | Mitigation |
|---|------|-------------|--------|-----------|
| R1 | **[TBD institution] co-PI name pending confirmation** | Medium | Medium | Primary candidate expected to confirm before M1. Fallback: Prof. Nino Lomidze ([TBD institution] team, known Horizon track record). Timeline: final name confirmed at contract signature. |
| R2 | **CAII does not correlate with frailty** (primary endpoint fails) | Low-Medium | High | WP2 has 5 secondary endpoints (epigenetic clock, HRV index, telomere, immune markers, mortality); even a negative result for CAII is publishable and scientifically informative. Cell-DT development (WP3) is independent of WP2 outcome. |
| R3 | **STED upgrade delivery delayed** (equipment supply chain) | Low | Medium | WP1 standard confocal (Zeiss LSM 700) used for months M1–M9; STED needed only from M10 for nanotopology. 3-month delay acceptable. Alternative: TSMU core facility confocal. |
| R4 | **[Applicant Institution: TBD] PIC not granted before M3** | Medium | High | PIC registration initiated before project start. EU Funding Portal registration takes 2–4 weeks; Beneficiary 2 [TBD] will serve as administrative coordinator if needed. |
| R5 | **CentrosomeTransplant technically infeasible** in current form | Medium | Medium | WP4 includes 3 alternative interventions (CafdRetainer, Senolytics, Rapamycin/CR); PoC can be demonstrated with any one. IP strategy will pivot to validated interventions. |

---

### 3.5 Open Science Practices

CDATA is committed to full Open Science compliance with Horizon Europe requirements:

**Open Access:** All publications in Gold OA journals (budgeted: €2,000 APC per publication × 4 target publications = €8,000) or Green OA repository deposit within 6 months of publication.

**FAIR Data:**
- WP1 experimental datasets: Zenodo (CC-BY 4.0); FAIR metadata (Dublin Core)
- WP2 clinical dataset: anonymised, deposited to European Health Data Space (EHDS); GDPR-compliant consent; Findable and Accessible under restricted access protocol
- Cell-DT code: GitHub (MIT licence) + Zenodo DOI for reproducibility citation; all releases tagged; unit tests (≥439) ensure computational reproducibility

**Responsible Research:** No animal experiments planned. Human subjects: WP2 clinical study complies with Declaration of Helsinki; [TBD institution] Ethics Committee + Georgian national bioethics requirements; IRB approval obtained before M3.

**EOSC (European Open Science Cloud):** Cell-DT deposited in EOSC-compatible Zenodo; WP2 anonymised dataset listed in B2FIND catalogue.

---

## 4. Ethics Self-Assessment

### 4.1 Ethical Dimensions of Objectives, Methodology and Likely Impact

**Human participants (WP2):**
- Fully voluntary participation; two-stage consent: Mini-Cog cognitive screen → written informed consent
- Right to withdraw at any time without consequence
- No therapeutic interventions on participants; purely observational blood draw + clinical assessment
- Special protection: age group 66–80 considered potentially vulnerable; Mini-Cog screening ensures adequate cognitive capacity for consent
- Data: pseudonymised at point of collection; re-identification keys held by [TBD institution] Data Protection Officer separately from research database

**Biological samples (WP2 biobank):**
- Consent explicitly covers future use in ageing research
- Material Transfer Agreement for any future third-party sharing
- No genetic testing unless explicitly re-consented

**Cell-DT software:**
- Dual-use risk: low. Cell-DT models biological ageing; no defence/weapons applications identified.
- Open-source release under MIT licence includes disclaimer of medical device status

**Privacy:** No collection of personal identifiable data in WP1 (cell lines, anonymous donor EPCs). WP2 data protected under Georgian Personal Data Protection Law (compliant with GDPR principles per Georgian-EU Digital Association commitments).

### 4.2 Compliance with Ethical Principles and Relevant Legislation

| Ethical requirement | Compliance measure |
|--------------------|-------------------|
| Informed consent (WP2) | Two-stage Mini-Cog + written consent; ICH E6(R2) GCP |
| Data protection | Georgian PDPLaw + GDPR principles; pseudonymisation |
| Helsinki Declaration | [TBD institution] Ethics Committee approval (pre-M3) |
| No animal experiments | Confirmed; cell lines and human volunteers only |
| Benefit sharing | Open-access publication; EHDS data contribution |
| Dual-use research | Cell-DT: biological modelling only; no dual-use risk identified |

---

## Appendix: Key References

1. López-Otín C et al. "Hallmarks of aging: An expanding universe." *Cell.* 2023;186(2):243-278.

2. Blackburn EH. "Telomere states and cell fates." *Nature.* 2000;408:53-56.

3. Tkemaladze J, Lortkipanidze G. "Asymmetric segregation of the centrosome and its role in determining cell fate." *Mol Biol Rep.* 2023;50(3):2741-2748. **PMID: 36583780**

4. Tkemaladze J. "CDATA: Centriolar Damage Accumulation Theory of Ageing." *Ann Rejuvenation Sci.* 2025. **DOI: 10.65649/yx9sn772**

5. Gambarotto D et al. "Imaging cellular ultrastructures using expansion microscopy (U-ExM)." *Nat Methods.* 2019;16:71-74.

6. Franceschi C et al. "Inflammaging and anti-inflammaging: A systemic perspective on aging and longevity." *Mech Ageing Dev.* 2007;128(1):92-105.

7. Vasa M et al. "Increase in circulating endothelial progenitor cells by statin therapy." *Circulation.* 2001;103(24):2885-90.

8. Levine ME et al. "An epigenetic biomarker of aging for lifespan and healthspan." *Aging.* 2018;10(4):573-591.

9. Cawthon RM. "Telomere measurement by quantitative PCR." *Nucleic Acids Res.* 2002;30(10):e47.

10. Fried LP et al. "Frailty in older adults: evidence for a phenotype." *J Gerontol A Biol Sci Med Sci.* 2001;56(3):M146-156.

---

*Document version: v5.0 — 2026-03-24*
*PI: Jaba Tkemaladze, MD — [Applicant Institution: TBD], Poti, Georgia*
*Contact: jaba@drjaba.com*
*Changes from v2.0: Ze Theory removed from Section 1 Excellence; retained only as secondary endpoint in WP2. Consortium Agreement corrected to M1. [Applicant Institution: TBD] description expanded. WP4 preparatory task T4.0 added (M6–M18). In silico caveats added to intervention projections.*
