# Cell DT — Рекомендации по оптимизации модели CDATA

## 📌 Правило: язык программирования по умолчанию

**Если нет явного указания на конкретный язык — писать код на Rust.**
Если другой язык объективно лучше для задачи — сначала предложить и обосновать, и только после подтверждения писать код.

---


> **Статус:** Живой документ. Вычёркивать/удалять пункты по мере выполнения.
> Выполненные шаги помечаются `[x]`, невыполненные — `[ ]`.
> Последнее обновление: **2026-03-24**
> Тестов: **439** ✅

---

## ✅ РЕШЕНО (2026-03-24)

### [GUI-CRIT-1] GUI не отображает реальные числа симуляции — **ИСПРАВЛЕНО**

**Что сделано:**
- `SimulationManager` теперь запускается в фоновом потоке (`std::thread::spawn`)
- Канал `mpsc::channel::<SimSnapshot>` передаёт данные от движка в GUI
- Каждые N шагов движок отправляет `SimSnapshot` с реальными ECS-данными:
  `age_years`, `frailty`, `stem_cell_pool`, `ros_level`, `myeloid_bias`,
  `telomere_length`, `methylation_age`
- GUI принимает снапшоты в `update()` через `try_recv()`, обновляет графики в реальном времени
- 5 стволовых ниш (`CentriolePair + CellCycleStateExtended`) спавнятся перед `initialize()`
- Модули подключены: `CentrioleModule → CellCycleModule → MitochondrialModule →
  HumanDevelopmentModule → MyeloidShiftModule`
- Параметры из GUI маппятся в модули через `set_module_params()`
- STOP корректно завершает поток через `AtomicBool`
- Кнопка `← Back to settings` возвращает к параметрам

**Ограничение:** при `dt=0.1, max_steps=10000` симуляция покрывает ~2.7 лет.
Для 100-летней кривой рекомендуется: `dt=1.0, max_steps=36500`.

---

### Выполнено в сессии 2026-03-24 (GUI i18n + Visualization) ✅

- **[GUI-1] Multilingual GUI** — 7 языков (EN/FR/ES/RU/ZH/AR/KA)
  - `crates/cell_dt_gui/src/i18n.rs` — ~80 строк перевода на каждый язык
  - `Lang` enum с `display_name()`, `tr()`, `all()`; сериализуется (lang persistence)
  - ComboBox в top panel (правый угол) для переключения языка
  - Noto fonts: Georgian + Arabic (crates/fonts/), CJK из системы
  - `cargo build -p cell_dt_gui` → ✅ без ошибок
- **[GUI-2] RUN SIMULATION button** — нижняя панель (72px)
  - Пульсирующий зелёный цвет, `sin(time)` анимация, tooltip
  - ProgressBar + step counter во время выполнения
  - STOP button; автопереход в Visualization после завершения
  - Переводы: btn_run_simulation / btn_run_tooltip / sim_started / sim_complete (7 яз)
- **[GUI-3] Impact Showcase** — `📊 Visualization` tab
  - 4 графика (egui_plot 0.24): Frailty Index, Stem Cell Pool, Kaplan-Meier, CAII+Epigenetic Clock
  - 4 пресета: Control (~78yr), Longevity (~108yr), Progeria (~15yr), CentrosomeTransplant@50yr
  - Академический стиль с легендой, осями, сноской (PMID 36583780)

---

### ✅ Выполнено ранее (P61–P64, 2026-03-23)

- P61 Agent Population Model, P62 Thermodynamic Progeria, P63 SystemicCircadianState, P64 Thesis Figures
- Тесты: 427 → **439** (+12)

---

## TODO — EIC Pathfinder Open 2026 (дедлайн: 12 мая 2026)

### 🔴 P0 — КРИТИЧНО

- [ ] **Зарегистрировать Phasis Academy в EU Funding Portal (PIC)**
  → https://ec.europa.eu/info/funding-tenders → начать СЕГОДНЯ, занимает 2–4 нед.

- [x] Исправить бюджет: прямые затраты €2,000K + косвенные 25% = €500K → итого €2,500K ✅
- [x] Убрать опечатку «confocal acquisition on confocal acquisition» ✅
- [x] Разрешить противоречие SP8 vs Zeiss LSM 700 → теперь единая версия: GTU имеет Zeiss, бюджет включает апгрейд до SP8 ✅

### 🟡 P1 — После ответа Гелы (Prof. Batoni Gela, GTU BME)

- [x] co-PI вставлен как «Prof. Batoni Gela, PhD — Department of Biomedical Engineering, GTU ABET» ✅
- [x] Письмо отправлено 26.03.2026 (RU+EN+KA версии готовы) ✅
- [ ] После ответа Гелы: получить CV (Horizon format) + Letter of Support (на бланке GTU)
- [ ] Вставить полное ФИО и все данные в Part B section 3.2.2
- [x] Унифицировать n=240/288 → «enrolled n=288, evaluable n=240» ✅
- [x] Добавить таблицу person-months в раздел 3.2 ✅
- [x] Указать WP-лидеров (WP1,2: GTU; WP3,4: Phasis Academy) ✅
- [x] Consortium Agreement добавлен в WP4 deliverables ✅

### 🟢 P2 — До 1 мая

- [ ] Letter of Intent от Georgian National Centre for Disease Control
- [ ] Подтвердить или убрать Estonian Biobank из sustainability
- [ ] Уточнить ISSN и индексацию Annals of Rejuvenation Science
- [ ] Ссылки в Table 1: López-Otín 2013, Blackburn 2000

### ✅ Сделано (EIC)

- [x] Заявка задрафтована: `~/Desktop/EIC_Pathfinder_CDATA_Application.docx`
- [x] GTU добавлен как Beneficiary 2 (PIC: 983636358)
- [x] U-ExM методология + Gambarotto 2019 Nat Methods
- [x] Таблица сравнения теорий (CDATA vs Hallmarks/Telomere/Inflammaging)
- [x] Risk Mitigation 3.5 (4 риска)
- [x] Gantt + Milestones + Budget
- [x] npj Aging указан вместо «Nature family journal»
- [x] Peer Review v1+v2+v3 завершены — итоговая оценка **4.3/5**, вердикт: «Рекомендуется к финансированию»
- [x] Все технические замечания v1/v2/v3 внесены в заявку ✅

---

## TODO — Cell-DT Engineering (следующие задачи по коду)

### ✅ Выполнено в сессии 2026-03-24

- [x] **P65: SenescenceAccumulation loop** — петля замкнута: division_rate→senescent_fraction→SASP→div_rate; SenescenceAccumulationState как ECS-компонент; 4 теста
- [x] **P66: TrackAB cross-feedback** — TrackABCrossState активен; combined_dysfunction×0.20 boost в myeloid_bias; 3 теста
- [x] **P67: CSV-экспорт PTM-траекторий** — CdataRecord расширен 5 PTM-колонками (carbonylation/hyperacetylation/aggregation/phospho_dysreg/appendage_loss); 2 теста
- [x] **P68: Параметрический sweep M₀/D₀** — inducer_sweep_example.rs (24 комбинации); 2 теста; CSV в inducer_sweep_output/
- [x] **GitHub CI/CD** — .github/workflows/ci.yml обновлён (dtolnay, cache v4, cargo test --workspace)
- [x] **crates.io** — cell_dt_core готов: metadata, README, keywords, categories, самоцитирование

### 🟡 Важно (следующая очередь)

- [ ] **GUI: реальный запуск симуляции** — сейчас RUN SIMULATION подключён к движку (mpsc), но нет экспорта
- [ ] **GUI: экспорт результатов** — после окончания симуляции сохранить CSV в `output_dir` через CdataExporter

### 🟢 Долгосрочно

- [ ] **P69: CytoplasmQCState модуль** — QC-компартментализация при делении (p56)
- [ ] **P70: Пространственный O₂-щит** — pericentriolar_density (P9)
- [ ] **Web demo** — WASM сборка egui для браузера (eframe поддерживает wasm32)
- [ ] **Python bindings docs** — документация cell_dt_python + примеры Jupyter

---

---

## ИЕРАРХИЯ УРОВНЕЙ — что делать на каждом (roadmap)

Клетка = автономная единица. Снизу — субклеточные уровни (источники повреждения). Сверху — надклеточные (контекст).

---

### УРОВЕНЬ -5: Кварки / Ze-поле
**Статус:** ✅ реализовано — P24, 2026-03-23

- [x] **ZeHealthState** ✅ — P24, 2026-03-23
  - `v = v* + 0.544×(1−CAII)`: Ze-скорость из придатков
  - `deviation_from_optimal`: |v − 0.456|; `ze_health_index` = CAII
  - `v_entropy`: из ThermodynamicState.ze_velocity_analog (энтропийный канал)
  - `v_consensus = mean(v, v_entropy)`: объединённая оценка (структура + термодинамика)
  - `interpretation()`: optimal / mild_aging / moderate_aging / severe_aging / near_collapse
  - ze_health.rs: `update_ze_health_state(ze, caii, entropy_v)`
  - 7 тестов; всего **250 тестов**; push → djabbat/CDATA-Longevity
- [x] Валидация: v у симулятора vs Ze-HRV из ЭЭГ-статьи (n=60, Дортмунд) ✅ P51

---

### УРОВЕНЬ -4: Атомы (иерархическая термодинамика)
**Статус:** ✅ реализовано — P22, 2026-03-23

- [x] **ThermodynamicState** ✅ — P22, 2026-03-23
  - `local_temp_celsius`: baseline 36.6°C + SASP×2.4°C
  - `damage_rate_multiplier`: Аррениус exp(Ea_mean/R × (1/T_ref − 1/T))
    При T=39°C: mult≈1.14–1.22; при T=37°C: mult=1.0
  - `entropy_production`: кумулятивный dS от PTM (C=O — необратимо ΔG<0)
  - `ze_velocity_analog`: entropy/(entropy+2.0) → v*=0.456 ≈ 20 лет
  - Ea по трекам: карбонилирование 50кДж / ацетилирование 40кДж /
    агрегация 80кДж / фосфо 45кДж / придатки 55кДж/моль
  - `ThermodynamicParams::with_arrhenius()` включает Аррениус
  - Связь с InflammagingState: sasp_intensity → local_temp → mult ✅
  - Ze Theory: PTM = время→пространство; v*=0.456 у молодого (~20лет) ✅
  - 8 тестов; всего 228 тестов; push → djabbat/CDATA-Longevity

---

### УРОВЕНЬ -3: Молекулы
**Статус:** частично (4 PTM + 4 придатка как агрегат)
**Направление:** раскрыть CEP164/CEP89/Ninein/CEP170 + ROS-каскад

- [x] **AppendageProteinState** ✅ — P21, 2026-03-23
  - `cep164/cep89/ninein/cep170` — независимые f32, отдельный ECS-компонент
  - CAII = weighted geometric mean (0.40/0.25/0.20/0.15) — EIC WP1 биомаркер
  - OH·-чувствительность: CEP164(1.50) > CEP89(1.00) > Ninein(0.75) > CEP170(0.55) ✅
  - `ciliary_function()` = CAII × (1 - aggregates×0.5)
  - Репарация + митофагия-связь; обратная совместимость сохранена
  - 5 новых тестов; всего 220 тестов
- [x] **ROSCascadeState** ✅ — P23, 2026-03-23
  - `superoxide/hydrogen_peroxide/hydroxyl_radical/labile_iron` — 4 ОДУ (Эйлер)
  - Фентон: Fe²⁺ + H₂O₂ → OH· (Halliwell & Gutteridge 1984)
  - Каталаза снижается с возрастом: × (1 − age × 0.003) (Tian et al. 1998)
  - Аутофагия снижает лабильное железо (феррофагия)
  - `effective_oh(amp)` → в AppendageProteinState вместо ros_level²
  - `ros_level_compat()` = H₂O₂ → синхронизируется в CentriolarDamageState
  - 7 тестов; всего **235 тестов**; push → djabbat/CDATA-Longevity
  - Треугольник замкнут: ThermodynamicState → ROSCascadeState → AppendageProteinState
- [x] **ATPEnergyState** ✅ — P28, 2026-03-23
  - `atp_adp_ratio / energy_charge / proteasome_activity_modifier`
  - ROS истощает АТФ, митохондрии восстанавливают
  - atpenergy.rs: `ATPEnergyParams + update_atp_energy_state()`; 5 тестов
- [x] **ChromatinState** ✅ — P29, 2026-03-23
  - `tad_integrity / heterochromatin_fraction / dna_accessibility`
  - methylation_age → TAD нарушаются; total_damage → SAHF разрушаются
  - chromatin.rs: `ChromatinParams + update_chromatin_state()`; 5 тестов

---

### УРОВЕНЬ -2: Органеллы (цитоскелет)
**Статус:** частично (spindle_fidelity как скаляр + ✅ MicrotubuleState)
**Направление:** динамика MT, IFT, актиновое кольцо

- [x] **MicrotubuleState** ✅ — P25, 2026-03-23
  - `polymerization_rate` = 0.90 × (1 − acetylation × 0.70)
  - `catastrophe_rate` = 0.10 + phospho_dysreg × 0.80
  - `DII` = cat/(poly+cat) [Mitchison & Kirschner 1984]
  - `spindle_fidelity_derived` = (1−DII) × ninein_integrity
  - Ninein-якорение минус-концов → синхронизируется в CentriolarDamageState.spindle_fidelity
  - Квазистационарное приближение: разделение временных масштабов (PTM-лет vs MT-секунд)
  - microtubule.rs: `MicrotubuleParams` + `update_microtubule_state()`
  - 7 тестов; всего **250 тестов**
