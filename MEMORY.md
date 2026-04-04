# MEMORY.md — CDATA: Project Memory

## Key Decisions & Architecture Choices

### 2026-03-27 — CDATA v3.0 Finalized
- **Status:** Code complete. 483/483 tests passing. R²=0.84 (independent validation).
- **32 parameters** — reduced from 120 over 5 peer review rounds. Must not change count.
- **8 crates** — cell_dt_core, 4 modules, validation, gui, python bindings.
- **Repos:** private `djabbat/CDATA` · public `djabbat/CDATA-public` (sanitized)

### Why CDATA focuses on centrioles (not telomeres / epigenetic drift)
- Centriolar damage is upstream: it explains WHY stem cells slow their divisions with age
- Telomere dynamics and epigenetic drift are downstream outputs modeled as biomarkers
- Mechanistic uniqueness is the project's competitive advantage (R² > competitors)

### 32 Parameters — Hard Constraint
- Reduced from 120 via MCMC sensitivity analysis + peer review (5 rounds)
- Adding parameters requires peer-review justification + new PMID
- alpha fixed at 0.0082 (collinear with tau_protection in MCMC — r=0.858)
- hsc_nu and dnmt3a_fitness fixed (insensitive: ΔR²≈0 at ±20%)

### Frailty Index — 5 Components (as of 2026-04-04)
- 0.40 × centriole_damage
- 0.25 × SASP
- 0.20 × (1 − stem_cell_pool)
- 0.10 × (1 − differentiated_telomere_length)
- 0.05 × chip_vaf  ← added 2026-04-04 (CHIP frailty integration)
- **Stem cell telomere stays at 1.0** (telomerase) — not in frailty

### R² Clarification
- **0.84** = independent validation (use in all papers and grant materials)
- **0.98** = scale-anchored training R² (MCMC calibration cohort 20–50 yr) — do NOT cite as main result

### Frailty Weight History
| Date | Weights |
|------|---------|
| Round 7 | 0.45/0.25/0.20/0.10 (4 terms) |
| 2026-04-04 | 0.40/0.25/0.20/0.10/0.05 (5 terms, CHIP added) |

### Peer Review History
- Round 1–5: Parameter reduction (120 → 32)
- Round 6: 5 bugs (B1-B5), 3 missing equations (M1-M3), 3 bio links (L1-L3), 4 calibration fixes
- Round 7: 5 fixes (telomere dynamics, epi-age, ROS, MCMC insensitive params, alpha collinearity)
- 2026-04-04: CHIP frailty integration (direct pathway)

---

## Version History

| Version | Date | Status |
|---------|------|--------|
| 1.0 | 2023 | PMID: 36583780 — core paper |
| 2.0 | 2025 | Python prototype |
| 3.0 | 2026-03-27 | Rust simulator, 32 params, R²=0.84, 483 tests |
| 3.0.1 | 2026-04-04 | CHIP frailty integration (5th frailty term) |
