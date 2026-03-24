# CDATA — Human Lifecycle Simulation

**Centriolar Damage Accumulation Theory of Aging**
Implementation on the Cell DT Platform (Rust)

Theory author: Jaba Tkemaladze (2005–2026)

---

## Table of Contents

1. [The CDATA Hypothesis](#1-the-cdata-hypothesis)
2. [Key Mechanisms](#2-key-mechanisms)
3. [Module Architecture](#3-module-architecture)
4. [Core Components](#4-core-components)
5. [human_development_module](#5-human_development_module)
6. [Developmental Stages](#6-developmental-stages)
7. [Parameters and Calibration](#7-parameters-and-calibration)
8. [Running the Simulation](#8-running-the-simulation)
9. [Results and Interpretation](#9-results-and-interpretation)
10. [Biological Hierarchy — 11 Levels](#10-biological-hierarchy--11-levels)
11. [References](#11-references)

---

## 1. The CDATA Hypothesis

**CDATA (Centriolar Damage Accumulation Theory of Aging)** is a theory that
explains organismal aging as an inevitable consequence of the cell
differentiation program.

The central claim: **the mother centriole of a stem cell is the only biological
structure that irreversibly accumulates molecular damage throughout the entire
lifespan of an organism**, because:

1. The centriole replicates by a template mechanism — the old (mother) centriole
   is never fully rebuilt from scratch.
2. During asymmetric division of a stem cell, the daughter that retains
   stem identity **always inherits the old mother centriole**.
3. This means all chronologically accumulated molecular damage remains inside
   the stem cell — while the replaced differentiated cells receive relatively
   "fresh" centrioles.

**The paradox of aging:** tissues are continuously renewed by stem cells, yet
the organism still ages — precisely because stem cells carry an ever-growing
burden of damage in their mother centrioles.

---

## 2. Key Mechanisms

### 2.1 The Differentiation Inducer System

According to the theory (Tkemaladze 2005/2023), the centriole contains two sets
of **irreversible differentiation inducers**:

| Structure | Function | Initial count |
|-----------|----------|---------------|
| **S-structure** | Somatic lineage | ~50 inducers (≈ Hayflick limit) |
| **H-structure** | Germline | ~4 inducers (until meiosis) |

At each **differentiation-inducing mitosis**, one S-inducer is released from the
centriole, enters the nucleus, switches off one gene network and activates
another — this is the irreversible change in morphogenetic status.

```
S_count = 50  →  totipotent (zygote)
S_count = 40  →  pluripotent (ICM of blastocyst)
S_count = 25  →  multipotent (adult stem cells)
S_count = 10  →  unipotent
S_count = 0   →  terminal differentiation / apoptosis
```

### 2.2 Accumulation of Molecular Damage

Five types of irreversible damage accumulate in the mother centriole:

```
STRESSORS (ROS, UV, metabolic by-products)
        │
        ▼
MOTHER CENTRIOLE — the non-renewable keeper of time
        │
        ├── Protein carbonylation (SAS-6, CEP135)
        │     → loss of structural integrity, PCM destabilisation
        │
        ├── α-tubulin hyperacetylation (↓ HDAC6, ↓ SIRT2)
        │     → reduced microtubule flexibility, impaired transport
        │
        ├── Protein aggregates (CPAP, CEP290)
        │     → block of the duplication machinery, PCM disorganisation
        │
        ├── Phosphorylation dysregulation (PLK4, NEK2, PP1)
        │     → ectopic amplification or failure of centriole disengagement
        │
        └── Loss of distal appendages (CEP164, CEP89, Ninein, CEP170)
              → inability to assemble the primary cilium
```

### 2.3 Two Parallel Pathways to Pathology

Once damage accumulates, the centriole loses two critical functions:

**Track A — ciliary failure ("centriolar blindness"):**
```
Loss of CEP164/CEP89 → no primary cilium
→ stem cell is deaf to Shh and Wnt niche signals
→ wrong cell-fate decisions
```

**Track B — failure of asymmetric cell division (ACD):**
```
PCM degradation → weak, misoriented mitotic spindle
→ random segregation of fate determinants (Numb, Prominin-1)
→ either pool exhaustion (both daughters differentiate)
→ or clonal hyperproliferation (both daughters remain stem cells)
```

### 2.4 The Positive Feedback Loop

```
Damaged centriole
    │
    ▼
Microtubule disorganisation
    │
    ▼
Impaired mitophagy → mitochondrial dysfunction
    │
    ▼
↑ ROS → more centriolar damage (self-amplifying cycle)
```

Additional loops:
- Mislocalisation of HDAC6/SIRT2 → global epigenetic alterations
- False DDR signal (BRCA1/53BP1 displaced) → p53/p21 → senescence

---

## 3. Module Architecture

```
cell_dt/
├── crates/
│   ├── cell_dt_core/
│   │   └── src/components.rs          # All ECS components (P21–P27 included)
│   └── cell_dt_modules/
│       └── human_development_module/
│           └── src/
│               ├── lib.rs             # Main module (SimulationModule) — step 3г–3и
│               ├── damage.rs          # Molecular damage + AppendageProteinState (P21)
│               ├── thermodynamics.rs  # Arrhenius model, ThermodynamicState (P22)
│               ├── ros_cascade.rs     # ROS ODE cascade O₂⁻→H₂O₂→OH·→Fe (P23)
│               ├── ze_health.rs       # Ze Vector Theory bridge, ZeHealthState (P24)
│               ├── microtubule.rs     # MT dynamics DII model, MicrotubuleState (P25)
│               ├── golgi.rs           # Golgi fragmentation → CEP164 glycosylation (P26)
│               ├── genetic.rs         # SNP-based DamageParams modifiers, GeneticProfile (P27)
│               ├── atpenergy.rs       # ATP/ADP energy charge, proteasome modifier (P28)
│               ├── chromatin.rs       # TAD integrity, heterochromatin, DNA accessibility (P29)
│               ├── ift.rs             # Intraflagellar transport, cargo delivery (P30)
│               ├── actin_ring.rs      # Contractile ring, cytokinesis fidelity (P31)
│               ├── er_stress.rs       # ER stress, UPR, Ca²⁺ buffering (P32)
│               ├── lysosome.rs        # pH, hydrolase activity, membrane permeability (P33)
│               ├── peroxisome.rs      # Catalase, H₂O₂ clearance, β-oxidation (P34)
│               ├── ribosome.rs        # Translation rate, RQC, aminoacyl-tRNA (P35)
│               ├── extracellular_matrix.rs  # Collagen crosslinking, stiffness, integrin (P36)
│               ├── vascular_niche.rs  # Angiogenesis, O₂ supply, growth factors (P37)
│               ├── fibrosis.rs        # Myofibroblast activation, functional replacement (P38)
│               ├── hpa_axis.rs        # Cortisol, HPA reactivity, chronic stress (P39)
│               ├── metabolic_phenotype.rs  # BMI, adipokines, insulin sensitivity (P40)
│               ├── organ.rs           # OrganState aggregation, poly-organ failure (P41)
│               ├── clone_epigenetic.rs  # Clone-specific epigenetic drift, CHIP (P42)
│               ├── fate_switching.rs  # Stochastic fate bias, Langevin noise (P43)
│               ├── inducers.rs        # M/D inducer system, O₂-detachment
│               ├── development.rs     # Developmental stages and rates
│               ├── tissues.rs         # 11 tissue types, TissueState
│               ├── aging.rs           # Aging phenotypes, senescence links
│               └── interventions.rs   # 8 therapeutic interventions (P11)
└── examples/
    └── src/bin/
        └── human_development_example.rs  # Full 100-year simulation
```

**Tests: 357+** (human_development_module: 273+; full workspace: 357+)
**Rule: before every git push — update README.md to reflect implemented components.**

---

## 4. Core Components

Added to `cell_dt_core::components`:

### `DevelopmentalStage`

Enum of developmental stages:

| Stage | Age | Notes |
|-------|-----|-------|
| `Zygote` | Day 0 | No centrioles, totipotency |
| `Cleavage` | Days 1–4 | De novo centriole formation |
| `Blastocyst` | Days 4–14 | ICM (pluripotent) vs trophoblast |
| `Gastrulation` | Days 14–28 | Three germ layers |
| `Organogenesis` | Days 28–56 | Organ formation |
| `Fetal` | 2–9 months | Fetal period |
| `Postnatal` | 0–18 years | Growth and development |
| `Adult` | 18–40 years | Tissue homeostasis |
| `MiddleAge` | 40–65 years | Accelerating damage |
| `Senescent` | 65+ years | Pronounced aging |
| `Death` | — | Organismal death |

### `CentriolarInducers`

```rust
pub struct CentriolarInducers {
    pub s_count: u32,              // Remaining S-inducers
    pub s_max:   u32,              // Initial stock
    pub h_count: u32,              // Remaining H-inducers
    pub h_max:   u32,
    pub differentiation_divisions: u32,  // Division counter
}
```

Key methods:
- `s_status() -> f32` — morphogenetic status [0 = totipotent, 1 = terminal]
- `consume_s_inducer()` — spend one S-inducer on differentiation
- `is_terminally_differentiated()` — returns true when S_count = 0

### `CentriolarDamageState`

```rust
pub struct CentriolarDamageState {
    // Molecular damage
    pub protein_carbonylation:         f32,  // SAS-6, CEP135
    pub tubulin_hyperacetylation:      f32,  // α-tubulin Lys40
    pub protein_aggregates:            f32,  // CPAP, CEP290
    pub phosphorylation_dysregulation: f32,  // PLK4, NEK2, PP1

    // Appendage integrity [0..1]
    pub cep164_integrity: f32,
    pub cep89_integrity:  f32,
    pub ninein_integrity: f32,
    pub cep170_integrity: f32,

    // Derived functional metrics
    pub ciliary_function: f32,  // Primary cilium function
    pub spindle_fidelity: f32,  // Mitotic spindle accuracy
    pub ros_level:        f32,  // ROS level (feedback loop)
    pub is_senescent:     bool,
}
```

### `TissueState`

Aggregated regenerative metrics for a tissue:

```rust
pub struct TissueState {
    pub tissue_type:         TissueType,
    pub stem_cell_pool:      f32,  // Stem cell pool size [0..1]
    pub regeneration_tempo:  f32,  // Regeneration rate [0..1]
    pub senescent_fraction:  f32,  // Fraction of senescent cells [0..1]
    pub functional_capacity: f32,  // Tissue functional capacity [0..1]
}
```

### `OrganismState`

Global organism-level metrics:

```rust
pub struct OrganismState {
    pub age_years:           f64,
    pub developmental_stage: DevelopmentalStage,
    pub inflammaging_score:  f32,  // Systemic inflammation [0..1]
    pub frailty_index:       f32,  // Frailty index [0..1]
    pub cognitive_index:     f32,  // Cognitive reserve [0..1]
    pub immune_reserve:      f32,  // Immune reserve [0..1]
    pub muscle_mass:         f32,  // Muscle mass (sarcopenia) [0..1]
    pub is_alive:            bool,
}
```

### `AppendageProteinState` — P21 (level −3: molecules)

Independent kinetics of 4 centriolar appendage proteins:

```rust
pub struct AppendageProteinState {
    pub cep164: f32,  // Distal (cilium initiation, IFT recruitment)  — OH· sensitivity 1.50
    pub cep89:  f32,  // Distal (cilium docking)                      — OH· sensitivity 1.00
    pub ninein: f32,  // Subdistal (MT minus-end anchoring)           — OH· sensitivity 0.75
    pub cep170: f32,  // Subdistal (MT elongation)                    — OH· sensitivity 0.55
    pub caii:   f32,  // CAII = cep164^0.40 × cep89^0.25 × ninein^0.20 × cep170^0.15
}
```

CAII (Centriolar Appendage Integrity Index) is the primary EIC WP1 biomarker measured by
U-ExM (n=288 donors, ages 20–80).

### `ThermodynamicState` — P22 (level −4: atoms)

Arrhenius amplification of damage rates by SASP-driven temperature:

```rust
pub struct ThermodynamicState {
    pub local_temp_celsius:       f32,  // 36.6 + sasp × 2.4 °C
    pub damage_rate_multiplier:   f32,  // exp(Ea_mean/R × (1/T_ref − 1/T))
    pub entropy_production:       f32,  // cumulative ΔS from irreversible PTMs
    pub ze_velocity_analog:       f32,  // entropy/(entropy + 2.0) → v*=0.456 at ~20yr
}
```

Activation energies: carbonylation 50 / acetylation 40 / **aggregation 80** / phospho 45 /
appendages 55 kJ/mol. At +2°C (chronic SASP): aggregation accelerates +22%.

### `ROSCascadeState` — P23 (level −3: molecules)

4-ODE ROS cascade with Fenton chemistry:

```rust
pub struct ROSCascadeState {
    pub superoxide:         f32,  // O₂⁻ (mitochondrial leak ~2%)
    pub hydrogen_peroxide:  f32,  // H₂O₂ (SOD product, catalase substrate)
    pub hydroxyl_radical:   f32,  // OH· (Fenton: Fe²⁺ + H₂O₂ → OH·)
    pub labile_iron:        f32,  // Fe²⁺ (ferritin degradation, ferroptosis risk)
}
// effective_oh(amp) = OH· × (1 + Fe²⁺ × amp) → CEP164/CEP89/Ninein/CEP170 damage
```

### `ZeHealthState` — P24 (level −5: Ze field)

Maps CAII → Ze-velocity space (Ze Vector Theory, Tkemaladze):

```rust
pub struct ZeHealthState {
    pub v:                    f32,  // v = 0.456 + 0.544 × (1 − CAII)
    pub deviation_from_optimal: f32, // |v − 0.456|; 0 = young, 0.544 = collapse
    pub ze_health_index:      f32,  // = CAII (normalised biomarker)
    pub v_entropy:            f32,  // entropy/(entropy+2.0) from ThermodynamicState
    pub v_consensus:          f32,  // mean(v, v_entropy) — structure + thermodynamics
}
```

At CAII=1.0 (intact appendages): v = v* = 0.456 (optimal). At CAII→0: v→1.0 (collapse).

### `MicrotubuleState` — P25 (level −2: cytoskeleton)

Dynamic instability model replaces scalar `spindle_fidelity`:

```rust
pub struct MicrotubuleState {
    pub polymerization_rate:       f32,  // 0.90 × (1 − acetylation × 0.70)
    pub catastrophe_rate:          f32,  // 0.10 + phospho_dysreg × 0.80
    pub dynamic_instability_index: f32,  // cat / (poly + cat)  [Mitchison & Kirschner 1984]
    pub spindle_fidelity_derived:  f32,  // (1 − DII) × ninein_integrity → overrides CDS.spindle_fidelity
}
```

### `GolgiState` — P26 (level −1: organelles)

Golgi fragmentation → CEP164 glycosylation deficit → accelerated appendage loss:

```rust
pub struct GolgiState {
    pub fragmentation_index:      f32,  // ROS/SASP → cisternae fragmentation [0..1]
    pub glycosylation_capacity:   f32,  // (1 − frag × 0.85).clamp(0.1, 1.0)
    pub cep164_glycosylation:     f32,  // glycosyl × 0.95 — N-glycosylation of CEP164
    pub vesicle_trafficking_rate: f32,  // glycosyl × 0.90 — COPI/COPII to cilia
}
// Extra CEP164 loss = (1 − cep164_glycosylation) × sensitivity × dt
```

### `GeneticProfile` + `GeneticVariant` — P27 (level 0: cell)

SNP-based per-niche DamageParams multipliers — population-level heterogeneity:

```rust
pub enum GeneticVariant {
    Average, Apoe4, Apoe2, Lrrk2G2019s, FoxO3aLongevity, Sod2Ala16Val, Custom,
}

pub struct GeneticProfile {
    pub carbonylation_risk: f32,  // multiplier on base_ros_damage_rate
    pub acetylation_risk:   f32,  // multiplier on acetylation_rate
    pub aggregation_risk:   f32,  // multiplier on aggregation_rate
    pub phospho_risk:       f32,  // multiplier on phospho_dysregulation_rate
    pub appendage_risk:     f32,  // full effect on CEP164/CEP89; ×0.5 excess on Ninein/CEP170
    pub ros_feedback_risk:  f32,  // multiplier on ros_feedback_coefficient (NOT scaled by lf)
    pub longevity_factor:   f32,  // global rate scaler; <1.0 also boosts repair_rates
    pub variant: GeneticVariant,
}
// Presets: average() [all 1.0], apoe4() [lf=1.15], apoe2() [lf=0.90],
//          lrrk2_g2019s() [phospho×1.40, aggreg×1.25], foxo3a_longevity() [lf=0.82],
//          sod2_ala16val() [ros_feedback×1.25]
```

`apply_genetic_modifiers(base, profile)` returns a new `DamageParams` with SNP-adjusted rates.
Entities without `GeneticProfile` use base params (backward-compatible).
Implemented via pre-pass HashMap to stay within hecs 0.10 tuple query limit (15 elements).

---

## 5. human_development_module

### `damage.rs` — Damage Accumulation

The `accumulate_damage()` function implements all five damage types per time
step `dt_years`. Key features:

- **Age multiplier**: after age 40 the damage rate is multiplied by
  `midlife_damage_multiplier` (antagonistic pleiotropy)
- **ROS feedback loop**: `ros_boost = 1 + α × total_damage_score()`
- **Appendage loss is irreversible**: `cep164_integrity` only decreases, never
  recovers
- **Derived metrics** are updated automatically:
  ```
  ciliary_function = mean(appendages) × (1 − aggregates × 0.5)
  spindle_fidelity = (1 − structural_damage) × (1 − phospho × 0.3)
  ```

### `inducers.rs` — S/H Inducer System

Trait `InducerDivisionExt` for `CentriolarInducers`:

```rust
pub trait InducerDivisionExt {
    fn morphogenetic_level(&self) -> MorphogeneticLevel;
    fn asymmetric_divide(&mut self, spindle_ok: bool, rng_val: f32) -> DivisionOutcome;
}

pub enum DivisionOutcome {
    Asymmetric { stem_daughter, differentiating_daughter },
    SymmetricDifferentiation,  // pool exhaustion
    SymmetricSelfRenewal,      // clonal expansion
    TerminalDifferentiation,   // S_count = 0
}
```

When `spindle_ok = false` (damaged spindle), the division has a 0.5 probability
of becoming symmetric — either exhaustion or clonal expansion.

### `tissues.rs` — Tissue-Specific Niches

Each tissue has a unique `TissueProfile`:

| Tissue | Damage multiplier | Ciliary sensitivity | Biological basis |
|--------|------------------|---------------------|-----------------|
| `Hematopoietic` | ×1.3 | 0.9 | HSC: myeloid bias with age |
| `Neural` | ×0.8 | 1.3 | NSC: critical dependence on Shh cilium |
| `IntestinalCrypt` | ×1.2 | 0.7 | High division rate → pool exhausted faster |
| `Muscle` | ×0.9 | 0.8 | Satellite cells: moderate degradation |
| `Skin` | ×1.1 | 0.6 | Epithelium: moderate damage |
| `Germline` | ×0.5 | 1.0 | Germ cells: protected |

### `organism.rs` — Metric Integration

Mapping of tissue metrics → global organism indicators:

```
Neural.functional_capacity          →  cognitive_index
HSC.functional_capacity             →  immune_reserve
Muscle.functional_capacity          →  muscle_mass
(gut + HSC).senescent_fraction / 2  →  inflammaging_score
mean(all tissues).functional_capacity →  frailty_index
```

Death occurs when `frailty_index ≥ 0.95` or `max_lifespan_years` is reached.

---

## 6. Developmental Stages

```
ZYGOTE (day 0)
  No centrioles → totipotency
  S_count = 50, H_count = 4
  Minimal ROS
         │
         ▼  de novo centriole formation
CLEAVAGE (days 1–4)
  Newly formed centrioles — no accumulated damage
  ~2 divisions/day
         │
         ▼
BLASTOCYST (days 4–14)
  ICM: pluripotent cells, S_count ~40
  Trophoblast: onset of differentiation
         │
         ▼
GASTRULATION → ORGANOGENESIS (weeks 2–8)
  Three germ layers, organ formation
  First stem cell niches established
         │
         ▼
FETAL (8 weeks – 9 months)
  Rapid stem cell proliferation
  Centriolar damage remains minimal
         │
         ▼
POSTNATAL (0–18 years)
  Active growth → high division rate
  Slow linear damage accumulation begins
         │
         ▼
ADULT (18–40 years)
  Tissue homeostasis, stable regeneration
  Damage accumulates linearly
         │
         ▼  [key CDATA inflection point]
MIDDLE AGE (40–65 years)
  Antagonistic pleiotropy: ×1.6 damage rate
  HSC: onset of myeloid bias
  SASP activation
  Immunosenescence, inflammaging
         │
         ▼
SENESCENT (65+ years)
  Pronounced sarcopenia, immunodeficiency, cognitive decline
  HSC, intestinal, and skin pools exhausted
  Unstable spindle → clonal haematopoietic expansion
         │
         ▼
DEATH (~76–80 years, normal mode)
  Frailty index ≥ 0.95
  Residual function: Neural (cognition preserved the longest)
```

---

## 7. Parameters and Calibration

### Damage Parameters (`DamageParams`)

| Parameter | Default | Meaning |
|-----------|---------|---------|
| `base_ros_damage_rate` | 0.0018 /yr | Carbonylation of SAS-6, CEP135 |
| `acetylation_rate` | 0.0014 /yr | α-tubulin hyperacetylation |
| `aggregation_rate` | 0.0014 /yr | CPAP, CEP290 aggregates |
| `phospho_dysregulation_rate` | 0.0010 /yr | PLK4/NEK2/PP1 imbalance |
| `cep164_loss_rate` | 0.0027 /yr | CEP164 loss (cilium initiation) |
| `ninein_loss_rate` | 0.0020 /yr | Ninein loss (subdistal appendages) |
| `ros_feedback_coefficient` | 0.12 | Damage → ROS loop strength |
| `midlife_damage_multiplier` | 1.6 | Acceleration factor after age 40 |
| `senescence_threshold` | 0.75 | Damage threshold for senescence entry |

### Development Parameters (`DevelopmentParams`)

| Parameter | Default | Meaning |
|-----------|---------|---------|
| `s_inducers_initial` | 50 | Hayflick limit (~50 divisions) |
| `h_inducers_initial` | 4 | Divisions until meiosis |
| `max_lifespan_years` | 120 | Absolute maximum lifespan |
| `senescence_death_frailty` | 0.95 | Frailty threshold for death |

### Damage Presets

```rust
// Normal aging (~76 years)
DamageParams::default()

// Progeria HGPS (~20 years) — all rates ×5
DamageParams::progeria()

// Longevity (~95 years) — all rates ×0.6
DamageParams::longevity()
```

### Calibration Results

| Mode | Death (years) | Real-world analogue |
|------|--------------|---------------------|
| `normal` | ~76 | Average human lifespan |
| `longevity` | ~95 | Long-lived individuals |
| `progeria` | ~20 | HGPS (actual: 13–14 years) |

---

## 8. Running the Simulation

### Basic run

```bash
cargo run --bin human_lifecycle
```

### Aging modes

```bash
# Normal aging (~76 years)
cargo run --bin human_lifecycle

# Progeria (~20 years)
cargo run --bin human_lifecycle -- --mode progeria

# Longevity (~95 years)
cargo run --bin human_lifecycle -- --mode longevity
```

### Using the module in code

```rust
use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams,
    damage::DamageParams,
    development::DevelopmentParams,
};
use cell_dt_core::SimulationModule;

// Create with default parameters
let mut module = HumanDevelopmentModule::new();

// Or with custom parameters
let params = HumanDevelopmentParams {
    damage: DamageParams::longevity(),
    development: DevelopmentParams {
        s_inducers_initial: 50,
        max_lifespan_years: 120.0,
        ..Default::default()
    },
    steps_per_year: 10,
    seed: 42,
};
let mut module = HumanDevelopmentModule::with_params(params);

// Simulation step (dt = 1 step = 0.1 year at steps_per_year = 10)
let mut world = cell_dt_core::hecs::World::new();
module.initialize(&mut world).unwrap();
module.step(&mut world, 1.0).unwrap();

// Retrieve a metric snapshot
let snap = module.snapshot();
println!("Age:       {:.1} years", snap.age_years);
println!("Frailty:   {:.3}", snap.frailty);
println!("Cognition: {:.3}", snap.cognitive);
println!("Immunity:  {:.3}", snap.immune);
```

### Sample output

```
╔══════════════════════════════════════════════════════════════════╗
║    CDATA — Human Lifecycle Simulation                            ║
║    Centriolar Damage Accumulation Theory of Aging                ║
║    Mode: NORMAL                                                  ║
╚══════════════════════════════════════════════════════════════════╝

Age     Stage             Frailty  Cognit.  Immune  Muscle  Inflamm  Alive?
────────────────────────────────────────────────────────────────────────────
   0.1  Organogenesis     0.000    1.000    1.000   1.000   0.000    +
  18.1  Adult             0.212    0.936    0.758   0.889   0.002    +
  40.0  MiddleAge         0.689    0.811    0.197   0.596   0.027    +
  65.0  Senescent         0.898    0.532    0.166   0.084   0.150    +
  76.2  Death             0.950    0.394    0.138   0.000   0.269    ✗

  ✦ Organism died at age 76.2 years
  ✦ Frailty index: 0.950
```

---

## 9. Results and Interpretation

### Order of tissue pool exhaustion

In normal mode, tissues degrade in the following order:

1. **Skin** (`Skin`) — exhausted first (~50–55 years)
2. **Intestine** (`IntestinalCrypt`) — ~55–60 years
3. **HSC** (`Hematopoietic`) — ~60–65 years → immunodeficiency
4. **Muscle** (`Muscle`) — ~70–75 years → sarcopenia
5. **Neural SC** (`Neural`) — preserved the longest

This matches clinical observations: neurodegeneration is the latest stage of
aging.

### Key CDATA predictions modelled

| Phenomenon | Mechanism in model | Supported by data |
|------------|-------------------|-------------------|
| Immunosenescence | HSC: loss of CEP164 → no cilium → no Wnt → myeloid bias | Yes |
| Sarcopenia | Satellite cells: defective spindle → symmetric exhaustion | Yes |
| Cognitive decline | NSC: Shh dependence → loss of neurogenesis | Yes |
| Inflammaging | Senescent HSC + intestine → SASP | Yes |
| Progeria | All rates ×5 → death ~20 years | HGPS: 13–14 years |
| Stem cell transplantation | Reset of `CentriolarDamageState` → lifespan +>30% | Mouse experimental data |

### Why neural stem cells last the longest

This is not accidental — neural stem cells have:
- The lowest damage multiplier (×0.8)
- The highest ciliary sensitivity (×1.3) — they "hear" Shh longest
- The slowest division rate (0.8/year) → their pool depletes more slowly

---

## 10. Biological Hierarchy — 11 Levels

CDATA positions the **cell as the autonomous unit** — the central level (0).
All other levels are either sub-structures (negative) or supra-cellular context (positive).

```
+5  Noosphere     — interventions, evidence base, AI integration (AIM)
+4  Society       — social stress, loneliness → cortisol → ROS
+3  Organism      — OrganismState: frailty, cognitive, HPA axis, metabolism
+2  Organs        — OrganState: 11 organs, poly-organ failure criterion  ✅
+1  Tissues       — TissueState: 11 types, ECM, vascular niche, fibrosis ✅
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 0  CELL ★        — CentriolarDamageState, GeneticProfile, CloneEpigeneticState, FateSwitchingState  ← default tab
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
-1  Organelles    — GolgiState, MitochondrialState, ER, Lysosome, Ribosome [TODO]
-2  Cytoskeleton  — MicrotubuleState, IFTState, ActinRingState             [TODO]
-3  Molecules     — ROSCascadeState, AppendageProteinState, ATPEnergy      [TODO]
-4  Atoms         — ThermodynamicState (Arrhenius, entropy)
-5  Ze field      — ZeHealthState (Ze Vector Theory, v*)
```

### Status

| Level | Name | Status | Key Components |
|-------|------|--------|---------------|
| −5 | Ze field | ✅ | ZeHealthState |
| −4 | Atoms | ✅ | ThermodynamicState |
| −3 | Molecules | ✅ | ROSCascadeState, AppendageProteinState, ATPEnergyState, ChromatinState |
| −2 | Cytoskeleton | ✅ | MicrotubuleState, IFTState, ActinRingState |
| −1 | Organelles | ✅ | GolgiState, MitochondrialState, ERStressState, LysosomeState, PeroxisomeState, RibosomeState |
| 0 | **Cell** ★ | ✅ | CentriolarDamageState, GeneticProfile, CloneEpigeneticState, FateSwitchingState |
| +1 | Tissues | ✅ | TissueState, ExtracellularMatrixState, VascularNicheState, FibrosisState |
| +2 | Organs | ✅ | OrganState (11 organs), poly-organ failure, cardiac oxygen delivery |
| +3 | Organism | ✅ | OrganismState, HPAAxisState, MetabolicPhenotypeState |
| +4 | Society | ❌ | SocialStressInput TODO |
| +5 | Noosphere | ✅ partial | Interventions (P11); AIM integration TODO |

### GUI Architecture

The `cell_dt_gui` crate (egui) maps directly to this hierarchy:
- **11 tabs**, one per level (−5 to +5)
- **Level 0 "Cell" tab opens by default** — the central CDATA unit
- Each tab shows parameters and live metrics for its level
- **7 languages**: EN / FR / ES / RU / ZH / AR / KA (Georgian)
- Language persisted in `~/.config/cell_dt_gui/settings.toml`

---

## 11. References

1. **Tkemaladze J., Chichinadze K.** Centriolar mechanisms of differentiation
   and replicative aging of higher animal cells. *Biochemistry (Moscow)*, 70(11),
   2005.

2. **Tkemaladze J.** Old Centrioles Make Old Bodies. *Annals of Rejuvenation
   Science*, 1(1), 2026.

3. **Tkemaladze J.** Reduction, proliferation, and differentiation defects of
   stem cells over time: a consequence of selective accumulation of old centrioles
   in the stem cells? *Molecular Biology Reports*, 50(3):2751–2761, 2023.
   PMID: 36583780. https://pubmed.ncbi.nlm.nih.gov/36583780/

4. **Tkemaladze J.** The Centriolar Theory of Differentiation Explains the
   Biological Meaning of the Centriolar Theory of Organismal Aging.
   *Longevity Horizon*, 1(3), 2025.

5. **Tkemaladze J., Chichinadze K.** Potential role of centrioles in determining
   the morphogenetic status of animal somatic cells. *Cell Biology International*,
   29(5):370–374, 2005. PMID: 15886028.
   https://pubmed.ncbi.nlm.nih.gov/15886028/

6. **Tkemaladze J., Chichinadze K.** Centriole, differentiation, and senescence.
   *Rejuvenation Research*, 13(2–3):339–342, 2010. PMID: 20426623.
   https://pubmed.ncbi.nlm.nih.gov/20426623/

7. **Lezhava T., Jokhadze T., Tkemaladze J., Tsiskarishvili N., Chikava L.**
   Gerontology research in Georgia. *Biogerontology*, 12(2):87–91, 2011.
   PMID: 20480236. https://pubmed.ncbi.nlm.nih.gov/20480236/

8. **Chichinadze K., Tkemaladze J., Lazarashvili A.** Discovery of centrosomal RNA
   and centrosomal hypothesis of cellular ageing and differentiation.
   *Nucleosides, Nucleotides and Nucleic Acids*, 31(3):172–183, 2012.
   PMID: 22356233. https://pubmed.ncbi.nlm.nih.gov/22356233/

9. **Chichinadze K., Lazarashvili A., Tkemaladze J.** RNA in centrosomes:
   structure and possible functions. *Protoplasma*, 250(1):397–405, 2013.
   PMID: 22684578. https://pubmed.ncbi.nlm.nih.gov/22684578/

10. **Tkemaladze J.** Editorial: Molecular mechanism of ageing and therapeutic
    advances through targeting glycative and oxidative stress.
    *Frontiers in Pharmacology*, 14:1324446, 2024. PMID: 38510429.
    https://pubmed.ncbi.nlm.nih.gov/38510429/

11. **Tkemaladze J.** CDATA Digital Twin — Cell Differentiation and Aging
    Simulation Platform (v0.3). *Zenodo*, 2026.
    DOI: https://doi.org/10.5281/zenodo.19174506

12. **Tkemaladze J.** Ze Vector Theory of Biological Aging — Validation Dataset
    and Analysis Scripts. *Zenodo*, 2026.
    DOI: https://doi.org/10.5281/zenodo.19174630

---

*Cell DT Platform — High-Performance Cell Differentiation Simulator*
*https://github.com/djabbat/cell_dt*

2024–2026 © Jaba Tkemaladze. All rights reserved. No warranty. For educational and research use only.

---

## 11. Иерархия уровней — архитектурная карта CDATA

Клетка — **автономная единица** симуляции. Снизу — субклеточные уровни (источники повреждения).
Сверху — надклеточные уровни (контекст, в котором клетка функционирует).

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ +6  ЭКОСФЕРА / БИОСФЕРА
            Замкнутые биогеохимические циклы. Вид как элемент.
            Потоки: C, N, O, энергия солнца → биомасса → распад
            CDATA: human lifespan как эволюционно-оптимальный параметр
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ +5  НООСФЕРА
            Информационная оболочка (Вернадский). Культура, наука,
            медицина как управляющие сигналы на биологию.
            CDATA: интервенции — сенолитики, NAD+, образ жизни →
            параметры модели меняются извне ноосферой
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ +4  СОЦИУМ
            Сеть индивидов. Стресс ↔ кортизол ↔ ROS ↔ CDATA.
            Одиночество → inflammaging↑. Когезия → longevity↑.
            CDATA: social_stress: f32 → ros_feedback_coefficient×
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ +3  ОРГАНИЗМ                           [частично в CDATA]
            OrganismState: frailty, cognitive, immune, muscle
            Смерть = frailty ≥ 0.95 или панцитопения или нейродегенерация
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ +2  ОРГАНЫ
            Сердце, почки, печень, лёгкие, мозг.
            Каждый орган = специфический набор тканей.
            CDATA: OrganState (11 типов) — реализован (P41). ✅
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ +1  ТКАНИ                              [в CDATA: TissueState]
            Skin, Hematopoietic, Neural, Muscle, IntestinalCrypt,
            Liver, Germline, Bone, Lung, Kidney, Cardiac (11 типов)
            Нет: межклеточный матрикс, сосудистая ниша
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ  0  ═══ КЛЕТКА ═══ АВТОНОМНАЯ ЕДИНИЦА ═══ [ядро CDATA]
            Стволовая ниша. ECS-сущность. Все компоненты.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ -1  ОРГАНОИДЫ (мембранные)
            Митохондрии  ✅ Track E (мтДНК, ROS, mito_shield)
            Гольджи      ✅ P26 GolgiState: фрагментация → гликозил. CEP164
            ЭПС шерохов. ❌ UPR-стресс, скорость синтеза белков
            ЭПС гладкая  ❌ Ca²⁺-буфер, детоксикация
            Лизосомы     ❌ аутолиз (есть AutophagyState — без органеллы)
            Пероксисомы  ❌ H₂O₂/каталаза баланс с ROS
            Ядро/хромат. ⚠️ TelomereState, DDRState, EpigeneticClockState
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ -2  ОРГАНЕЛЛЫ (немембранные / цитоскелет)
            Центриоли     ✅ CentriolarInducerPair, PTM (4 типа)
            Дист. придатки ✅ P21 AppendageProteinState (4 белка отдельно)
            Первичная рес. ✅ ciliary_function → Shh/Wnt
            Микротрубочки ✅ P25 MicrotubuleState: DII → spindle_fidelity_derived
            Рибосомы      ❌ скорость трансляции = константа
            Акт. филамен. ❌ кольцо сужения, миграция
            Промеж. фила. ❌ защита ядра от механических сил
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ -3  МОЛЕКУЛЫ
            PTM центриолей ✅ карбонилирование, ацетилирование,
                             агрегация, фосфорилирование (4 скаляра)
            CEP164/89/Ninein/CEP170 ✅ P21 — 4 независимых протеина + CAII
            ROS-каскад    ✅ P23 ROSCascadeState: O₂⁻→H₂O₂→OH·→Fe²⁺ (4 ОДУ)
            ATP/ADP       ❌ энергетический статус клетки
            Хроматин      ⚠️ methylation_age (1 число), нет 3D-структуры
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ -4  АТОМЫ (иерархическая термодинамика)
            ✅ P22 ThermodynamicState: T = baseline + SASP
            damage_rate × exp(Ea_mean/R × (1/T_ref − 1/T))
            Ea: агрегация 80 > придатки 55 > карбонил. 50 > фосфо 45 > ацетил. 40 кДж/моль
            entropy_production → ze_velocity_analog → v*=0.456 (~20 лет)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
УРОВЕНЬ -5  КВАРКИ / Ze-ПОЛЕ
            ✅ P24 ZeHealthState: v = v* + 0.544×(1−CAII)
            v* = 0.456 (критическая точка T/S квантов, Tkemaladze)
            v_consensus = mean(v, v_entropy) — структура + термодинамика
            interpretation(): optimal/mild/moderate/severe/near_collapse
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Принцип моделирования по уровням

Каждый уровень характеризуется собственным **временны́м масштабом**:

| Уровень | Характерное время | Метод в CDATA |
|---|---|---|
| Кварки | < 10⁻²³ с | — не моделируется |
| Атомы | 10⁻¹⁵ – 10⁻¹² с | Термодинамические коэффициенты |
| Молекулы | 10⁻³ – 10³ с | Mean-field: концентрации [0..1] |
| Органеллы | 10² – 10⁵ с | Скалярные метрики, 1 день = 1 шаг |
| Клетка | 10⁴ – 10⁶ с | ECS-сущность — **текущий уровень** |
| Ткань | 10⁶ – 10⁸ с | TissueState: пул, темп, сенесценция |
| Орган | 10⁷ – 10⁸ с | OrganState ✅ — 11 органов, полиорганная недостаточность |
| Организм | 10⁸ – 10⁹ с | OrganismState: frailty, смерть |
| Социум | 10⁸ – 10¹⁰ с | social_stress → параметр ROS |
| Ноосфера | 10⁹ – 10¹¹ с | Интервенции из базы знаний |
| Экосфера | 10¹⁰ – 10¹² с | Эволюционные ограничения lifespan |

### Что CDATA моделирует vs что нужно добавить

**Реализовано (✅):**
- Уровень −5: ZeHealthState (P24) — v = v* + 0.544×(1−CAII)
- Уровень −4: ThermodynamicState (P22) — Аррениус, 5 Ea, Ze-энтропия
- Уровень −3: ROSCascadeState (P23) — 4 ОДУ Фентон; AppendageProteinState (P21) — 4 белка + CAII
- Уровень −2: MicrotubuleState (P25) — DII = cat/(poly+cat) → spindle_fidelity
- Уровень −1: GolgiState (P26) — фрагментация → CEP164 гликозилирование → распад придатков
- Уровень 0: клетка (ядро CDATA, ECS, шум, интервенции P11)
- Уровень +1: TissueState (11 типов), StemCellDivisionRateState (Track F)
- Уровень +3: OrganismState (frailty, смерть 3 критерия, системный SASP, IGF-1)

**Ближайшие приоритеты (❌→):**

```
1. GolgiState ✅ P26 — реализован 2026-03-23
2. Genetic heterogeneity: SNP-профили → разные DamageParams на ниши
3. OrganState: 11 органов — агрегация тканей → полиорганная недостаточность ✅ P41
4. LysosomeState: pH, hydrolase_activity — связь с AutophagyState
5. SocialStressState: stress → cortisol → ROS → CDATA (надклеточный вход)
```


---

## 12. Graphical Interface — Cell DT GUI

> **Launch:**
> ```bash
> cargo run --bin cell_dt_gui --release
> # or
> ./run_gui.sh
> ```
> Window size: 1200 × 820 px (resizable, min 900 × 650).

---

### 12.1 Layout — five zones

```
┌─────────────────────────────────────────────────────────────────┐
│  TOP PANEL (2 rows)                                             │
│  Row 1: 🧬 title                    language ▾   ❌ Exit        │
│  Row 2: ↩️Undo  ↪️Redo │ Load Save Presets 🐍Export Validate   │
├──────────────────┬──────────────────────────────┬───────────────┤
│                  │                              │               │
│   LEFT PANEL     │     CENTRAL PANEL            │  RIGHT PANEL  │
│   (tab list)     │  (tab content / dashboard)   │  (realtime)   │
│                  │                              │               │
├──────────────────┴──────────────────────────────┴───────────────┤
│  BOTTOM PANEL: ▶ Run Simulation  │  ← Back to settings          │
└─────────────────────────────────────────────────────────────────┘
```

---

### 12.2 Top panel

**Row 1 — title and application controls**

| Element | Description |
|---------|-------------|
| `🧬 Cell DT — Simulation Configurator` | Application title; changes to the localized variant when a non-English language is selected |
| Language picker | Drop-down: **EN · FR · ES · RU · ZH · AR · KA** (Georgian). Fonts loaded automatically: NotoSansGeorgian, NotoSansArabic, NotoSansCJK |
| ❌ Exit | Terminates the process immediately (`std::process::exit(0)`) |

**Row 2 — action toolbar (left-aligned)**

| Button | Shortcut logic | Effect |
|--------|---------------|--------|
| `↩️ Undo` | Grayed when no history | Restores previous parameter state (up to 50 steps) |
| `↪️ Redo` | Grayed when at latest state | Re-applies undone change |
| `Load` | — | Opens the **Load dialog** — select a `.toml` or `.yaml` config file |
| `Save` | — | Opens the **Save dialog** — write current settings to file |
| `Presets` | — | Opens the **Presets dialog** — choose from 6 ready-made configurations |
| `🐍 Export to Python` | — | Opens the **Python Export dialog** — generates a ready-to-run `cell_dt` Python script |
| `Validate` | — | Runs parameter validation; opens the **Validation dialog** with a list of errors and warnings |
| Status message | Auto | Green text showing the last event (simulation started, completed, stopped, error) |

---

### 12.3 Left panel — tab navigation

Nine tabs are listed vertically. The active tab is highlighted. Clicking a tab also records a history snapshot.

| Icon | Tab name | Module |
|------|----------|--------|
| ⚙️ | Simulation | `SimulationConfig` |
| 🔬 | Centriole | `CentrioleConfig` |
| 🔄 | Cell Cycle | `CellCycleConfig` |
| 🧬 | Transcriptome | `TranscriptomeConfig` |
| ⚖️ | Asymmetric Division | `AsymmetricDivisionConfig` |
| 🌱 | Stem Hierarchy | `StemHierarchyConfig` |
| 💾 | I/O | `IOConfig` |
| 📊 | Visualization | `VisualizationConfig` |
| 🔴 | CDATA / Aging | `CdataGuiConfig` |

---

### 12.4 Right panel — real-time visualization

A collapsible side panel with a checkbox **Enable**. When enabled, the GUI continuously snapshots selected parameter values (up to 100 snapshots in a ring buffer) and displays the current value next to each parameter name. Parameters tracked by default:

- `simulation.max_steps`
- `centriole.acetylation_rate`
- `cell_cycle.base_cycle_time`

An **⚙️ Settings** collapsible inside the panel allows adding more parameters to the list.

---

### 12.5 Bottom panel

**While simulation is running:**

| Element | Description |
|---------|-------------|
| `⏹ STOP` | Dark-red button; stops the simulation immediately and records the step at which it stopped |
| Progress bar | `egui::ProgressBar` with animation; shows `Step N / M (X.X%)` above |

**While simulation is idle:**

| Element | Description |
|---------|-------------|
| `▶ Run Simulation` | Dark-navy button (`#10 1E 34`) with a slow teal breathing glow (~0.7 Hz); text color `#DC E8 F8`. Starts the simulation. |
| `← Back to settings` | Appears only after a simulation has completed. Returns the central panel from the results dashboard to the tab view. |

---

### 12.6 Central panel — tab contents

#### ⚙️ Simulation tab

Core execution parameters. All controls record an undo snapshot on change.

| Parameter | Type | Range | Default | Notes |
|-----------|------|-------|---------|-------|
| Number of steps | Slider | 1 – 1 000 000 | — | Total ECS steps per run |
| Time step (dt) | Slider (log) | 0.001 – 1.0 | — | Simulation time per step; values > 1 trigger a validation warning |
| Checkpoint interval | Slider | 1 – 10 000 | — | Steps between state saves |
| Number of threads | Slider | 1 – 64 | 1 | Rayon thread pool size |
| Random seed | Slider | 0 – 999 999 | 42 | Reproducibility seed |
| Output directory | Text field | — | `results/` | Directory for CSV / checkpoint output |
| Parallel module execution | Checkbox | — | off | Run modules in parallel within each step |

---

#### 🔬 Centriole tab

Controls the `CentrioleModule` — PTM accumulation on the mother/daughter centriole pair.

| Parameter | Range | Default | Notes |
|-----------|-------|---------|-------|
| Enable module | checkbox | on | |
| Acetylation rate | 0 – 0.1 | 0.0002 | Tubulin acetylation per step (mother); validation: must stay in range |
| Oxidation rate | 0 – 0.1 | 0.0001 | Tubulin oxidation per step (mother) |
| Parallel cell processing | checkbox | off | Processes all centriole ECS entities with Rayon |

---

#### 🔄 Cell Cycle tab

Controls the `CellCycleModule` — G1/S/G2/M phase progression, cyclins, and checkpoints.

| Parameter | Range | Default | Notes |
|-----------|-------|---------|-------|
| Enable module | checkbox | on | |
| Base cycle duration | 1 – 100 h | 24.0 | Full G1→M cycle length in hours |
| Checkpoint strictness | 0 – 1 | 0.0 | 0 = no arrests; 0.3 = moderate; 0.7 = strict G1/G2 arrest |
| Enable apoptosis | checkbox | on | Trigger apoptosis when checkpoint arrest is unresolved |
| Nutrient availability | 0 – 1 | — | Scales G1 progression |
| Growth factor level | 0 – 1 | — | Scales cyclin D accumulation |
| Random variation | 0 – 1 | — | Langevin noise on cycle duration |

---

#### 🧬 Transcriptome tab

Controls the `TranscriptomeModule` — gene expression states (p21, p16, cyclin D/E, MYC) and mutation accumulation.

| Parameter | Range | Default | Notes |
|-----------|-------|---------|-------|
| Enable module | checkbox | on | |
| Mutation rate | 0 – 0.01 (log) | 0.0001 | Somatic mutation rate per step; validation: must be ≤ 0.1 |
| Noise level | 0 – 0.5 | 0.02 | Stochastic noise on gene expression levels |

---

#### ⚖️ Asymmetric Division tab

Controls the `AsymmetricDivisionModule` — division type classification, NichePool management, and CHIP drift.

| Parameter | Range | Default | Notes |
|-----------|-------|---------|-------|
| Enable module | checkbox | on | |
| Asymmetric division probability | 0 – 1 | 0.30 | Probability of an asymmetric outcome per division |
| Self-renewal probability | 0 – 1 | 0.40 | Probability of symmetric renewal |
| Differentiation probability | 0 – 1 | 0.30 | Probability of symmetric differentiation |
| Niche capacity | 1 – 100 | 10 | Maximum stem cells per niche |
| Maximum niches | 1 – 1000 | 100 | Total number of niches in the simulation |
| Enable polarity | checkbox | on | Activates spindle-orientation polarity logic |
| Enable fate determinants | checkbox | on | Activates Numb/Wnt asymmetric fate determinants |

> **Validation:** sum of the three division probabilities must be ≈ 1.0 (tolerance 0.01); niche_capacity must be > 0.

---

#### 🌱 Stem Hierarchy tab

Controls the `StemCellHierarchyModule` — potency level tracking and plasticity.

| Parameter | Type | Default | Notes |
|-----------|------|---------|-------|
| Enable module | checkbox | on | |
| Initial potency level | Dropdown | Pluripotent | Totipotent / Pluripotent / Multipotent / Differentiated |
| Enable plasticity | checkbox | on | Allow dedifferentiation |
| Plasticity rate | 0 – 0.1 (log) | 0.01 | Per-step probability of dedifferentiation |
| Differentiation threshold | 0 – 1 | 0.70 | `spindle_fidelity` below this value triggers potency drop |

---

#### 💾 I/O tab

Controls file output — CSV export, checkpoints, compression.

| Parameter | Type | Default | Notes |
|-----------|------|---------|-------|
| Enable module | checkbox | on | |
| Output directory | Text field | `results` | Destination for all output files |
| Format | Dropdown | CSV | CSV · Parquet · HDF5 |
| Compression | Dropdown | None | None · Snappy · Gzip |
| Buffer size | 100 – 10 000 | 1000 | Write buffer in rows |
| Save checkpoints | checkbox | on | |
| Checkpoint interval | 10 – 1000 | 100 | Steps between checkpoint writes |
| Maximum checkpoints | 1 – 100 | 10 | Older checkpoints are deleted when limit is reached |

---

#### 📊 Visualization tab — Aging Trajectory Showcase

Four interactive plots, each 230 px tall. All curves are pre-computed from calibrated CDATA model equations. Plots support **zoom** (scroll), **pan** (drag), and **legend toggle** (click on a legend entry).

**① Frailty Index Trajectory**

`frailty(age, scale) = 1 / (1 + exp(−0.08·scale·(age − 45/√scale)))`

Four curves + a horizontal dashed **Death threshold** at 0.95:

| Curve | Color | Scale | Predicted lifespan |
|-------|-------|-------|--------------------|
| Control | Blue | 1.0 | ~78 yr |
| Longevity preset | Green | 0.55 | ~108 yr |
| Progeria | Orange | 5.0 | ~15 yr |
| CentrosomeTransplant @ yr 50 | Gold | 1.0→0.5 | ~93 yr |

**② Stem Cell Pool Depletion**

`pool(age, scale) = max(0, 1 − 0.011·scale·age)`

Same four scenarios. Pool reaching zero corresponds to pancytopenia (death criterion).

**③ Kaplan–Meier Survival Curves**

`S(age, μ) = exp(−(age/μ)⁴)`

Cohort survival probability. Median (μ) values: 78, 108, 15, 93 yr.

**④ CAII Biomarker & Epigenetic Clock Acceleration**

- CAII (carbonic anhydrase II marker): `caii(age, scale) = min(1, 0.008·scale·age²/100)`
- Epigenetic clock: `epi(age, scale) = age·(1 + 0.5·scale·age/100)` normalised to [0..1.5]

Both shown for Control (solid) and Longevity (dashed).

**Collapsible output settings:** enable/disable the module, update interval (1–100 steps), output directory, save plots on/off.

**Citation footer:** *Tkemaladze J. Mol Biol Reports 2023 (PMID 36583780)*

---

#### 🔴 CDATA / Aging tab

The core CDATA theory parameters. All controls are grouped in collapsible sections.

**🔬 Inducer system**

| Parameter | Range | Default | Biological meaning |
|-----------|-------|---------|-------------------|
| `base_detach_probability` | 0 – 0.01 (log) | 0.0003 | Per-step O₂-dependent probability of losing one inducer from the mother or daughter centriole. Calibrated to produce lifespan ≈ 78 yr. |
| `mother_bias` | 0 – 1 | 0.60 | Fraction of total detachments falling on the **mother** centriole (older, more PTM-covered centriole). 0 = symmetric; 1 = mother only. |
| `age_bias_coefficient` | 0 – 0.01 (log) | 0.003 | How much each additional year of age further shifts the bias toward the mother centriole. |

**🧬 Inductor lifecycle**

| Parameter | Range | Default | Biological meaning |
|-----------|-------|---------|-------------------|
| `de_novo_centriole_division` | 1 – 8 (integer) | 4 | Blastomere division number (from zygote) at which centrioles with differentiation inductors are assembled de novo. 4 = morula (16-cell stage) — the human biological default. Earlier stages have `inductors_active = false`. Stage label shown live (Zygote / Cleavage / Morula ✓ / Blastocyst / Implantation). |
| `meiotic_elimination_enabled` | checkbox | on | When enabled, the Adolescence stage registers meiotic elimination of centrioles; the next generation starts from DifferentiationStatus::Totipotent. Biologically correct default: on. |

**🩸 Myeloid shift (weights)**

Weights for the formula:
`myeloid_bias = spindle_w·(1−spindle)^1.5 + cilia_w·(1−cilia) + ros_w·ros + agg_w·aggregates`

| Parameter | Range | Default |
|-----------|-------|---------|
| `spindle_weight` | 0 – 1 | 0.45 |
| `cilia_weight` | 0 – 1 | 0.30 |
| `ros_weight` | 0 – 1 | 0.15 |
| `aggregate_weight` | 0 – 1 | 0.10 |

A live **Σ indicator** shows the current sum in green (≈ 1.00) or yellow (off-target).

**⚡ Damage preset**

| Preset | Scale | Expected lifespan |
|--------|-------|------------------|
| Normal aging | ×1.0 | ~78 yr |
| Progeria (×5 rates) | ×5.0 | ~15 yr (Hutchinson-Gilford) |
| Longevity (×0.6 rates) | ×0.6 | ~108 yr |

**🔋 Track E — Mitochondria (read-only formula display)**

Shows the equations linking mtDNA mutations → ROS → fusion index → mito_shield → O₂ at centriole. Reference only; parameters are controlled by `MitochondrialModule`.

**📉 Track F — Division rate**

`division_rate = cilia_drive × spindle_drive × age_factor × ros_brake × mtor_brake`

| Parameter | Range | Default | Effect |
|-----------|-------|---------|--------|
| `division_rate_floor` | 0.05 – 0.50 | 0.15 | Minimum value of `age_factor` (prevents division rate from reaching zero in elderly) |
| `ros_brake_strength` | 0 – 1 | 0.40 | `ros_brake = 1 − ros_level × strength` |
| `mtor_brake_strength` | 0 – 1 | 0.35 | `mtor_brake = 1 − max(0, mtor−0.3) × strength` |

---

### 12.7 Live dashboard (during and after simulation)

When the RUN button is pressed, the central panel switches from the tab view to the **Live Dashboard**. The dashboard remains visible after the simulation finishes, until the user clicks **← Back to settings**.

**Header line:**
```
Age  42.7 yr    step 15 623  /  36 500    42.8%
```
Updates every ~50 ms during the run.

**Three interactive plots (all zoomable / pannable):**

| Plot | Y-axis | Curves |
|------|--------|--------|
| **Frailty index** | 0 – 1 | Control (blue), Longevity (green), Progeria (orange) + gold **"Now" cursor** |
| **Stem-cell pool** | 0 – 1 | Same three scenarios + cursor |
| **Biomarkers** | 0 – 1 (normalised) | ROS (red), Myeloid bias (amber), Telomere length (cyan), Epigenetic clock (purple) + cursor |

The **gold dashed cursor** is synchronized to `sim_progress → current age (years)` and moves right as the simulation advances.

**Footer note:** *Curves are calibrated CDATA model projections. Real-time ECS data will replace these in v0.4.*

---

### 12.8 Dialogs

#### Save / Load

Both dialogs present a single text field for the file path and a **Save / Load** button plus **Cancel**. Format is auto-detected from the file extension (`.toml` or `.yaml`).

#### Presets

Six ready-made configurations. Each shows an icon, name, and one-line description. Clicking **Apply** overwrites the current settings and adds a history snapshot.

| Preset | Icon | Key settings |
|--------|------|-------------|
| Quick Test | ⚡ | 100 steps, dt 0.1, centriole + cell cycle only |
| Standard Experiment | 🔬 | 10 000 steps, dt 0.05, 8 threads, all core modules |
| High Performance | 🚀 | 100 000 steps, 16 threads, parallel modules, no viz/checkpoints |
| Stem Cells | 🌱 | 50 000 steps, asymmetric_probability 0.4, Pluripotent start |
| Cell Cycle | 🔄 | 20 000 steps, dt 0.02, checkpoint_strictness 0.3, apoptosis on |
| Transcriptome Analysis | 🧬 | 5 000 steps, dt 0.05, mutation_rate 0.001, Parquet+Snappy output |

#### Export to Python

Generates a complete Python script (`cell_dt` PyO3 bindings) reflecting the current GUI settings:
- `PySimulation(max_steps, dt, num_threads, seed)`
- `create_population(100)` or `create_population_with_transcriptome(100)`
- `PyCellCycleParams(...)` with all cell cycle sliders
- `register_modules(enable_centriole, enable_cell_cycle, ...)`
- `sim.run()` + analysis + `matplotlib` bar chart of phase distribution

Two buttons: **📋 Copy to clipboard** and **💾 Save as script.py**.

#### Validation

Runs `ParameterValidator::validate_all()` and lists all findings:

| Check | Severity |
|-------|----------|
| max_steps = 0 | ❌ Error |
| dt ≤ 0 | ❌ Error |
| dt > 1.0 | ⚠️ Warning |
| acetylation/oxidation rates out of [0, 0.1] | ❌ Error |
| base_cycle_time ≤ 0 | ❌ Error |
| checkpoint_strictness outside [0, 1] | ❌ Error |
| mutation_rate outside [0, 0.1] | ❌ Error |
| sum of division probabilities ≠ 1 | ⚠️ Warning |
| niche_capacity = 0 | ❌ Error |

Shows a **Close** button. All errors must be resolved before results can be considered valid.

---

### 12.9 History system (Undo / Redo)

- Every interactive control (slider, checkbox, combo, text field) calls `push_history()` on change.
- History is a `VecDeque<ConfigAppState>` with a maximum of **50 states**.
- `Undo` decrements the index and restores the snapshot; `Redo` increments it.
- The Undo button is grayed when the index is at 0; Redo is grayed when at the latest state.
- Simulation run-state fields (`simulation_running`, `sim_progress`, `sim_elapsed_steps`) are **not** subject to Undo — only parameter configuration is tracked.

---

### 12.10 Multilingual support

Seven languages are available in the language picker. The UI font stack is extended at startup:

| Language | Font added | Notes |
|----------|-----------|-------|
| English / French / Spanish / Russian | System default | Latin + Cyrillic covered |
| Georgian (KA) | `NotoSansGeorgian-Regular.ttf` | Bundled in `crates/cell_dt_gui/fonts/` |
| Arabic (AR) | `NotoSansArabic-Regular.ttf` | Bundled; right-to-left rendering via egui |
| Chinese (ZH) | `NotoSansCJK-Regular.ttc` | Loaded from system path if present |

All translatable strings are defined in `crates/cell_dt_gui/src/i18n.rs` as static `Translations` structs keyed by `Lang` enum.

---

### 12.11 Build and run

```bash
# Development build
cargo build -p cell_dt_gui

# Release build (recommended — significantly faster rendering)
cargo build -p cell_dt_gui --release

# Run directly
cargo run --bin cell_dt_gui --release

# Via shell wrapper (sets RUST_LOG and working directory)
./run_gui.sh
```

Dependencies: `eframe 0.27`, `egui 0.27`, `egui_plot 0.27`, `serde` + `serde_json`.