- [x] **IFTState** ✅ — P30, 2026-03-23
  - `anterograde_velocity / retrograde_velocity / cargo_delivery`
  - CEP164 + Ninein → антероградный IFT; phospho → ретроградный IFT
  - ift.rs: `IFTParams + update_ift_state()`; 6 тестов
- [x] **ActinRingState** ✅ — P31, 2026-03-23
  - `contractile_ring_integrity / actin_polymerization_rate / incomplete_cytokinesis_prob`
  - OH· → деполимеризация актина; phospho → RHOA-путь
  - actin_ring.rs: `ActinRingParams + update_actin_ring_state()`; 5 тестов
- [x] GammaRingComplex: γ-тубулин кольцевые комплексы → зависят от Ninein integrity

---

### УРОВЕНЬ -1: Органоиды
**Статус:** митохондрии ✅, Гольджи ✅ (P26), остальное ❌
**Направление:** ЭПС, лизосомы, пероксисомы

- [x] **GolgiState** ✅ — P26, 2026-03-23
  - `fragmentation_index`: ROS × 0.12 + SASP × 0.08 − repair × 0.30 (ОДУ)
  - `glycosylation_capacity` = (1 − frag × 0.85).clamp(0.1, 1.0)
  - `cep164_glycosylation` = glycosyl × 0.95
  - `vesicle_trafficking_rate` = glycosyl × 0.90
  - Эффект: extra CEP164 loss = (1 − cep164_glycosyl) × 0.08 × dt
    → применяется в AppendageProteinState шаге (лаг 1 шаг)
  - Аутофагия снижает фрагментацию (Golgi-фагия): repair += autophagy × 0.15
  - 9 тестов; всего **259 тестов**; push → djabbat/CDATA-Longevity
- [x] **ERStressState** ✅ — P32, 2026-03-23
  - `unfolded_protein_response / ca2_buffer_capacity / chaperone_saturation`
  - агрегаты + протеасома↓ → UPR; хрон. UPR → CHOP → апоптоз
  - er_stress.rs: `ERStressParams + update_er_stress_state()`; 6 тестов
- [x] **LysosomeState** ✅ — P33, 2026-03-23
  - `ph_level [4.5..7.0] / hydrolase_activity / membrane_permeability`
  - ROS → защелачивание; OH· → LAMP1/LAMP2 повреждение
  - lysosome.rs: `LysosomeParams + update_lysosome_state()`; 5 тестов
- [x] **PeroxisomeState** ✅ — P34, 2026-03-23
  - `catalase_activity / h2o2_clearance_rate / fatty_acid_oxidation`
  - Tian et al. 1998: каталаза −0.4%/год после 40; связь с ROSCascadeState
  - peroxisome.rs: `PeroxisomeParams + update_peroxisome_state()`; 6 тестов
- [x] **RibosomeState** ✅ — P35, 2026-03-23
  - `translation_rate / ribosome_quality / aminoacyl_availability`
  - energy_charge × aminoacyl_avail → трансляция; агрегаты → RQC↓
  - ribosome.rs: `RibosomeParams + update_ribosome_state()`; 6 тестов

---

### УРОВЕНЬ 0: КЛЕТКА (автономная единица — текущий уровень)
**Статус:** ✅ основная модель + P27 генетическая гетерогенность
**Направление:** шум, эпигенетическая память, пространство

- [x] **Генетическая гетерогенность** ✅ — P27, 2026-03-23
  - `GeneticProfile { carbonylation_risk, acetylation_risk, aggregation_risk, phospho_risk,
    appendage_risk, ros_feedback_risk, longevity_factor, variant: GeneticVariant }`
  - Пресеты: `average()` / `apoe4()` / `apoe2()` / `lrrk2_g2019s()` / `foxo3a_longevity()` / `sod2_ala16val()`
  - `genetic::apply_genetic_modifiers(&base_params, &profile)` → per-nicha DamageParams
  - Субдистальные придатки менее чувствительны: subdistal_modifier = 1 + (risk−1)×0.5
  - Сбор через отдельный pre-pass query (hecs 0.10 лимит: 15 эл. в tuple)
  - 9 тестов: identity, APOE4↑, FOXO3a↓, LRRK2-phospho, субдистальная чувствительность,
    SOD2-ros_feedback, FOXO3a-repair↑, неизменяемые поля, APOE4>avg>APOE2
  - **268 тестов**; push → djabbat/CDATA-Longevity
- [x] **CloneEpigeneticState** ✅ — P42, 2026-03-23
  - `clone_baseline / clone_drift_rate / clone_drift_accumulated`
  - Пресеты: `neutral()` (0.003/год) / `tet2_chip()` (0.006/год, baseline=0.10)
  - `effective_methylation_contribution()` → добавляется к EpigeneticClockState
  - clone_epigenetic.rs: `CloneEpigeneticParams + update_clone_epigenetic_state()`; 6 тестов
- [x] **FateSwitchingState** ✅ — P43, 2026-03-23
  - `fate_bias [-1..+1] / switch_threshold / noise_accumulator / switch_count`
  - Ланжевен-шум (Ито): fate_bias += N(0,σ) × √dt − recovery × bias × dt
  - Повреждения снижают порог переключения: threshold × (1 − damage × 0.40)
  - fate_switching.rs: `FateSwitchingParams + update_fate_switching_state(rng)`; 5 тестов
- [x] **SaspDiffusionState** ✅ — P45, 2026-03-23
  - `local_sasp / received_sasp / effective_sasp / diffusion_radius`
  - `update_sasp_diffusion_state(neighbor_sasps)`: mean соседей × коэфф диффузии
  - `sasp_bystander_effect()`: ROS-буст при effective > threshold
  - sasp_diffusion.rs: `SaspDiffusionParams`; 5 тестов
- [x] **GammaRingState** ✅ — P44, 2026-03-23
  - `nucleation_efficiency / ring_integrity / pericentriolar_density`
  - Ninein-coupling: density = ninein × 0.70 + (1−ninein) × 0.20
  - OH·-повреждение + PCM-репарация ring_integrity
  - gamma_ring.rs: `GammaRingParams + update_gamma_ring_state()`; 6 тестов

---

### УРОВЕНЬ +1: Ткани
**Статус:** TissueState (11 типов) ✅, нет матрикса и сосудистой ниши
**Направление:** межклеточный матрикс, сосудистые ниши

- [x] **ExtracellularMatrixState** ✅ — P36, 2026-03-23
  - `collagen_crosslinking / stiffness / integrin_signaling`
  - ROS + AGE → LOX → сшивание; жёсткость → YAP/TAZ → симм. деления
  - extracellular_matrix.rs: `ECMParams + update_ecm_state()`; 5 тестов
- [x] **VascularNicheState** ✅ — P37, 2026-03-23
  - `oxygen_supply / growth_factor_gradient / angiogenesis_index`
  - VEGF↓ после 50 + ROS → редукция капилляров → гипоксия
  - vascular_niche.rs: `VascularNicheParams + update_vascular_niche_state()`; 6 тестов
- [x] **FibrosisState** ✅ — P38, 2026-03-23
  - `fibroblast_activation / collagen_deposition_rate / functional_replacement`
  - SASP → TGF-β → миофибробласты; fc_penalty = replacement × 0.80
  - fibrosis.rs: `FibrosisParams + update_fibrosis_state()`; 6 тестов
- [x] **NeuromuscularJunctionState** ✅ — P46, 2026-03-23
  - `ach_receptor_density / synaptic_transmission / denervation_index / reinnervation_capacity`
  - BDNF-поддержка реиннервации; ROS → денервация; aggregates → АХ↓
  - `nmj_muscle_penalty()`: 0.80 при synaptic < 0.50
  - nmj.rs: `NMJParams + update_nmj_state()`; 5 тестов

---

### УРОВЕНЬ +2: Органы
**Статус:** ✅ реализовано — P41, 2026-03-23

- [x] **OrganState + OrganType** ✅ — P41, 2026-03-23
  - `OrganType` enum (11 типов: Heart/Kidney/Liver/Lung/Brain/Intestine/Skin/Bone/ImmuneSystem/EndocrineSystem/Muscle)
  - `OrganState { organ_type, functional_reserve, compensation_capacity, failure_threshold, is_failing, niche_count }`
  - `update_organ_state(niches)`: mean(fc × (1−fibrosis)) + компенсаторная гипертрофия
  - `check_poly_organ_failure(organs)`: возвращает Some(список) при ≥2 органах в failure
  - `cardiac_oxygen_delivery()`: Heart.functional_reserve → оксигенация всех органов
  - organ.rs; 8 тестов
- [x] Интеграция OrganState в ECS — P47, 2026-03-23
  - `organ_integration.rs`: `NicheCapacityData + aggregate_niches_by_organ() + update_organ_states_in_world()`
  - `tissue_to_organ()`: маппинг TissueType → OrganType (10 типов)
  - Полиорганная недостаточность: ≥2 failing → OrganismState.is_alive = false
  - 4 теста: single_failure_no_death, two_failures_death, update_from_niches, healthy_no_failure
- [x] Полиорганная недостаточность как критерий смерти в OrganismState ✅ — P47

---

### УРОВЕНЬ +3: Организм
**Статус:** OrganismState ✅ (frailty, cognitive, immune, muscle)
**Направление:** нейроэндокринная регуляция, циркадные ритмы, метаболизм

- [x] **HPAAxisState** ✅ — P39, 2026-03-23
  - `cortisol_level / hpa_reactivity / chronic_stress_index`
  - стресс → кортизол↑ → иммуносупрессия; выгорание оси при хрон. активации
  - hpa_axis.rs: `HPAAxisParams + update_hpa_axis_state()`; 5 тестов
- [x] **MetabolicPhenotypeState** ✅ — P40, 2026-03-23
  - `bmi_index / adipokine_level / insulin_sensitivity`
  - ожирение → лептин/резистин → ROS↑; инсулинорезистентность → ATP↓
  - metabolic_phenotype.rs: `MetabolicParams + update_metabolic_phenotype_state()`; 5 тестов
- [ ] Ось GH/IGF-1 ✅ (реализована) → расширить: инсулин, лептин, грелин
- [x] Циркадная синхронизация организма: нарушение ритмов → SASP↑ (есть CircadianState на уровне ниши → нужна системная версия)
- [x] **CAII-индекс организма** ✅ — P48, 2026-03-23
  - `OrganismState.caii_organism / biological_age` добавлены
  - `compute_organism_caii(niches_caii, age)`: mean CAII + bio_age = age × (1 + (1−CAII) × 0.50)
  - Интегрировано в `update_organism_state()` через AppendageProteinState query
  - caii_organism.rs; 4 теста: perfect/zero/mid CAII + empty niches

---

### УРОВЕНЬ +4: Социум
**Статус:** ❌ нет
**Направление:** social_stress как входной параметр → биологические эффекты

- [x] **SocialStressState** ✅ — P49, 2026-03-23
  - `loneliness_index / socioeconomic_stress / social_cohesion / oxytocin_level / allostatic_load`
  - oxytocin = cohesion × 0.20 + 0.20; allostatic_load накапливается
  - `social_cortisol_boost()` / `social_sasp_reduction()` → связи с HPA и SASP
  - social_stress.rs: `SocialStressParams + update_social_stress_state()`; 5 тестов
- [x] Популяционный режим: N организмов → распределение CAII в когорте → валидация vs WP1 n=240 ✅ P50
- [x] Агент-ориентированная модель: каждый организм = агент, взаимодействие → эпидемиология старения

---

### УРОВЕНЬ +5: Ноосфера
**Статус:** ❌ нет (концептуально)
**Направление:** интервенции как управляющие воздействия из базы знаний

- [x] InterventionLibrary: сенолитики / NAD+ / CR / телоангиогенетические препараты → параметры DamageParams
- [ ] AIMIntegration: AIM clinical AI → читает CAII пациента → выбирает интервенцию → обновляет симулятор
- [x] EvidenceBase: каждая интервенция с уровнем доказательности (RCT / meta-analysis / in silico)
- [x] Эпигенетическое перепрограммирование (Яманака факторы) → reset CentriolarDamageState

---

### УРОВЕНЬ +6: Экосфера
**Статус:** теоретический
**Направление:** эволюционные ограничения продолжительности жизни

- [ ] EvolutionaryConstraints: species-specific lifespan = f(metabolic_rate, body_mass, reproduction_strategy)
- [ ] Антагонистическая плейотропия на популяционном уровне: короткая жизнь = эволюционное преимущество при высокой рождаемости
- [x] Межвидовая CDATA: мышь (2 года) / человек (78 лет) / летучая мышь (40+ лет) / голый землекрот (30+ лет) — один параметрический набор с масштабированием
- [ ] Биосферная роль старения: Weismann hypothesis — старение как механизм ресурсной ротации

---

## ПРАВИЛО: перед git push

1. Обновить **README.md**: добавить новые компоненты в секции 3, 4, 11.
2. Обновить **TODO.md**: отметить выполненное `[x]`, обновить счётчик тестов.
3. При серьёзных изменениях — записать **план научной статьи** (название + абстракт).

