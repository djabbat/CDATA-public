# CLAUDE.md — CDATA (Cell-DT v3.0)

## Project Identity

**CDATA** — Centriolar Damage Accumulation Theory of Aging
**Cell-DT** — Digital Twin Simulator of Human Aging
**Version:** 3.0 | **Status:** R²=0.84 validated | **EIC Pathfinder deadline: 10 May 2026**
**Location:** `~/Desktop/CDATA/`
**Repos:** private `djabbat/CDATA` · public `djabbat/CDATA-public`

---

## Source of Truth

**CONCEPT.md is the authoritative document.**
All code, parameters, and documentation must match CONCEPT.md.
If there is a conflict between code and CONCEPT.md, fix the code — never change CONCEPT.md without user approval.

---

## Architecture

```
crates/
├── cell_dt_core/             ECS core, components, systems, 32 fixed parameters
├── cell_dt_modules/
│   ├── mitochondrial/        Track E: sigmoid ROS, mito_shield, mtDNA mutations
│   ├── inflammaging/         SASP, DAMPs, cGAS-STING, NK clearance, NF-κB
│   ├── asymmetric_division/  Stochastic inheritance, CHIP (DNMT3A/TET2)
│   └── tissue_specific/      4 tissues: HSC, ISC, Muscle, Neural
├── cell_dt_validation/       MCMC calibration (NUTS), biomarker validation, basic_simulation
└── cell_dt_python/           PyO3 Python bindings

gui/                          cdata_gui.py (requires display — skipped in headless)
docs/                         README.md, additional documentation
Horizon/                      EIC Pathfinder grant (separate sub-project)
```

---

## Critical Constraints (from CONCEPT.md)

### 32 Parameters — must not change count
All parameters are in `crates/cell_dt_core/src/fixed_params.rs`.
See `PARAMETERS.md` for full table with values, priors, and sources.

### Core Equation
```
d(Damage)/dt = α × ν(t) × (1 - Π(t)) × S(t) × A(t)
```
- α = 0.0082 (base damage per division)
- ν(t) = division rate (tissue-specific)
- Π(t) = protection factor (declines with age)
- S(t) = SASP hormetic modifier (non-monotonic)
- A(t) = asymmetric division fidelity

### Biological constraints (must be enforced in code)
- `senescent_fraction` ≥ 0.0 (clamp after NK clearance)
- `nfkb` clamp threshold = 0.95 (not 1.0)
- `nk_age_decay` = 0.01 (50% efficiency at age 70, PMID: 12803352)
- `mito_shield` uses exponential decay: `exp(-k*age)`, not linear
- CHIP VAF at age 70 ≈ 0.07 (Jaiswal 2017, PMID: 28792876)
- SASP hormetic response: continuous at `stim_threshold` and `inhib_threshold`
- `telomere_length -= division_rate * telomere_loss_per_division * dt`
- `epigenetic_age += rate * dt + stress_factor * damage * dt`
- CHIP → SASP: `sasp_prod *= (1 + chip.sasp_amplification() * 0.5)` (L1 link)
- Damage → quiescence: `division_rate *= (1 - centriole_damage * 0.5)` (L2 link)
- Fibrosis → regeneration: `regen_factor = 1.0 - fibrosis_level * 0.4` (L3 link)

---

## Build & Run

```bash
# Build workspace
cargo build --workspace

# Run tests (should pass 400+)
cargo test --workspace

# Run main simulation
bash run.sh

# Or directly
cargo run --example basic_simulation --release
```

---

## What NOT to change without user approval
1. CONCEPT.md
2. Parameter values (α, Π₀, P₀, etc.) — any change requires biological justification + PMID
3. Number of parameters (must stay at 32)
4. Validation results claims (R²=0.84)
5. Crate names and workspace structure

---

## DeepSeek Rule
Route ALL non-trivial tasks through DeepSeek API (`~/.aim_env → DEEPSEEK_API_KEY`).
Entry point: `~/Desktop/AIM/llm.py`
Models: `deepseek-chat` (fast) · `deepseek-reasoner` (complex reasoning)

---

## Git Push Rule
- **Private only** unless user explicitly says "push public"
- Private remote: `djabbat/CDATA`
- Public remote: `djabbat/CDATA-public` (sanitized, no patient data)
- Always ask: "private or public?" before every push

---

## Self-Citation (include in any paper from this project)
1. PMID 36583780 — Tkemaladze J. Mol Biol Rep 2023 (core CDATA paper)
2. DOI: https://doi.org/10.5281/zenodo.19174506 (Cell-DT v3.0 code)
