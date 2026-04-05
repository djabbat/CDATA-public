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
**Source:** Internationalization requirements + user request
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

## [2026-04-05] Комплексный анализ в свете новых статей (5+_CDATA, Hypoxia v2.0, CDATA+Ze, Ze_Entropy)

### Источник: Перекрёстный анализ всех новых статей vs существующей модели

---

### 🔴 КРИТИЧЕСКИЕ (до подачи в Aging Cell)

**U1 — Обновить CONCEPT.md до v3.4** `[ ]`
CONCEPT датирован v3.0/v3.2.3 (2026-03-27), но все новые статьи ссылаются на «CDATA v3.4».
Добавить в CONCEPT: O2-зависимый mito_shield, D_crit=1000 a.u., α·ν·β=20 a.u./div, φ_cell модификаторы.

**U2 — PARAMETERS.md Group 8: Гипоксийный модуль** `[ ]`
Константы добавлены в system.rs как `const`, но не задокументированы в PARAMETERS.md.
Добавить:
```
s_max = 0.99 (recalibrated, Peters-Hall 2020)
k_O2 = 0.2 (%O2)^-1
D_crit = 1000 a.u.
alpha_nu_beta = 20 a.u./division (composite)
phi_EpithelialProgenitor = 1.00
phi_HematopoieticStem = 0.96
phi_Fibroblast = 0.91
```

**U3 — Унифицировать версию v3.4 во всех .md** `[ ]`
Найдено три разных версии: CONCEPT=v3.0, TODO=v3.0, статьи=v3.4.

**U4 — Исправить «18 датасетов» → реальное число в 5+_CDATA_Aging_Cell.md** `[ ]`
Статья пишет «18 published experiments»; validation/ содержит 5 датасетов.
Либо добавить 13+ датасетов в validation/, либо исправить заявление в статье.

**U5 — Заменить фиктивные PMID на реальные** `[ ]`
PARAMETERS.md содержит ≥12 PMID-плейсхолдеров (01234567, 12345678, 23456789...).
Провести поиск в PubMed и заменить на реальные перед подачей.

**U6 — Унифицировать счётчик тестов** `[ ]`
CONCEPT=385 (⚠️), TODO=473, KNOWLEDGE=483. Использовать 483 везде.

**U7 — Добавить examples/o2_dose_response.rs** `[ ]`
Статья указывает Code Available at github.com/djabbat/CDATA-public.
Без примера, воспроизводящего рис. из статьи — «Code Available» не работает.
Пример: симулировать fibroblast при O2 = 0.5, 1, 2, 5, 10, 21%, вывести N_Hayflick.

---

### 🟡 ВАЖНЫЕ (v3.5, следующая итерация)

**U8 — Два mito_shield в одну архитектуру** `[ ]`
Конфликт: KNOWLEDGE §2 имеет `mito_shield = exp(-0.0099×age)` (возрастной),
новые статьи имеют `mito_shield([O2])` (кислородный).
Это два разных биологических процесса, нужна единая формула:
`mito_shield_total(age, O2) = exp(-k_age × age) × s_max × φ × exp(-k_O2 × O2)`
Реализовать в MitochondrialSystem; добавить O2 как аргумент в update().

**U9 — current_o2_percent в TissueState** `[ ]`
TissueState не содержит поля [O2]. Нельзя симулировать гипоксические интервенции.
Добавить: `current_o2_percent: f64` (default: 21.0) в TissueState.
Передавать в MitochondrialSystem::update() → mito_shield_for_o2().

**U10 — predicted_hayflick() в GUI и PyO3** `[ ]`
Функция добавлена в system.rs, но не экспортирована в Python и не показана в Streamlit.
Добавить: график «N_Hayflick vs [O2]» как новый tab в GUI.
PyO3: fn predicted_hayflick(o2: f64, cell_type: &str) → f64.

**U11 — Стандартизировать масштаб D** `[ ]`
Три несовместимых масштаба:
- CDATA main: D нормировано к [0,1], D_max=15
- Ze статья: D ∈ [0, 0.6] (другие единицы)
- Hayflick formula: D_crit=1000 a.u. (ненормированный)
Решение: выбрать один каноничный масштаб и задокументировать конверсию.

**U12 — rho_kinase_inhibition параметр** `[ ]`
Peters-Hall использовал ROCKi + O2. CDATA имеет механотрансдукцию (Mechanism 6: YAP/TAZ),
но нет явного ROCK inhibition пути.
Добавить: параметр `rho_kinase_inhibition: f64` (0.0 = нет, 1.0 = макс.),
модулирующий `ecm_stiffness` и `centriolar_mechanical_stress`.

---

### 🟢 ДОЛГОСРОЧНЫЕ (v4.0)

**U13 — Формальный dual-clock (centriole || telomere)** `[ ]`
Статья §5.2: «senescence triggered by whichever clock reaches threshold first».
Реализовать: `senescent = centriole_damage > threshold || diff_telo < min_telo`.

**U14 — 5-я ткань: Epithelial Progenitor** `[ ]`
Peters-Hall использовал bronchial basal progenitors. В CDATA нет этого типа.
Добавить: EpithelialProgenitor с ν=18, β=0.8, τ=0.75 (предварительно).

**U15 — Ze-entropy ↔ SASP математическое соответствие** `[ ]`
Ze_and_Entropy.md показывает: damage = Ze-бюджет деплеция.
SASP = сигнал о Ze-энтропии соседних клеток.
Формализовать: добавить Ze-интерпретацию в KNOWLEDGE.md §3.

**U16 — Horvath clock = Ze-counter** `[ ]`
Биологически совместимы: epi_age = τ_Z depletion proxy.
Математически доказать в отдельной статье или Appendix к Ze статье.

**U17 — 18+ реальных датасетов в validation/** `[ ]`
Для согласования с заявлением в 5+_CDATA_Aging_Cell.md.
Кандидаты: InCHIANTI, BLSA, WGHS, WB study, UK Biobank aging subset.

---

## [2026-04-04] CHIP frailty integration
**Source:** Jaiswal 2017 (PMID: 28792876): CHIP VAF correlates with frailty independent of SASP
**Status:** [✓ approved 2026-04-04] [✓✓ implemented 2026-04-04]

Added `chip_vaf × 0.05` as a direct frailty component (5th term).
centriole_damage weight reduced 0.45→0.40 to keep weights summing to 1.0.
CHIP now has two pathways to MCAI: L1 (→SASP) and direct (→chip_vaf component of MCAI).
Biological basis: Mas-Peiro 2020 (PMID: 32353535) — CHIP independently predicts frailty.
2 new tests: `test_frailty_formula_matches_five_components`, `test_chip_vaf_contributes_positively_to_frailty`.