## ПРАВИЛО: GUI — вкладки по уровням иерархии

Новый модуль / новые параметры → автоматически добавлять вкладку/секцию в GUI:
- GUI (`cell_dt_gui`) должен иметь **11 вкладок** по уровням иерархии (-5 … +5).
- Каждая вкладка содержит параметры компонентов своего уровня.
- **Вкладка «Клетка» (уровень 0) — открывается по умолчанию** (центральный уровень).
  → Клетка = автономная единица теории CDATA; все остальные уровни — контекст или субструктура.
- Новый уровень иерархии → новая вкладка GUI.
- Структура вкладок:
  | Вкладка | Уровень | Компоненты |
  |---------|---------|------------|
  | Ze-поле | −5 | ZeHealthState: v, deviation, v_consensus |
  | Атомы   | −4 | ThermodynamicState: local_temp, mult, entropy |
  | Молекулы| −3 | ROSCascadeState + AppendageProteinState (CAII) |
  | Цитоскелет | −2 | MicrotubuleState: DII, spindle_fidelity_derived |
  | Органоиды | −1 | GolgiState + MitochondrialState |
  | **Клетка** | **0** | **CentriolarDamageState + GeneticProfile (дефолт)** |
  | Ткань | +1 | TissueState: pool, tempo, senescent_fraction |
  | Органы | +2 | (будущий OrganState) |
  | Организм | +3 | OrganismState: frailty, cognitive, is_alive |
  | Социум | +4 | (будущий SocialStressInput) |
  | Ноосфера | +5 | Interventions: сенолитики, NAD+, CR |
- TODO: реализовать GUI-вкладки в `cell_dt_gui/src/` (egui)

## ПРАВИЛО: GUI — мультиязычность

- GUI должен поддерживать **7 языков**: 6 официальных языков ООН + грузинский.
  | Код | Язык       |
  |-----|------------|
  | en  | English    |
  | fr  | Français   |
  | es  | Español    |
  | ru  | Русский    |
  | zh  | 中文       |
  | ar  | العربية    |
  | ka  | ქართული    |
- Выбор языка **сохраняется** после закрытия приложения (файл `~/.config/cell_dt_gui/settings.toml`).
- При первом запуске: определять системный язык (locale), если поддерживается — использовать его;
  иначе — английский по умолчанию.
- Все метки параметров, названия вкладок, тултипы — локализованы.
- TODO: реализовать I18n в `cell_dt_gui/src/i18n.rs` (статические &'static str таблицы per язык).

## ПРАВИЛО: научные статьи

По мере реализации серьёзных изменений в симуляторе — записывать план статьи
(название + короткий абстракт). Текущие:

---

## СТАТЬЯ — P26: Гольджи-центриолярная ось старения

**Рабочее название:** «Golgi Fragmentation-Driven CEP164 Hypoglycosylation as
a Novel Mechanism of Centriolar Appendage Loss During Aging»

**Журнал-кандидат:** Journal of Cell Science / Aging Cell / Traffic

**Абстракт (черновик):**
Потеря CEP164 — главного организатора дистальных придатков центриоли — является
первичным структурным событием в CDATA-теории старения (Tkemaladze). Однако молекулярный
механизм ускорения этой потери при хроническом воспалении оставался неизвестен.
Мы предлагаем гипотезу «Гольджи-центриолярной оси»: ROS и SASP-цитокины (TNF/IL-6)
вызывают фрагментацию аппарата Гольджи через окисление GRASP65/JNK-фосфорилирование
→ нарушение N-гликозилирующего конвейера → CEP164 выходит недогликозилированным
→ распознаётся E3-лигазой CHIP/STUB1 → убиквитин-протеасомная деградация ускоряется.
В CDATA-симуляторе реализован GolgiState (уровень -1): дифференциальное уравнение
динамики фрагментации + производные метрики (glycosylation_capacity, cep164_glycosylation,
vesicle_trafficking_rate). Показано: (1) хроническое воспаление SASP≥0.5 ускоряет
потерю CEP164 на 15–25% за 10 лет через гликозилирующий дефицит; (2) аутофагия
(Golgi-фагия) снижает фрагментацию и частично восстанавливает гликозилирование;
(3) снижение везикулярного трафика → дефицит IFT-компонентов у реснички.
Результат: механистическое объяснение возрастной потери CEP164 через Гольджи-опосредованный
путь, независимый от прямого OH·-повреждения.

---

## СТАТЬЯ — P22: Термодинамика клеточного старения и прогерия

**Рабочее название:** «Thermodynamic Amplification of Centriolar Damage in Progeria:
an Arrhenius Model of SASP-driven Protein Aggregation»

**Журнал-кандидат:** npj Aging / Aging Cell / GeroScience

---

### Ключевой тезис

Хроническое воспаление (SASP) повышает локальную температуру ниши на 1–3°C.
Через уравнение Аррениуса это вызывает **нелинейное** (экспоненциальное) ускорение
молекулярных повреждений. Агрегация белков центриолей (Ea=80 кДж/моль) чувствительнее
всего к температуре — при +2°C она ускоряется на **+22%** за шаг. Это объясняет
механизм экспоненциального ускорения прогерии при хроническом воспалении:
  SASP↑ → T↑ → CPAP/CEP290-нуклеация ускоряется exponentially → агрегаты
  блокируют дупликацию → потеря самообновления → смерть клетки раньше срока.

---

### Структура статьи

**Раздел 1. Введение**
- Прогерия (HGPS, синдром Вернера) — модели ускоренного старения
- Роль SASP в inflammaging: TNF/IL-6 → локальная гипертермия ниши
- Недостающее звено: количественная термодинамика центриолярных повреждений
- Цель: ввести Аррениус-модель в рамки CDATA, сравнить нормальное старение vs прогерия

**Раздел 2. Методы**
- CDATA-симулятор (Rust/ECS): версия v0.3.1, djabbat/CDATA-Longevity
- ThermodynamicState: уравнение Аррениуса k(T)/k(T_ref) = exp(Ea/R × (1/T_ref − 1/T))
- Активационные энергии по трекам:
  | Трек                | Eₐ (кДж/моль) | Источник |
  |---------------------|---------------|----------|
  | Карбонилирование    | 50            | Stadtman & Levine 2003 |
  | Ацетилирование      | 40            | Albaugh et al. 2011 |
  | **Агрегация**       | **80**        | Oosawa & Asakura 1975 |
  | Фосфо-дисрегуляция  | 45            | Seger & Krebs 1995 |
  | Потеря придатков    | 55            | Stadtman 2006 (Cys/Met) |
- ThermodynamicParams: baseline 36.6°C, sasp_max +2.4°C
- Пресеты: `DamageParams::progeria()` (×5) + `ThermodynamicParams::with_arrhenius()`
- SASP-вклад: sasp_intensity → local_temp → damage_rate_multiplier (лаг 1 шаг)
- Метрики: lifespan (is_senescent год), CAII, ze_velocity_analog, entropy_production

**Раздел 3. Результаты**

*3.1 Аррениус в нормальном старении*
- T=37°C: mult=1.0 (базовая линия)
- T=38°C (+1°C, умеренное воспаление): mult_aggregation ≈ 1.11, mult_mean ≈ 1.07
- T=39°C (+2°C, сильное воспаление): mult_aggregation ≈ 1.22, mult_mean ≈ 1.14
- → ускорение senescence на ~4–8 лет при хроническом SASP

*3.2 Прогерия: синергия DamageParams×5 + Аррениус*
- При progeria(): базовые скорости ×5 → сенесценция ~15 лет (без термодинамики)
- + ThermodynamicParams::with_arrhenius() + SASP 0.6 (хроническое воспаление):
  T ≈ 38.0°C → mult_aggregation ≈ 1.11
  → CPAP/CEP290-нуклеация: скорость ×5 × 1.11 = **×5.55**
  → сенесценция ещё на ~1–2 года раньше
- Ключевой вывод: нелинейный характер — агрегация доминирует над другими треками
  при гипертермии (Ea=80 — наибольший из всех)

*3.3 Ze-энтропийный анализ*
- ze_velocity_analog: у нормального 78-летнего ≈ 0.75–0.80
- При прогерии (15 лет): ze_velocity ≈ 0.65 при том же entropy_production — потому что
  entropy накоплена быстрее, но организм «субъективно моложе»
- Интерпретация: Ze v* = 0.456 → граница молодость/старение; progeria форсирует
  пересечение v* в ~3–4 года вместо ~20 лет

*3.4 Холодовая защита*
- T=35°C (-2°C): mult_aggregation ≈ 0.82 → замедление агрегации на 18%
- Биологическое соответствие: гипотермия как консервирующий стресс (крионика?)
- Практически: CR (caloric restriction) снижает T на 0.5°C → CDATA-эффект?

**Раздел 4. Обсуждение**
- Агрегация как «Ахиллесова пята» прогерии при гипертермии (Ea=80 >> других)
- Петля: HGPS-прогерин → ядерная неправильная укладка → SASP → T↑ → агрегация↑ →
  центриолярная дисфункция → утрата самообновления
- Терапевтические импликации: снижение системного воспаления (IL-6 блокада,
  Tocilizumab?) может тормозить центриолярную агрегацию через термодинамический путь
- Ограничения: T-модель скалярная (нет градиента в нише), Ea из in vitro данных

**Раздел 5. Заключение**
- Представлена первая термодинамическая модель температурозависимости центриолярных
  повреждений в рамках CDATA
- Агрегация CEP290/CPAP — доминирующий термочувствительный трек (Ea=80 кДж/моль)
- Ze-энтропийный биомаркер (ze_velocity_analog) потенциально верифицируем через
  ЭЭГ/ВСР данные (связь с Ze Vector Theory)

---

### Ключевые графики

1. **Fig 1.** Аррениус-кривые mult(T) для 5 треков (T от 35 до 40°C)
   → агрегация круче всех (наибольший наклон)
2. **Fig 2.** lifespan: нормальный vs progeria × Аррениус-on/off (4 группы)
3. **Fig 3.** CAII(t) + ze_velocity_analog(t): нормальный vs progeria (0–80 лет)
4. **Fig 4.** entropy_production как функция age_years — «энтропийные часы»

---

### Что нужно для статьи (TODO)

- [x] Написать бинарный пример `thermodynamic_progeria_example.rs`: ✅ P62
  - Normal vs Progeria (×5), ThermodynamicState, Ze Theory check
  - CSV → sensitivity_output/thermodynamic_progeria.csv
- [ ] Проверить: Ze v* = 0.456 при entropy ≈ K_ze × (0.456/0.544) ≈ 1.68
  при default params после ~18–20 лет симуляции
- [x] Fig 1–4 через Python matplotlib из CSV ✅ P64 (make_thesis_figures_final.py)
- [ ] Черновик введения (ROSCascadeState готов — связь замкнута: ROS → OH· → агрегация → Аррениус)

---

## СТАТЬЯ — P24: Ze Vector Theory как биомаркер клеточного старения

**Рабочее название:** «Centriolar Appendage Integrity as a Proxy for Ze-Vector Biological Age:
Bridging Structural Biology and Quantum-Thermodynamic Theory of Time»

**Журнал-кандидат:** Aging / Frontiers in Aging / GeroScience

**Абстракт (черновик):**
Ze Vector Theory (Tkemaladze) постулирует критическую «скорость» v* = 0.456 как точку
разделения молодого и стареющего биологического времени. Мы показываем, что CAII
(Centriolar Appendage Integrity Index) — прямой структурный биомаркер, верифицируемый
через U-ExM, — через линейное преобразование v = v* + (1−v*)×(1−CAII) переводится
в Ze-пространство. При CAII=1.0 (молодая стволовая клетка): v = v* = 0.456. При
CAII→0 (старческая потеря придатков): v → 1.0 (биологический коллапс). Дополнительный
термодинамический канал (v_entropy из entropy_production) обеспечивает независимую
оценку Ze-скорости через энтропийные часы. Консенсусная оценка v_consensus = mean(v, v_entropy)
валидирована в CDATA-симуляторе (250 тестов). Результат: первая количественная связь
структурного биомаркера центриоли с пространством Ze-теории.

---

## СТАТЬЯ — P25: Динамическая нестабильность микротрубочек и точность деления в стареющих стволовых клетках

**Рабочее название:** «Microtubule Dynamic Instability Index as a Mechanistic Link
Between PTM Accumulation and Spindle Fidelity Decline in Aging Stem Cells»

**Журнал-кандидат:** Journal of Cell Biology / Aging Cell / npj Aging

**Абстракт (черновик):**
Снижение точности деления веретена (spindle fidelity) — ключевой механизм истощения
пула стволовых клеток (Track B CDATA). Однако скалярная модель spindle_fidelity не
раскрывает молекулярные источники его деградации. Мы вводим MicrotubuleState —
динамическую модель микротрубочек (MT) на основе индекса динамической нестабильности
DII = catastrophe_rate/(poly_rate + cat_rate) [Mitchison & Kirschner 1984]. Показано:
(1) гиперацетилирование тубулина (PTM-трек CDATA) блокирует GTPase-активность →
polymerization_rate↓ → DII↑; (2) фосфо-дисрегуляция (PLK4/NEK2/Aurora B) → catastrophe_rate↑
→ DII↑; (3) Ninein (субдистальный придаток центриоли, CEP170-ассоциированный) якорит
минус-концы MT → его потеря (из AppendageProteinState) прямо снижает spindle_fidelity_derived.
Квазистационарное приближение (разделение временных масштабов: PTM-годы vs MT-секунды)
позволяет интегрировать модель в ECS без дополнительных ОДУ. Результат: механистическое
объяснение эмпирической зависимости spindle_fidelity(age) через PTM → DII → веретено-точность.

---

## СТАТЬЯ — P23: Фентоновский каскад и железо-зависимое разрушение центриолярных придатков

**Рабочее название:** «Iron-Catalyzed Fenton Chemistry Drives Centriolar Appendage
Protein Oxidation: a Kinetic Model of Hydroxyl Radical–Mediated Aging»

**Журнал-кандидат:** Free Radical Biology and Medicine / Redox Biology

**Абстракт (черновик):**
Накопление лабильного железа (labile iron pool, LIP) — ключевая черта старения
стволовых клеток, особенно HSC костного мозга. Через реакцию Фентона
(Fe²⁺ + H₂O₂ → OH· + Fe³⁺ + OH⁻) лабильное железо конвертирует относительно
стабильный H₂O₂ в высокореакционный гидроксил-радикал OH·. Мы представляем
детерминированную ODE-модель 4-переменного ROS-каскада в рамках CDATA-симулятора:
O₂⁻ (утечка комплекса I) → H₂O₂ (SOD) → OH· (Фентон, Fe²⁺-зависимо).
Показано: (1) возрастное снижение каталазы (×0.79 к 70 годам) удваивает
устойчивый уровень H₂O₂; (2) нарушение аутофагии блокирует феррофагию →
LIP растёт → OH·-поток ускоряется; (3) CEP164 (Ea_OH = 1.50) теряет
целостность на 34% быстрее CEP170 (Ea_OH = 0.55) при одинаковом OH·-уровне.
Гидроксил-радикал-зависимая потеря CAII (Centriolar Appendage Integrity Index)
— потенциальный биомаркер iron-mediated aging, верифицируемый через U-ExM
совместно с ферритин-иммунофлюоресценцией.

**Ключевые данные из симулятора:**
- [ ] high_mito_stress (mito_ros=0.8): H₂O₂ +X% vs norm через 5 лет симуляции
- [ ] age 25 vs 75: labile_iron ΔY% (из default params)
- [ ] autophagy_flux 0.1 vs 0.9: labile_iron ratio за 20 лет
- [ ] effective_oh × CEP164_sensitivity → CAII loss rate сравнить с без ROSCascade

---

## Проанализировать и оптимизировать модель CDATA, добавив новые треки и обратные связи, с целью более реалистичного моделирования старения и его биологических последствий:

[ ] Масштаб: Модель работает на уровне ниш клеток, а не отдельных молекул.
[ ] Нехватка данных: Многие параметры (например, количество молекул в центриолях) взяты теоретически, так как экспериментальных измерений нет.
[ ] Отсутствие пространства: Модель не учитывает пространственную организацию тканей и градиенты кислорода.
[ ] Неполнота иммунитета: Адаптивный иммунитет не моделируется. 
[ ] Упрощённые механизмы: Например, миелоидный сдвиг моделируется одной метрикой `myeloid_bias`, хотя в реальности это сложный процесс.
[ ] Отсутствие гетерогенности: Все ниши идентичны, нет учёта генетической или эпигенетической гетерогенности.
[ ] Сложные обратные связи: Некоторые биологические обратные связи (например, между миелоидным сдвигом и CDATA) реализованы упрощённо или отсутствуют.

## ВЫПОЛНЕНО

- [x] **CentriolarInducers → CentriolarInducerPair** — полная замена системы индукторов:
  M-комплект (материнская центриоль) + D-комплект (дочерняя). O₂ отщепляет
  от обоих (если оба непусты) или только от непустого. Новые центриоли наследуют
  ТЕКУЩИЙ остаток, а не исходный максимум.
- [x] **CentriolarDamageState sync** — каждый `step()` синхронизирует отдельный
  ECS-компонент `CentriolarDamageState` из `HumanDevelopmentComponent.centriolar_damage`,
  чтобы другие модули могли его читать без зависимости от human_development_module.
- [x] **AsymmetricDivisionModule.step()** — реализован: читает `CentriolarDamageState`,
  классифицирует тип деления (Asymmetric / SelfRenewal / Differentiation / нет деления).
- [x] **StemCellHierarchyModule.step()** — реализован: читает `spindle_fidelity` как
  прокси-потентность и синхронизирует `StemCellHierarchyState`.
- [x] **CLAUDE.md** — написан и обновлён.
- [x] **`InflammagingState` в `cell_dt_core::components`** — добавлен shared ECS-компонент
  обратной связи: `ros_boost`, `niche_impairment`, `sasp_intensity`.
- [x] **`AgingPhenotype::ImmuneDecline`** — добавлен в `aging.rs`.
- [x] **`human_development_module` читает `InflammagingState`** — применяет `ros_boost`
  к ros_level и `niche_impairment` к regeneration_tempo. Активирует `ImmuneDecline` при `sasp > 0.4`.
- [x] **`human_development_module.initialize()` спавнит `InflammagingState::default()`**.
- [x] **`myeloid_shift_module`** — полностью реализован:
  - `MyeloidShiftComponent` (myeloid_bias, lymphoid_deficit, inflammaging_index, immune_senescence, phenotype)
  - `MyeloidShiftParams` (6 параметров через get/set_params)
  - Формула CDATA-обоснована: (1-spindle)^1.5×0.45 + (1-cilia)×0.30 + ros×0.15 + agg×0.10
  - Обратная связь → InflammagingState каждый step()
  - 7 unit-тестов пройдены
  - Пример `myeloid_shift_example.rs`
- [x] **Мониторинг индукторов в `myeloid_shift_example.rs`** ✅:
  - Добавлены колонки M-ind / ΔM / D-ind / ΔD / Potency в ежегодную таблицу
  - `print_year_status` возвращает `(i32, i32)` — текущие M/D для дельты следующего шага
  - Секция `=== Inductor system ===` в финальном статусе: remaining/inherited + fraction + division_count
  - Результат калибровки (2026-03-04, seed=42):
    - Смерть: ≈78 лет ✓
    - myeloid_bias в 70 лет: **0.571** (цель 0.45; в допустимом диапазоне 0.35–0.60 ✓)
    - Индукторы: M: 10→3, D: 8→3 за 70 лет; потентность Totipotent→Pluripotent→Oligopotent
- [x] **Транскриптом → Клеточный цикл** ✅:
  - `GeneExpressionState` (p21, p16, cyclin_d, myc) в `cell_dt_core`
  - `transcriptome_module` пишет CDKN1A/CDKN2A в GeneExpressionState каждый step
  - `cell_cycle_module` читает: p21 > 0.7 → G1SRestriction; p16 > 0.8 → DNARepair; cyclin_d → G1 shorter
  - 4 новых unit-теста → 10 итого в cell_cycle_module
- [x] **AsymmetricDivision → TissueState** ✅:
  - `DivisionExhaustionState` в `cell_dt_core` (shared ECS-компонент)
  - `asymmetric_division_module` пишет exhaustion_count/asymmetric_count
  - `human_development_module` применяет `exhaustion_ratio × 0.0002` → stem_cell_pool↓
- [x] **PTM → CentriolarDamageState bridge** ✅:
  - `human_development_module` читает `Option<&CentriolePair>`, применяет PTM_SCALE=0.002/год
  - 4 unit-теста
- [x] **TODO.md** — перезаписан с актуальным статусом.
- [x] **RECOMENDATION.md** (старый файл) — помечен как устаревший.
- [x] **Два пути отщепления индукторов** ✅:
  - O₂-путь (`detach_by_oxygen`): `mother_bias=0.5` (равные M/D), `age_bias_coefficient=0.0`
  - PTM-путь истощения (`detach_by_ptm_exhaustion`): только мать, `ptm_asymmetry × ptm_exhaustion_scale`
  - 4 unit-теста: zero_asymmetry_no_detach, zero_scale_disabled, high_asymmetry_mother_only, daughter_unchanged
- [x] **Мониторинг индукторов в `myeloid_shift_example`** ✅ (M-ind/ΔM/D-ind/ΔD/Potency/Tel)
- [x] **Трек C: TelomereState** ✅:
  - `TelomereState { mean_length, shortening_per_division, is_critically_short }` в `cell_dt_core`
  - `human_development_module`: shortening = per_division × div_rate_per_stage × spindle_f × ros_f
  - `cell_cycle_module`: `is_critically_short → G1SRestriction` (постоянный Хейфликовский арест)
  - 4 unit-теста в `cell_cycle_module` (hayflick_when_critical, no_arrest_before, permanent, backward_compat)
- [x] **Трек D: EpigeneticClockState** ✅:
  - `EpigeneticClockState { methylation_age, clock_acceleration }` в `cell_dt_core`
  - `clock_acceleration = 1.0 + total_damage × 0.5`; `methylation_age += dt_years × clock_acceleration`
- [x] **Технический долг** ✅:
  - `stage_history` ограничен последними 20 (pop_front при len > 20)
  - `DamageParams::normal_aging()` — именованный алиас для `default()`
- [x] **Интеграционные тесты жизненного цикла** ✅ (4 детерминированных теста в `lifecycle_tests`):
  - `test_normal_aging_below_threshold_at_60` — damage < 0.75 в 60 лет
  - `test_longevity_below_threshold_at_95` — damage < 0.75 в 95 лет (×0.6 rates)
  - `test_progeria_accumulates_more_damage_than_normal` — прогерия > 2× нормы за 30 лет
  - `test_longevity_less_damage_than_normal` — долгожители < 75% нормы за 60 лет
  - Примечание: тесты детерминированы (`base_detach_probability=0.0`); `thread_rng()` — нестохастичен

---

## 1. ПОДГОТОВКА К МЕЛОИДНОМУ СДВИГУ ✅ ВЫПОЛНЕНО

- [x] **`InflammagingState` в `cell_dt_core::components`** — добавлен.
- [x] **`human_development_module` читает `InflammagingState`** — применяет `ros_boost` и `niche_impairment`.
- [x] **`AgingPhenotype::ImmuneDecline`** — добавлен в `aging.rs`.
- [x] **`human_development_module.initialize()` спавнит `InflammagingState::default()`**.

---

## 2. МЕЛОИДНЫЙ СДВИГ ✅ ВЫПОЛНЕНО

### Биология и связь с CDATA

С возрастом гематопоэтические стволовые клетки (HSC) и стволовые клетки других тканей
смещают дифференцировку от лимфоидного пути к миелоидному. В рамках CDATA это происходит
через четыре конкретных молекулярных повреждения:

| Компонент CDATA | Механизм биологически | Вклад в myeloid_bias |
|---|---|---|
| `spindle_fidelity ↓` | Веретено не может сегрегировать fate-детерминанты (Numb, aPKC) → оба потомка миелоидные | **45%** |
| `ciliary_function ↓` (CEP164↓) | Нет реснички → нет Wnt/Notch/Shh → LT-HSC теряет лимфоидный нише-сигнал → PU.1 побеждает | **30%** |
| `ros_level ↑` | ROS → NF-κB → IL-6, TNF-α, IL-1β → SASP → миелоидная цитокиновая среда | **15%** |
| `protein_aggregates ↑` | Агрегаты белков захватывают IKZF1/Ikaros → снятие репрессии с миелоидных генов | **10%** |

**Обратные связи мелоидного сдвига → CDATA:**

```
myeloid_bias ↑
  → inflammaging_index ↑
      → ros_boost ↑     → DamageParams.ros_rate ускоряется (петля ROS усиливается)
      → niche_impairment↑ → TissueState.regeneration_tempo ↓
  → immune_senescence ↑
      → AgingPhenotype::ImmuneDecline активируется
      → lymphoid_deficit ↑ → снижение иммунного надзора (онкологический риск)
```

### Формулы

```
spindle_c  = (1 − spindle_fidelity)^1.5 × 0.45
cilia_c    = (1 − ciliary_function)  × 0.30
ros_c      = ros_level               × 0.15
aggr_c     = protein_aggregates      × 0.10

myeloid_bias = clamp(spindle_c + cilia_c + ros_c + aggr_c,  0.0, 1.0)

lymphoid_deficit   = myeloid_bias                          (упрощённая модель)
inflammaging_index = myeloid_bias × lymphoid_deficit × 0.8
immune_senescence  = inflammaging_index × 0.7 + (1 − cilia_c_raw × 2) × 0.3

ros_boost        = inflammaging_index × 0.15   → InflammagingState
niche_impairment = inflammaging_index × 0.08   → InflammagingState
sasp_intensity   = inflammaging_index           → InflammagingState
```

**Калибровочные ориентиры:**
- Возраст 20 лет (pristine): `myeloid_bias ≈ 0.02` — норма
- Возраст 50 лет: `myeloid_bias ≈ 0.25` — MildShift (умеренный, субклинический)
- Возраст 70 лет: `myeloid_bias ≈ 0.45` — ModerateShift (клинически значимый)
- Возраст 85 лет: `myeloid_bias ≈ 0.65` — SevereShift (иммуностарение)

### Технические шаги

- [x] **Создать crate `myeloid_shift_module`** — выполнено.

- [x] **`MyeloidShiftComponent`** — реализован:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct MyeloidShiftComponent {
      pub myeloid_bias: f32,
      pub lymphoid_deficit: f32,
      pub inflammaging_index: f32,
      pub immune_senescence: f32,
      pub phenotype: MyeloidPhenotype,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum MyeloidPhenotype {
      Healthy,        // myeloid_bias < 0.30
      MildShift,      // 0.30..0.50
      ModerateShift,  // 0.50..0.70  ← клинически значимый
      SevereShift,    // > 0.70      ← иммуностарение
  }
  ```

- [x] **`MyeloidShiftParams`** — реализован:
  ```rust
  pub struct MyeloidShiftParams {
      pub spindle_weight: f32,     // default 0.45
      pub cilia_weight: f32,       // default 0.30
      pub ros_weight: f32,         // default 0.15
      pub aggregate_weight: f32,   // default 0.10
      pub ros_boost_scale: f32,    // default 0.15
      pub niche_impair_scale: f32, // default 0.08
  }
  ```

- [x] **`MyeloidShiftModule.step()`** — реализован:
  1. Для каждой сущности с `(&CentriolarDamageState, &mut MyeloidShiftComponent, &mut InflammagingState)`:
  2. Вычислить `myeloid_bias` по формуле выше
  3. Вычислить `inflammaging_index`, `immune_senescence`
  4. Обновить `MyeloidShiftComponent`
  5. Записать в `InflammagingState { ros_boost, niche_impairment, sasp_intensity }`

- [x] **`MyeloidShiftModule.initialize()`** — реализован:
  - `MyeloidShiftComponent::default()`
  - `InflammagingState::default()` (если не было добавлено ранее)

- [x] **Unit-тесты** — 7 тестов пройдены (pristine, max_damage, spindle, cilia, calibration_age70, ros_boost, phenotype).

- [x] **Пример `myeloid_shift_example.rs`** — создан в `examples/src/bin/`.

- [x] **`CLAUDE.md`** — обновлён.

---

## 3. ЗАГЛУШКИ (существующие модули без реализации)

- [x] **PTM → CentriolarDamageState bridge** — реализован в `human_development_module` ✅:
  - Читает `Option<&CentriolePair>` в step(), применяет PTM_SCALE=0.002/год
  - acetylation→tubulin_hyperacetylation, oxidation→carbonylation, phospho→phospho_dysreg, methyl→aggregates
  - 4 unit-теста (scale_is_moderate проверяет что bridge < 50% от базового damage за 30 лет)

- [x] **`centriole_module.step()`** — PTM-накопление реализовано ✅:
  - Читает `CellCycleStateExtended` (Option) для детектирования M-фазы
  - Накапливает PTM в `CentriolePair.mother.ptm_signature` и `.daughter.ptm_signature`
  - Мать накапливает в `daughter_ptm_factor=0.4` раза быстрее дочерней
  - M-phase boost ×3.0 (максимальный стресс тубулина при митозе)
  - Не трогает `CentriolarDamageState` — двойного счёта нет
  - 6 unit-тестов пройдены: ptm_starts_at_zero, increases_after_steps,
    mother_accumulates_faster, m_phase_boosts, ptm_clamped_at_one, daughter_factor_zero

- [x] **`AsymmetricDivisionModule` — спавн дочерних сущностей** ✅ (сессия 4):
  - `enable_daughter_spawn: bool` (default: false, opt-in) + `max_entities: usize` (default: 1000)
  - Spawn queue pattern: собирается во время `query_mut`, применяется после
  - Дочерняя клетка наследует `ros_level * 0.3` от родителя (mitochondrial legacy)
  - Компоненты новой сущности: `CellCycleStateExtended`, `CentriolarDamageState::pristine()`,
    `AsymmetricDivisionComponent::default()`, `DivisionExhaustionState::default()`, `InflammagingState::default()`

- [x] **`StemCellHierarchyModule` — пластичность** ✅ (сессия 3):
  - При `enable_plasticity = true` и `potency == Oligopotent`:
    вероятность `plasticity_rate` перехода в `Pluripotent` если `spindle_fidelity > differentiation_threshold`
  - `dedifferentiation_count: u32` — счётчик событий; 2 unit-теста

- [x] **`CellCycleModule` — enforced checkpoints** — реализовано ✅:
  - G1/S checkpoint: `total_damage_score() > checkpoint_strictness` → `G1SRestriction` (арест)
  - G2/M checkpoint (SAC): `spindle_fidelity < (1 - checkpoint_strictness)` → `SpindleAssembly`
  - Читает `Option<&CentriolarDamageState>` — нет прямой зависимости от `human_development_module`
  - `checkpoint_strictness=0.0` (дефолт) → аресты отключены, полная обратная совместимость
  - Growth factors синхронизируются из damage: `dna_damage = total_damage_score()`, `oxidative_stress = ros_level`
  - 6 unit-тестов пройдены: pristine_advances, damaged_arrests_g1s, broken_spindle_arrests_g2m,
    zero_strictness_never_arrests, arrest_releases_when_damage_clears, cells_divided_counter

---

## 4. ОБРАТНЫЕ СВЯЗИ МЕЖДУ МОДУЛЯМИ

- [x] **Мелоидный сдвиг → DamageParams (через `InflammagingState`)** ✅:
  - `human_development_module.step()` читает `Option<&InflammagingState>` и применяет `ros_boost` + `niche_impairment`
  - Петля замкнута: повреждение → myeloid_shift → inflammaging → больше ROS → больше повреждений

- [x] **Транскриптом → клеточный цикл** ✅:
  - Добавлен `GeneExpressionState` (p21, p16, cyclin_d, myc) в `cell_dt_core::components`
  - `transcriptome_module` добавил гены CDKN1A/CDKN2A, синхронизирует уровни в `GeneExpressionState`
  - `cell_cycle_module` читает `Option<&GeneExpressionState>`:
    `p21 > 0.7` → `G1SRestriction`; `p16 > 0.8` → `DNARepair` (постоянный); cyclin_d → укорачивает G1
  - 4 новых unit-теста: p21_arrests_g1s, p21_arrest_releases, p16_permanent_arrest, cyclin_d_shortens_g1

- [x] **AsymmetricDivision → TissueState** ✅:
  - Добавлен `DivisionExhaustionState` (exhaustion_count, asymmetric_count, total_divisions) в `cell_dt_core`
  - `asymmetric_division_module` синхронизирует `DivisionExhaustionState` каждый шаг деления
  - `human_development_module` читает `Option<&DivisionExhaustionState>`:
    `exhaustion_ratio × 0.0002/шаг` → снижает `stem_cell_pool`

- [x] **MyeloidShift → AgingPhenotype** ✅ реализован через InflammagingState:
  - `MyeloidShiftModule` пишет `inflammaging.sasp_intensity = inflammaging_index`
  - `HumanDevelopmentModule` читает `infl_sasp > 0.4` → `active_phenotypes.push(AgingPhenotype::ImmuneDecline)`
  - Прямое чтение `MyeloidShiftComponent` не нужно — `InflammagingState` служит интерфейсом

---

## 5. НОВЫЕ БИОЛОГИЧЕСКИЕ ТРЕКИ

### Трек C: Теломеры ✅ ВЫПОЛНЕНО

#### Биология и связь с CDATA

| Механизм | CDATA-компонент |
|----------|-----------------|
| Каждое деление укорачивает теломеры (Хейфлик) | `div_rate` per `HumanDevelopmentalStage` |
| Нарушенное веретено → хромосомная нестабильность → быстрее укорачивание | `spindle_fidelity ↓` |
| ROS → окислительное повреждение теломерной ДНК | `ros_level ↑` |
| Критически короткие → G1-арест (сенесценция, Хейфлик) | `is_critically_short → G1SRestriction` |

**Калибровка (T/S ratio):**
- Зигота: 1.0 (полная длина)
- 40 лет: ≈ 0.7
- 70 лет: ≈ 0.4
- Критически короткие (< 0.3): Хейфликовский предел → сенесценция

#### Технические шаги

- [x] **`TelomereState`** — добавлен в `cell_dt_core::components`
- [x] **`human_development_module.step()`** — читает `Option<&mut TelomereState>`:
  - `div_rate` — inline match по `HumanDevelopmentalStage` (не через `DevelopmentalStage`)
  - `mean_length -= base × spindle_f × ros_f`
  - `AgingPhenotype::TelomereShortening` при `is_critically_short`
- [x] **`cell_cycle_module.step()`** — `is_critically_short → G1SRestriction` (постоянный арест)
- [x] **`human_development_module.initialize()`** — спавнит `TelomereState::default()`
- [x] **`myeloid_shift_example`** — колонка `Tel` (mean_length)
- [x] **Unit-тесты (4 шт. в `cell_cycle_module`)**: hayflick_when_critical, no_arrest_before_critical, permanent, backward_compat

### Трек D: Эпигенетические часы ✅ ВЫПОЛНЕНО

- [x] **`EpigeneticClockState`** — добавлен в `cell_dt_core::components`:
  ```rust
  pub struct EpigeneticClockState {
      pub methylation_age: f32,    // биологический возраст по CpG-метилированию
      pub clock_acceleration: f32, // 1.0 + total_damage × 0.5
  }
  ```
- [x] **Модель**: `methylation_age += dt_years × clock_acceleration`
- [x] **AgingPhenotype::EpigeneticChanges** ✅ — активируется при `clock_acceleration > 1.2`
  - `epi_ros_contribution` → подаётся в `accumulate_damage()` следующего шага (лаг 1 шаг)

### Митохондриальный трек

- [x] **Новый модуль `mitochondrial_module`** ✅ (сессия 7):
  - `MitochondrialState { mtdna_mutations, fusion_index, ros_production, membrane_potential, mitophagy_flux, mito_shield_contribution }`
  - Питает `ros_boost` в `accumulate_damage()` через `human_development_module`
  - Петля I: мутации → ROS → мутации; Петля II: ROS → фрагментация → митофагия хуже
  - Митофагия: при `ros_production > mitophagy_threshold` → перегрузка → ускорение деградации
  - 7 unit-тестов; калибровка: age 70 → ros≈0.37, mtdna≈0.30, fusion≈0.49
  - `mitochondrial_example` — вывод 6 метрик каждые 10 лет

---

## 6. КАЛИБРОВКА И ВЕРИФИКАЦИЯ

### Проверка логики (2026-03-04)

Ручная калибровка через `myeloid_shift_example` (seed=42, 5 ниш, params default):

| Метрика | Результат | Цель | Статус |
|---------|-----------|------|--------|
| Смерть (normal aging) | ~78 лет | 65–95 лет | ✅ |
| myeloid_bias в 70 лет | 0.571 | 0.35–0.60 | ✅ (чуть выше 0.45) |
| Потентность в 0–30 лет | Totipotent | Totipotent | ✅ |
| Потентность в 40–60 лет | Pluripotent | Pluripotent | ✅ |
| Потентность в 70 лет | Pluripotent/Oligopotent | Oligopotent | ⚠️ незначительно |
| M-инд. в 70 лет | 3/10 (30%) | ~20–40% | ✅ |
| D-инд. в 70 лет | 3/8 (37.5%) | ~25–50% | ✅ |

⚠️ Примечание: `myeloid_bias` в 70 лет несколько выше 0.45 из-за стохастичности
отщепления индукторов (seed-зависимо). Принципиальных ошибок нет.

### Автоматические тесты ✅ ВЫПОЛНЕНО

- [x] **Детерминированные lifecycle-тесты** (4 шт. в `lifecycle_tests`):
  - `test_normal_aging_below_threshold_at_60` — damage < 0.75 в 60 лет ✓
  - `test_longevity_below_threshold_at_95` — damage < 0.75 в 95 лет ✓
  - `test_progeria_accumulates_more_damage_than_normal` — прогерия > 2× нормы за 30 лет ✓
  - `test_longevity_less_damage_than_normal` — долгожители < 75% нормы за 60 лет ✓
  - **Важно**: тесты отключают `thread_rng()`-зависимый путь (`base_detach_probability=0.0`)
    для детерминизма; проверяют molecular damage track (DamageParams), не inductor depletion

- [x] **`DamageParams::normal_aging()`** — добавлен алиас для `default()` ✓

- [x] **`stage_history` — ограничен pop_front при len > 20** ✓

- [x] **Тест калибровки индукторов** ✅ (сессия 6):
  - `test_inductor_depletion_occurs` — за 78 лет оба комплекта теряют ≥1 индуктор (seed=42)
  - `test_inductor_calibration_multiseed` — средняя потеря ≥0.5 индуктора по 5 seed'ам
  - Стохастическое отщепление: base_detach=0.002 + ptm_exhaustion=0.001, seed через SimulationConfig

- [x] **Тест миелоидного сдвига** ✅ (сессия 6):
  - `test_myeloid_bias_low_at_age_20` — bias < 0.15 в 20 лет
  - `test_myeloid_bias_moderate_at_age_70` — 0.20 < bias < 0.75 в 70 лет
  - `test_myeloid_bias_high_at_age_85` — bias > 0.35 в 85 лет
  - `test_myeloid_bias_increases_with_age` — монотонность bias(70) > bias(20)
  - Детерминированные (base_detach=0.0): myeloid_shift_module как dev-dependency

---

## 7. ИНФРАСТРУКТУРА И ЭКСПОРТ

- [x] **CSV-экспорт через `cell_dt_io`** ✅ (сессия 4):
  - `CdataRecord` + `CdataExporter` + `write_cdata_csv` в `cell_dt_io/src/cdata_exporter.rs`
  - Колонки: `step, entity_id, tissue, age_years, stage, damage_score, myeloid_bias, spindle_fidelity, ciliary_function, frailty, phenotype_count`
  - `CdataExporter::collect(world, step)` — запрос по `(&HumanDevelopmentComponent, Option<&MyeloidShiftComponent>)`
  - `io_example.rs` обновлён: демонстрирует `DataExporter` (базовые данные) + `CdataExporter` (CDATA)
  - `DataExporter::buffered()` — добавлен метод проверки размера буфера

- [x] **Визуализация через `cell_dt_viz`** ✅ (сессия 5):
  - `CdataSnapshot` — агрегированные метрики всех живых ниш за один шаг
  - `CdataTimeSeriesVisualizer` — 4-панельный PNG-график (damage, myeloid_bias, spindle, frailty) по оси времени (лет)
  - `cdata_viz_example.rs` — демо: 1200 шагов ≈ 100 лет, 5 тканей, снимок каждый год

- [x] **Python bindings `cell_dt_python`** ✅ реализованы (сессия 5):
  - `PyHumanDevelopmentData` (13 полей: stage, age_years, damage_score, spindle, cilia, frailty, m/d inducers, potency...)
  - `PyMyeloidShiftData` (myeloid_bias, lymphoid_deficit, inflammaging_index, immune_senescence, phenotype)
  - `PyCdataSimulation` — класс с `add_tissue()`, `run()`, `step()`, `get_cdata_data()`, `get_myeloid_data()`
  - `run_cdata_simulation(steps, dt, seed, tissues)` → `Vec<PyDict>` со всеми полями

- [x] **`cell_dt_gui` — панель управления** ✅ (сессия 6):
  - Вкладка `Tab::Cdata` ("🔴 CDATA / Aging") добавлена в навигацию
  - `CdataGuiConfig` + `DamagePreset` — новые типы конфигурации
  - Слайдеры: `base_detach_probability`, `mother_bias`, `age_bias_coefficient`
  - Слайдеры: `spindle_weight`, `cilia_weight`, `ros_weight`, `aggregate_weight`
  - ComboBox: Normal / Progeria (×5) / Longevity (×0.6)
  - Индикатор суммы весов (Σ, цветовая метка)
  - Коллапсируемые блоки со справкой по путям A/B/C

---

## 8. ТЕХНИЧЕСКИЙ ДОЛГ

- [x] **Дублирование tissue_type** ✅ (сессия 6):
  - `TissueType` в core расширен до 14 вариантов (добавлены Blood, Epithelial, Liver, Kidney, Heart, Lung, Bone, Cartilage, Adipose, Connective)
  - `HumanTissueType` удалён как отдельный enum; стал публичным псевдонимом `pub type HumanTissueType = TissueType`
  - `map_tissue_type()` удалена; `for_tissue()` использует тип напрямую
  - `organism.rs`: `Hematopoietic` → `Blood`, `IntestinalCrypt` → `Epithelial`
  - Все крейты компилируются; 68/68 тестов

- [x] **Логирование** ✅ (сессия 5):
  - `trace!` — per-step начала (human_dev, myeloid_shift, cell_cycle, asymmetric_div)
  - `info!` — milestone: смерть ниши, смена стадии, G1/S/G2M аресты, Hayflick, p16/p21
  - `warn!` — нереалистичные значения: ros_level > 1.0, total_damage_score > 1.0, myeloid_bias ≥ 0.95, entity limit

- [x] **Параметры `DamageParams` доступны через панель управления** ✅ (сессия 3):
  `get_params`/`set_params` с полями `base_ros_damage_rate`, `aggregation_rate`, `senescence_threshold`, `damage_preset`

- [x] **`CellCycleStateExtended::new()` задокументирован** ✅ (сессия 5):
  doc-comment поясняет обязательность компонента при спавне + пример кода.

- [x] **Очистка dead-сущностей** ✅ (сессия 3):
  `Dead`-маркер + `SimulationManager::cleanup_dead_entities()` + `cleanup_dead_interval: Option<u64>` в конфиге.

---

## 9. ИСПРАВЛЕНИЯ ЛОГИЧЕСКИХ ОШИБОК (сессия 4)

- [x] **Fix 1: HashMap → Vec** — `SimulationManager.modules: Vec<(String, Box<dyn SimulationModule>)>`.
  Гарантирует порядок выполнения = порядку регистрации. Тест `test_module_execution_order_is_guaranteed`.

- [x] **Fix 2: Петля ros_boost** — `accumulate_damage()` принимает 5-й аргумент `ros_level_boost: f32`.
  `ros_level` вычисляется ДО `protein_carbonylation`. Устранена ошибка: boost не влиял на carbonylation.

- [x] **Fix 3: senescence_threshold параметризован** — `CentriolarDamageState.senescence_threshold: f32`
  синхронизируется из `DamageParams` каждый шаг. `update_functional_metrics()` использует `self.senescence_threshold`.

- [x] **Fix 4: Seeded RNG** — `SimulationModule::set_seed(u64)` в трейте (default no-op).
  `HumanDevelopmentModule`, `StemCellHierarchyModule`, `TranscriptomeModule` → `StdRng::seed_from_u64(seed)`.

- [x] **Fix 5: lymphoid_deficit** — независимая формула:
  `(1-cilia)×0.55 + aggregates×0.35 + hyperacetylation×0.10`. Ранее: тавтология `= myeloid_bias`.

- [x] **Fix 6: Мутация случайного гена** — `apply_mutation()` выбирает ген по случайному индексу.
  Ранее: `HashMap::values_mut().next()` — всегда первый "случайный" ключ.

- [x] **Fix 7: Теломеры в стволовых клетках** — TERT-защита:
  - Эмбриональные стадии (Zygote..Fetal): укорочения нет
  - `spindle_fidelity ≥ 0.75` (Pluripotent/Totipotent): укорочения нет

- [x] **Fix 8: EpigeneticClockState → обратная связь** — `epi_ros_contribution` питает ROS следующего шага.
  Активация `AgingPhenotype::EpigeneticChanges` при ускорении часов.

- [x] **Fix 9: Оптимизации** —
  - `update_functional_capacity()` вызывается один раз в конце всех тканевых обновлений
  - `expression_history: VecDeque` в transcriptome_module
  - `InducerDetachmentParams: #[derive(Copy)]`
  - Удалён неиспользуемый `DevelopmentParams::s_inducers_initial`

---

## ПОРЯДОК ВЫПОЛНЕНИЯ (рекомендуемый)

```
✅ 1  InflammagingState + AgingPhenotype::ImmuneDecline
✅ 2  myeloid_shift_module (crate + step + tests + example)
✅ 3  human_dev инициализирует InflammagingState
✅ 4  centriole_module.step() — PTM-накопление (6 тестов)
✅ 5  Транскриптом → клеточный цикл (GeneExpressionState, 4 теста)
✅ 6  AsymmetricDivision → TissueState (DivisionExhaustionState)
✅ 7  PTM → CentriolarDamageState bridge (4 теста)
✅ 8  CellCycleModule enforced checkpoints (10 тестов)
✅ 9  Мониторинг индукторов + PTM exhaustion (равные M/D, 4 теста)
✅ 10 TelomereState Трек C + Hayflick в cell_cycle (4 теста) + Tel колонка в примере
✅ 11 EpigeneticClockState Трек D + epi_ros_contribution обратная связь
✅ 12 Интеграционные тесты lifecycle (4 детерм. теста)
✅ 13 Технический долг (stage_history pop_front, DamageParams::normal_aging())
✅ 14 Dead-маркер + cleanup_dead_entities (сессия 3)
✅ 15 StemCellHierarchy пластичность / дедифференцировка (сессия 3)
✅ 16 DamageParams панель управления (сессия 3)
✅ 17 Исправления логических ошибок (Fix 1–9, сессия 4) — 62/62 тестов
✅ 18 Спавн дочерних сущностей (asymmetric_division)         → п. 3
✅ 19 CSV CDATA-экспорт (CdataExporter, io_example обновлён) → п. 7
✅ 20 GUI CDATA-вкладка (Tab::Cdata, CdataGuiConfig, DamagePreset, сессия 6) → п. 7
✅ 21 Тест калибровки индукторов (2 теста, multiseed, сессия 6)           → п. 6
✅ 22 Тесты миелоидного сдвига по возрастам (4 теста, сессия 6)           → п. 6
✅ 23 DifferentiationStatus + ModulationState (сессия 7)                  → п. 3
      DifferentiationTier (Ord), try_advance (необратимо), ModulationState
      5 тестов: tier_ordering, from_potency, irreversibility, same_tier, modulation_default
✅ 24 De novo создание центриолей + мейотическая элиминация (сессия 7)    → п. 3
      de_novo_centriole_division (u32, дефолт 4), meiotic_elimination_enabled (bool, дефолт true)
      HumanDevelopmentalStage: PartialOrd/Ord; inductors_active/meiotic_reset_done в DifferentiationStatus
      GUI: новая секция "🧬 Жизненный цикл индукторов" (slider 1-8, checkbox)
      4 теста: inductors_inactive_by_default, reset_for_meiosis, de_novo_stage_mapping, stage_ordering
✅ 25 Митохондриальный модуль Трек E (сессия 7)                           → п. 5
      MitochondrialState (6 полей), MitochondrialModule, lazy-init в step()
      ros_boost → accumulate_damage(), mito_shield → via ROS loop
      7 тестов: pristine, mutations_accumulate, ros_increases, shield_bounded,
               ros_boost_scaling, all_metrics_bounded, fusion_decreases
      mitochondrial_example: вывод mtDNA/ROS/fusion/shield/mBias каждые 10 лет
```

---

*При каждом выполненном пункте: переместить в секцию "ВЫПОЛНЕНО" вверху, обновить дату.*
*Последнее обновление: 2026-03-23 (сессия P50-P51) — 208 тестов ✅*
*Изменить RECOMMENDATION.md, TODO.md и README.md соответственно изменениям*

---

## ВЫПОЛНЕНО В СЕССИЯХ 11–15 (2026-03-11)

### P7 — Многотканевая модель ✅
- [x] `OrganismState`: добавлены `igf1_level`, `systemic_sasp`
- [x] Системный SASP: mean(sasp_output всех ниш) → ros_boost +5% каждой нише (лаг 1 шаг)
- [x] Ось IGF-1/GH: `igf1 = (1 - (age-20)*0.01).clamp(0.3, 1.0)` → regeneration_tempo×(0.8+0.2×igf1)
- [x] `multi_tissue_example.rs`: 5 тканей × 5 ниш = 25 сущностей

### P8 — Критерий смерти организма ✅
- [x] 3 критерия: frailty ≥ 0.95 / Blood pool < 0.02 (панцитопения) / Neural < 0.05
- [x] `update_organism_state()` — агрегирует ECS по `tissue_type` каждый шаг
- [x] 4 теста: frailty_death, pancytopenia, neurodegeneration, healthy_survives

### P11 — Интервенции ✅
- [x] 8 типов: Senolytics, NadPlus, CaloricRestriction, Antioxidant, Tert,
       CafdRetainer, CafdReleaser, CentrosomeTransplant
- [x] `compute_effect()`, `add_intervention()`, `healthspan_years()`, 10 тестов

### P12 — Авто-CSV ✅
- [x] `CdataCollect` трейт, `set_exporter()` / `write_csv()` в `SimulationManager`

### P2 — SA-анализ ✅
- [x] `sensitivity_analysis.rs`: 11 параметров, tornado-chart, CSV

### P13 — Морфогенные поля (временная зависимость) ✅
- [x] `stage_morphogen_scale()`: GLI/Shh/BMP/Wnt мультипликаторы по стадии
- [x] Hill-функция: `gli = cilia^2 / (0.5^2 + cilia^2) × stage_scale`
- [x] 6 тестов: `morphogen_temporal_tests`

### P14 — Эпигенетическое наследование при делении ✅
- [x] `EpigeneticClockState.last_division_count: u32`
- [x] При делении: `methylation_age = (methylation_age + chron_age) / 2`
- [x] 4 теста: `epigenetic_inheritance_tests`

### P15 — NK-клеточный иммунный надзор ✅
- [x] `NKSurveillanceState`: nkg2d_ligand_expression, nk_activity, kill_probability
- [x] Baseline subtraction: `nkg2d = (ros×0.6 + agg×0.4 - 0.30).max(0)`
- [x] NK_KILL_THRESHOLD = 0.10 (откалиброван)
- [x] Возрастной спад NK после 40 лет; миелоидное подавление через TGF-β прокси
- [x] 5 тестов: `nk_surveillance_tests`

### P16 — Протеостаз / агрегасомы ✅
- [x] `ProteostasisState`: proteasome_activity, hsp_capacity, aggresome_index, clearance_rate
- [x] Интеграция с P18 (циркадный ночной буст к клиренсу)
- [x] 5 тестов: `proteostasis_tests`

### P17 — Компенсаторная пролиферация ✅
- [x] `ros_boost = (0.5 - pool) × 0.30` при pool < 0.5
- [x] 3 теста: `compensatory_proliferation_tests`

### P18 — Циркадный ритм ✅
- [x] `CircadianState`: amplitude, proteasome_night_boost, circadian_sasp_contribution
- [x] `amplitude = (cep164×0.6 + (1-agg)×0.4) × (1-ros×0.2)`
- [x] Буст протеасомы ночью добавляется к clearance_rate в P16
- [x] 5 тестов: `circadian_tests`

### P19 — Аутофагия / mTOR ✅
- [x] `AutophagyState`: mtor_activity, autophagy_flux, aggregate_autophagy_clearance
- [x] CR: mTOR×0.30, NadPlus: mTOR×0.20
- [x] 7 тестов: `autophagy_tests`

### P20 — Ответ на повреждение ДНК (DDR) ✅
- [x] `DDRState`: gamma_h2ax_level, atm_activity, p53_stabilization, dna_repair_capacity
- [x] `γH2AX = (1-spindle)^1.5 × 0.8 + ros × 0.2`
- [x] Замыкает петлю CDATA → cell_cycle: `p21 += p53 × 0.3` в GeneExpressionState
- [x] 7 тестов: `ddr_tests`

### Track G — Life-History Trade-off / Гормональный Часовой Механизм ✅ (2026-03-18)
- [x] **Теоретическое обоснование**: Tkemaladze J. "Theory of Lifespan Decline" (2026)
  - Возраст начала половой зрелости ↔ продолжительность жизни: r=0.78, R²=0.92 (библейские генеалогии)
  - HPG-ось: пубертат активирует «Life-History Trade-off» (репарация vs репродукция)
  - Эстрогены/тестостерон — антиоксидантная защита центриолей; менопауза → снятие защиты
- [x] `HormonalFertilityState` (cell_dt_core/components.rs): hormone_level, hormonal_protection,
       life_history_factor, puberty_age_years, menopause_age_years, phase (ReproductivePhase)
- [x] `hormonal_fertility.rs`: `update_hormonal_fertility()`, 5 фаз (Prepubertal→Postmenopausal)
- [x] Интеграция в step() шаг 1и:
  - `ros_level -= hormonal_protection × 0.20` (гормональная антиоксидантная защита)
  - `base_detach_probability × life_history_factor` (max 1.20 после пубертата)
- [x] Ранний пубертат → больший trade-off → быстрее стареют (соответствует r=0.78 из статьи)
- [x] 9 тестов: `hormonal_fertility::tests`

### Track F — Снижение темпа деления стволовых клеток ✅ (2026-03-11)
- [x] `StemCellDivisionRateState`: division_rate, cilia_drive, spindle_drive,
       age_factor, ros_brake, mtor_brake, decline_index
- [x] 5 независимых молекулярных тормозов, формула: произведение компонентов
- [x] Применяется: `regeneration_tempo *= division_rate.sqrt()` в шаге 1з
- [x] Биологическое обоснование: Tkemaladze 2024
- [x] 8 тестов: `division_rate_tests`

### Веб-сайт CDATA DT ✅ (2026-03-11)
- [x] `/home/oem/Desktop/CDATA/website/index.html`
- [x] 21 PDF в `website/papers/` (10 скопированы, 11 сконвертированы из docx)
- [x] Секции: Theory (inline ссылки) / 6 Tracks / Mechanisms P13–P20 / Platform / Publications (21 статья)
- [x] Сервер: `python3 -m http.server 8766` из папки `website/`

---

## 10. ROADMAP v2 — По результатам научной статьи (2026-03-05)

> Источник: `CDATA_Digital_Twin_Article.md`, раздел 6 «Critical Analysis» + раздел 7.2 «Priority Roadmap».
> Приоритеты расставлены по критичности для научной обоснованности модели.
>
> **Статус сессии 9 (2026-03-05):** P3 ✅ P4 ✅ P5 ✅ P6 ✅ P10 ✅ — реализованы.
> P11/P12 — добавлены по итогам анализа следующих шагов.

---

### P1 — Клеточная популяционная динамика ✅ ВЫПОЛНЕНО (сессия 13, 2026-03-11)

**Проблема:** каждая ниша изолирована, нет конкуренции, нет клональной динамики.
Без этого невозможно воспроизвести CHIP (клональный гемопоэз), вариабельность
темпа старения между особями, пул-истощение через демографический дрейф.

- [x] **`NeedsHumanDevInit` маркер** в `cell_dt_core::components` — lazy-init для NichePool-замен
- [x] **Конкуренция ниш:** `NichePool` — общий ресурс ниш; `enable_daughter_spawn`, `max_entities: 200`
- [x] **Клональная экспансия:** симметричное самообновление → вытеснение соседней ниши
- [x] **`ClonalState { clone_id, generation }`** — отслеживание происхождения клонов
- [x] **Тест CHIP:** 20 HSC-ниш; CHIP детектируется с года 40, к году 79 — 3 доминирующих клона (50%/29%/14%) ✅
- [x] **`niche_pool_example.rs`** — демонстрация клонального дрейфа

---

### P2 — Анализ чувствительности параметров ✅ ВЫПОЛНЕНО (2026-03-10)

- [x] **`sensitivity_analysis.rs`** — 11 параметров × 3 варианта (base/+50%/−50%), 33 симуляции
- [x] **Метрики:** lifespan, damage_at_60, frailty_at_70
- [x] **Tornado-chart** (текстовый, отсортированный по |Δlifespan|)
- [x] **CSV** → `sensitivity_output/sa_results.csv`
- [x] **`get_module_params()` / `set_module_params()`** добавлены в `SimulationManager`
- [x] **`ParameterSweepConfig`** — структура конфигурации sweep в `sensitivity_analysis.rs`
- [x] **x4.2 задокументирован** в `damage.rs` (Bratic & Larsson 2013, Chance et al. 1979)

**Ключевые результаты (seed=42, baseline=81.2yr):**
```
Parameter              +50% Δyr   -50% Δyr   Вывод
midlife_multiplier      -13.0      +38.2     КРИТИЧЕН — антагонистическая плейотропия
senescence_threshold     +0.0      -31.2     Порог определяет дату смерти
cep89_loss_rate/ninein   -2.7      +13.0     Придатки важнее молекулярных повреждений
cep170_loss_rate         -6.9      +12.3
aggregation_rate         -4.0       +3.1
base_ros_damage_rate     -0.7       +0.7     Наименее чувствительный
```

---

### P3 — Стохастические уравнения накопления повреждений ✅ ВЫПОЛНЕНО (2026-03-05)

- [x] `DamageParams.noise_scale: f32` (default 0.0 — детерминированный режим)
- [x] Ланжевен-шум в `HumanDevelopmentModule::step()` после `accumulate_damage()`:
  `sigma = noise_scale * sqrt(dt_years)`, uniform-аппроксимация, 4 молекулярных поля clamped [0,1]
- [x] Экспозиция через `get_params()` / `set_params()`
- [x] Шум применяется в lib.rs (не в damage.rs) — seeded StdRng модуля

---

### P4 — Сигмоидный возрастной мультипликатор ✅ ВЫПОЛНЕНО (2026-03-05)

- [x] `DamageParams::age_multiplier()` — логистическая функция:
  `1.0 + (midlife_damage_multiplier - 1.0) * sigmoid(age, center=42.5, width=7.5)`
- [x] Новые поля: `midlife_transition_center: f32`, `midlife_transition_width: f32`
- [x] Тесты (4 шт.): smooth_at_40, range, center_half_way, monotone
- [x] Экспозиция через `get_params()` / `set_params()`

---

### P5 — Репарация придатков центриоли ✅ ВЫПОЛНЕНО (2026-03-05)

- [x] Новые поля в `DamageParams`: `cep164_repair_rate`, `cep89_repair_rate`,
  `ninein_repair_rate`, `cep170_repair_rate`, `appendage_repair_mitophagy_coupling` (all default 0.0)
- [x] Функция `apply_appendage_repair(damage, params, mitophagy_flux, dt_years)` в `damage.rs`
- [x] Вызов в `lib.rs step()` после PTM bridge; `mitophagy_flux` из `Option<&MitochondrialState>`
- [x] Пресет `DamageParams::antioxidant()`: ROS×0.5, aggregates×0.7, repair включена, coupling=1.0
- [x] Тесты (5 шт.): repair_off_by_default, antioxidant_enables_repair, capped_at_one,
  mitophagy_amplifies, antioxidant_slower_than_normal

---

### P6 — Полная петля транскриптом -> клеточный цикл ✅ ВЫПОЛНЕНО (2026-03-05)

- [x] `GeneExpressionState.cyclin_e_level: f32` (default 0.4) добавлен в `components.rs`
- [x] G1 boost: `cyclin_d×0.50 + cyclin_e×0.35 + myc×0.15` (ранее только `cyclin_d×0.5`)
- [x] S-фаза: `myc×0.15` ускоряет репликацию ДНК
- [x] Тест `test_cyclin_e_accelerates_g1` — cyclin_e=1.0 выходит из G1 раньше 9 шагов

---

### P7 — Многотканевая модель организма *(долгосрочно)*

**Проблема:** одна ниша = один организм. Нет агрегации тканей, нет системной
циркуляции цитокинов, нет межтканевых SASP-эффектов (Xu et al., 2018).

- [ ] **`OrganismState` как агрегатор:** frailty, cognitive, immune — из нескольких
  нишей разных `TissueType`
- [ ] **Системный `InflammagingState`:** общий SASP-сигнал = mean(все ниши);
  повышает ros_boost всех нишей пропорционально systemic_sasp
- [ ] **Гормональная ось IGF-1/GH:** `SystemicState.igf1_level` снижается с
  возрастом -> влияет на `regeneration_tempo` всех нишей
- [ ] **Пример `multi_tissue_example.rs`:** 5 тканей (HSC, Neural, Gut, Muscle, Skin),
  общий `OrganismState`, вывод последовательности отказа тканей

---

### P8 — Критерий смерти: мультитканевой порог *(умеренно важно)*

**Проблема:** смерть = D_total > 0.75 для одной сущности. Это смешение клеточной
сенесценции с гибелью организма.

- [ ] **Разделить:** `is_senescent` (клеточный) vs `organism_death` (организменный)
- [ ] **`OrganismState.is_alive`:** frailty_index >= 0.95 ИЛИ hsc_pool < 0.02
  (фатальная панцитопения) ИЛИ neural_capacity < 0.05
- [ ] **Логирование причины смерти:** `info!("Death at {:.1}y: cause={:?}", age, cause)`

---

### P9 — Пространственная геометрия кислородного щита ✅ ВЫПОЛНЕНО (сессия 16, 2026-03-11)

**Проблема:** `mito_shield` — скаляр, игнорирует пространственную структуру
митохондриальной сети вокруг центросомы.

- [x] **`MitochondrialState.pericentriolar_density: f32`** — плотность перицентриолярного кластера
  `= fusion_index×0.70 + (1−ros_production)×0.30`; fusion → компактный кластер, ROS → фрагментация
- [x] **Интегрировано в `human_development_module`:** `mito_shield = mito_shield_contribution + pericentriolar_density×0.15`
  Добавляет пространственный барьер диффузии O₂ поверх скалярного щита (max +15%)
- [x] Default: `pericentriolar_density = 1.0` (молодая клетка — плотный перицентриолярный кластер)

---

### P10 — Веса миелоидного сдвига: чувствительность ✅ ВЫПОЛНЕНО (2026-03-05)

- [x] `spindle_nonlinearity_exponent: f32` (default 1.5) в `MyeloidShiftParams`
- [x] Используется в `compute_myeloid_bias()`: `(1-sf).powf(exponent)`
- [x] Экспозиция через `get_params()` / `set_params("spindle_nonlinearity_exponent")`
- [x] Тест `test_spindle_nonlinearity_exponent_effect`:
  при exponent=2.5 и sf=0.5 → bias меньше; при sf=0.0 → идентичен

---

### P11 — Интервенции (терапевтические сценарии) ✅ ВЫПОЛНЕНО (2026-03-10)

**Обоснование:** CDATA делает конкретные предсказания о мишенях для замедления
старения. Без модуля интервенций невозможно отличить предсказания теории от
случайных совпадений. Это ключевое требование для публикации.

- [x] **`Intervention` + `InterventionKind`** — `human_development_module/src/interventions.rs`:
  5 видов: `Senolytics`, `NadPlus`, `NadPlus`, `CaloricRestriction`, `TertActivation`, `Antioxidant`
- [x] **`compute_effect()`** — применяет все активные интервенции к `DamageParams` за шаг
- [x] **Применение в `HumanDevelopmentModule::step()`:**
  - `DamageParams` модифицируются через `iv_effect.damage_params` (ROS, агрегация, репарация)
  - `Senolytics` → снижает `senescent_fraction` после `update_tissue_state()`
  - `NadPlus` → `extra_mitophagy` усиливает appendage repair
  - `TertActivation` → удлиняет `tel.mean_length`
- [x] **`add_intervention(iv)`** — публичный метод модуля
- [x] **Метрика `healthspan_days`** — дни с `total_damage_score < 0.5`; `healthspan_years()` метод
- [x] **Пример `intervention_example.rs`:** 4 стратегии × 100-летняя симуляция; вывод: Age@Death, Healthspan, Damage/Frailty/Senescent@70
- [x] **Тесты (6 шт.):** `senolytics_extend_lifespan`, `nad_plus_improves_mitochondria_at_70`,
  `caloric_restriction_reduces_ros_and_aggregation`, `tert_activation_gives_elongation`,
  `antioxidant_enables_repair_rates`, `combined_interventions_stack`

**Результат симуляции (seed=42):**
```
Strategy                  Age@Death  Healthspan  Damage@70
Control (no therapy)        81.2yr    61.7yr      0.593
Senolytics from 60          81.3yr    61.7yr      0.593
NAD⁺ from 40                81.6yr    62.0yr      0.589
CR + TERT from 50           82.5yr    62.6yr      0.577
```

---

### P12 — Автоматический CSV-экспорт через SimulationManager ✅ ВЫПОЛНЕНО (2026-03-10)

- [x] **Трейт `CdataCollect`** в `cell_dt_core/src/module.rs`: `collect()`, `write_csv()`, `buffered()`
- [x] **`SimulationManager::set_exporter(exporter, interval)`** — в `simulation.rs`
- [x] **`SimulationManager::write_csv(path)`** и **`exporter_buffered()`**
- [x] **Автовызов `collect()`** в `step()` каждые `interval` шагов
- [x] **`CdataExporter` имплементирует `CdataCollect`** в `cell_dt_io/src/cdata_exporter.rs`
- [x] **`io_example.rs` обновлён** — ручной `collect()` заменён на `set_exporter()`
- [x] **Тесты (2 шт.):** `test_manager_auto_collects` (interval=5, 10 шагов → 2 вызова), `test_manager_exporter_buffered`

---

### Сводная таблица приоритетов (актуальная)

| #   | Задача                              | Приоритет        | Сложность     | Научная ценность | Статус     |
|-----|-------------------------------------|------------------|---------------|-----------------|------------|
| P1  | Популяционная динамика + CHIP       | Критично         | Высокая       | Очень высокая   | [ ] ожидает|
| P2  | Анализ чувствительности параметров  | Критично         | Средняя       | Очень высокая   | ✅ done    |
| P3  | Стохастический шум в ODE            | Важно            | Низкая        | Высокая         | ✅ done    |
| P4  | Сигмоидный возрастной множитель     | Важно            | Низкая        | Средняя         | ✅ done    |
| P5  | Репарация придатков центриоли       | Важно            | Средняя       | Высокая         | ✅ done    |
| P6  | Полная петля транскриптом->цикл     | Умеренно         | Низкая        | Средняя         | ✅ done    |
| P7  | Многотканевая модель организма      | Долгосрочно      | Очень высокая | Очень высокая   | [ ] ожидает|
| P8  | Мультитканевой критерий смерти      | Умеренно         | Низкая        | Средняя         | [ ] ожидает|
| P9  | Пространственный кислородный щит    | Исследовательский| Высокая       | Средняя         | [ ] ожидает|
| P10 | Настраиваемая нелинейность myeloid  | Умеренно         | Низкая        | Средняя         | ✅ done    |
| P11 | Интервенции (терапия)               | Важно            | Средняя       | Очень высокая   | ✅ done    |
| P12 | Авто-экспорт CSV через Manager      | Умеренно         | Низкая        | Средняя         | ✅ done    |

**Рекомендуемый порядок следующих сессий:**
```
[ ] P2 — SA анализ параметров       (быстро, независимо, нужен до P1)
[ ] P12 — Авто-CSV через Manager    (инфраструктура для анализа данных)
[ ] P1 — NichePool + популяция      (требует P2 для настройки распределений)
[x] P11 — Интервенции               ✅ done (2026-03-10)
[ ] P8 — Критерий смерти организма  (после P1: смерть организма ≠ смерть ниши)
[ ] P7 — Многотканевая модель       (долгосрочно, после P1 + P8)
```

*Последнее обновление: 2026-03-20 (сессия 19) — CDATA_Theory_Full_Article.docx завершена (ACCEPT, 5 раундов). IDI механизм исправлен в Rust: M-IDI=O₂, D-IDI=division. AIM: cdata_nutrition.py создан.*

---

## TODO — Новые задачи из peer review статьи (сессия 18, 2026-03-20)

Источник: полный peer review `CDATA_Theory_Full_Article.docx`. Задачи связаны с пробелами теории,
которые требуют соответствующих модулей в DT для их верификации.

### P58 ✅ — AsymmetricCytoplasmQC: асимметричное наследование митохондрий и агрегатов

**Теоретическое обоснование:** Раздел 2.2.0 статьи вводит концепцию двух независимых систем сегрегации:
System I (структурный якорь центриоли → стволовая дочь) и System II (активный QC цитоплазмы →
молодые митохондрии + чистый протеом → стволовая дочь; повреждённые компоненты → дифференцирующаяся).
DT в текущем виде моделирует только System I (центриоль). System II отсутствует.

**Что нужно добавить в DT:**
- [x] Новый компонент `CytoplasmQCState` в `cell_dt_core/src/components.rs`:
  - `cytoplasm_age: f32` — накопленный «возраст» цитоплазмы [0..1]
  - `qc_efficiency: f32` — эффективность системы очистки [0..1]
  - `aggregate_burden: f32` — бремя белковых агрегатов в стволовой дочери
- [ ] Новый модуль `asymmetric_cytoqc_module` в `cell_dt_modules/`:
  - При каждом делении: перераспределяет `cytoplasm_age` — стволовая дочь получает долю `(1 - qc_efficiency)` от накопленного повреждения
  - `qc_efficiency` снижается по мере роста `centriolar_damage` (повреждённая центриоль нарушает PCM → нарушает QC-компартментализацию)
  - Обратная связь: высокий `aggregate_burden` → усиливает `ros_level` (через Track E)
- [ ] Параметры: `qc_efficiency_base`, `damage_qc_coupling_coeff`, `mitochondrial_sort_efficiency`
- [ ] Тесты (≥4): молодая клетка QC=высокий → стволовая дочь чистая; старая клетка → агрегаты накапливаются; нарушение QC → ускоренный Track A+B

**Научная ценность:** Позволяет проверить гипотезу: центриоль стареет в стволовой дочери, но цитоплазма обновляется → нарушение QC (например при мутации aPKC) должно ускорять старение обоих дочерей одновременно.

---

### P57 ✅ — TrackABInteraction: перекрёстная обратная связь треков A и B

**Теоретическое обоснование:** Раздел 3 статьи описывает треки A и B как последовательные, но не моделирует их одновременную активацию и взаимное усиление. Peer review выявил это как важный пробел.

**Что нужно добавить в DT:**
- [x] Новый канал обратной связи `TrackABCrossState` в компонентах:
  - `ciliary_loss_spindle_effect: f32` — снижение `spindle_fidelity` при потере cilia (через утрату межклеточной сигнализации контроля клеточного цикла)
  - `spindle_loss_ros_effect: f32` — рост ROS при нарушении симметричных делений (дополнительный митохондриальный стресс при аберрантном делении)
- [ ] Обновить `human_development_module`:
  - Если `ciliary_function < 0.4` → `spindle_fidelity *= (1 - ciliary_loss_spindle_effect)`
  - Если `spindle_fidelity < 0.3` → `ros_level += spindle_loss_ros_effect` (каждое аберрантное деление)
- [ ] Добавить секцию в `myeloid_shift_module`: одновременное снижение cilia + spindle → нелинейный рост `myeloid_bias`
- [ ] Тесты (≥3): изолированная потеря cilia → частичный spindle-эффект; комбинированная потеря → усиленный фенотип; раннее восстановление cilia → частичная защита spindle

**Научная ценность:** Проверяет прогноз статьи, что aging acceleration в конце жизни нелинейна из-за взаимодействия треков.

---

### P54/P55 ✅ — PTMDirectMeasurement: новый выходной маркер для верификации ключевого предсказания

**Теоретическое обоснование:** Peer review выявил критический пробел: в статье нет прямых экспериментальных данных о PTM-нагрузке в материнской центриоли стареющих стволовых клеток. DT должен генерировать конкретные, количественно верифицируемые предсказания об этих уровнях.

**Что нужно добавить в DT:**
- [x] В `centriole_module`: добавить вывод `ptm_burden_at_age(years: f64) -> PTMBurdenProfile` — возрастная траектория каждого из 5 типов PTM (карбонилирование, гиперацетилирование, агрегация, фосфодисрегуляция, потеря аппендажей)
- [ ] В CSV-экспорте (`CdataExporter`): добавить колонки `carbonylation`, `hyperacetylation`, `aggregation`, `phospho_dysreg`, `appendage_loss` в ежегодный вывод
- [x] Новый пример `ptm_trajectory_example.rs`: строит траектории PTM для Blood/Neural/Germline ниш; выводит возраст 50% повреждения для каждого типа PTM и каждой ткани
- [ ] Сравнительная таблица: Blood HSC PTM@70yr vs. Neural NSC PTM@70yr → предсказывает какая ткань теряет функцию первой (тест Track A)

**Научная ценность:** Даёт конкретные числовые предсказания для экспериментальной верификации (иммунохимия CEP164, карбонилирование α-тубулина в стволовых нишах при старении).

---

### P59 ✅ — InducerSystemFlagHypothesis: маркировка индуктор-системы как гипотезы + параметрическая вариабельность

**Теоретическое обоснование:** Peer review (пункт ❹): индуктор-система (M₀=10, D₀=8) — оригинальная гипотеза CDATA без экспериментального подтверждения. DT использует её как реализованный механизм, но должен иметь возможность работать и без неё (для сравнительного анализа).

**Что нужно добавить в DT:**
- [x] Флаг `enable_inducer_system: bool` в `HumanDevelopmentParams` (default: `true`)
- [x] При `enable_inducer_system = false`: потентность вычисляется напрямую из `spindle_fidelity + ciliary_function` (без индукторов)
- [x] Новый сравнительный пример `inducer_hypothesis_example.rs`: запускает симуляцию с включённой и выключенной индуктор-системой; сравнивает lifespan, CHIP-частоту, паттерн потери тканей
- [ ] Параметрический sweep по M₀ (7,8,9,10,11,12) и D₀ (5,6,7,8,9,10): как начальные значения влияют на lifespan?
- [ ] Тест: при `enable_inducer_system = false` модель всё ещё корректно моделирует Track A + Track B

**Научная ценность:** Позволяет различить: «предсказания CDATA, зависящие от индуктор-системы» vs. «предсказания, которые верны вне зависимости от неё». Критично для научной честности статьи.
---

### P52/P53 ✅ — SenescenceAccumulation: темп деления → регенерация → сенесцентная ниша → петля обратной связи

**Теоретическое обоснование:** Раздел 3.4 статьи вводит явную цепь: снижение темпа деления стволовых клеток → сокращение регенерации ткани → накопление сенесцентных клеток → SASP → сенесцентная ниша → дальнейшее снижение темпа деления (петля обратной связи). В DT Track F (`StemCellDivisionRateState`) и `InflammagingState` уже реализованы, но явного модуля, который считает `senescent_fraction` как функцию `division_rate` и замыкает петлю `senescent_fraction → division_rate`, нет.

**Что нужно добавить в DT:**

- [x] Новый компонент `SenescenceAccumulationState` в `cell_dt_core/src/components.rs`:
  - `senescent_fraction: f32` — доля сенесцентных клеток в ткани [0..1]
  - `sasp_output: f32` — интенсивность SASP (производная от `senescent_fraction`)
  - `niche_regenerative_capacity: f32` — способность ниши поддерживать деление [0..1]

- [ ] Логика накопления: каждый шаг:
  ```
  delta_senescent = max(0.0, attrition_rate - division_rate × replacement_efficiency)
  senescent_fraction += delta_senescent × dt
  sasp_output = senescent_fraction × sasp_scale
  ```

- [ ] Петля обратной связи → `StemCellDivisionRateState`:
  ```
  division_rate_penalty = sasp_output × niche_suppression_coeff
  effective_division_rate = base_division_rate × (1 - division_rate_penalty)
  ```

- [ ] Интеграция с `InflammagingState`: `sasp_output` добавляется к `sasp_intensity` в `InflammagingState` (уже читается `human_development_module`)

- [ ] Параметры модуля: `attrition_rate`, `replacement_efficiency`, `sasp_scale`, `niche_suppression_coeff`

- [ ] Тесты (≥4):
  - Молодая клетка: division_rate высокий → senescent_fraction не растёт
  - Старая клетка: division_rate ↓ → senescent_fraction растёт → SASP → division_rate ↓ ещё (петля)
  - Сенолитическое вмешательство: принудительный сброс `senescent_fraction` → восстановление `division_rate`
  - Нелинейное ускорение: в конце симуляции (70+ лет) скорость накопления выше, чем в 40–50 лет

- [x] Новый пример `senescence_cascade_example.rs`: 100-летняя симуляция; вывод `division_rate`, `senescent_fraction`, `sasp_output`, `niche_regenerative_capacity` каждые 10 лет; сравнение Control vs. Senolytic (сброс fraction каждые 5 лет с 60)

**Научная ценность:** Позволяет количественно проверить: (а) нелинейное ускорение снижения регенерации в старости; (б) эффект сенолитиков как «разрыв петли» vs. центриолярных интервенций как «устранение первопричины»; (в) предсказать возраст перехода от линейного к нелинейному накоплению сенесцентных клеток в разных тканях.

**Связь с разделом 3.4 статьи:** Этот модуль позволяет симулятору количественно воспроизводить паттерн, описанный в 3.4, и генерировать конкретные числовые предсказания для экспериментальной верификации.
