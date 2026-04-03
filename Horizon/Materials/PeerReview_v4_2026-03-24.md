# Peer Review — EIC Pathfinder Open 2026
## CDATA: Centriolar Damage Accumulation Theory of Ageing
### Reviewer: Claude Code | Version: v4 | Date: 2026-03-24

---

## ИТОГОВАЯ ОЦЕНКА

| Критерий | Оценка (0–5) | Порог | Статус |
|---------|-------------|-------|--------|
| **1. Excellence** | **3.8 / 5** | 3.5 | ✅ Выше порога |
| **2. Impact** | **4.0 / 5** | 3.5 | ✅ Выше порога |
| **3. Quality & Implementation** | **3.4 / 5** | 3.5 | ⚠️ На грани |
| **ИТОГО** | **3.73 / 5** | — | ⚠️ Риск отклонения |

**Вердикт:** «Рекомендуется к финансированию при условии устранения критических слабостей.»
Без устранения проблем P0 (ниже) — высокий риск попадания ниже порога по разделу 3.

---

## 1. EXCELLENCE (3.8/5)

### Сильные стороны

**1.1 Концептуальная оригинальность — ОТЛИЧНО**
- Гипотеза CDATA чёткая, смелая и ложнофальсифицируемая — три явных falsification criteria (1.2.1). Это именно то, что ищет EIC Pathfinder.
- «Missing root cause» framing убедителен: полемика с Hallmarks (описательная таксономия vs. механистическая причина) — сильный аргумент.
- Cell-DT: 14 модулей ECS, 439 тестов, предсказание 78.4 года — впечатляюще конкретно. Calibrated output с CV=5.6% показывает зрелость модели.
- Asymmetric retention rule (стволовая клетка сохраняет *старший* центриоль) — элегантный механизм с молекулярным обоснованием.

**1.2 Методология WP1 — ХОРОШО**
- Панель клеточных моделей разнообразна (HeLa / IMR90 / CD34⁺ EPC).
- U-ExM для наноструктуры центриолей — современный метод (Gambarotto 2019).
- CAII как композитный биомаркер (4 протеина аппендиксов) — новаторски и технически реализуемо.

**1.3 WP2 дизайн — ХОРОШО**
- n=288/240 — статистически обоснован (OR=2.5, α=0.05, power=0.80, требуется n=196 → запас 23%).
- 5 вторичных эндпоинтов страхуют от провала первичного.
- Fried Frailty Index + MMSE + grip + gait — стандартные, признанные инструменты.

---

### Слабые стороны Excellence

#### ❌ КРИТИЧНО-1: «Biblical patriarchal dataset, n=26»

> «Ze Theory predicts HRV-derived longevity markers with r = 0.78 in historical genealogical cohorts **(biblical patriarchal dataset, n=26)**»

Это самая опасная фраза во всей заявке. Для EIC-рецензента, не знакомого с Ze Theory, это прозвучит как псевдонаука. Два удара одновременно: (a) «библейский датасет» — не признанный научный источник; (b) n=26 — статистически ничтожно для валидационного утверждения r=0.78.

**Как исправить:** Убрать «biblical patriarchal dataset» полностью. Оставить Ze Theory как теоретическую рамку (не как валидированный инструмент). Если есть иные данные HRV — использовать их. Формулировка должна быть: «Ze Theory provides a candidate non-invasive biomarker (5-min ECG); clinical validation is a secondary endpoint of WP2 (Ze HRV v* proximity score).»

#### ⚠️ ВАЖНО-1: Annals of Rejuvenation Science — статус журнала

> DOI: 10.65649/yx9sn772 — «Annals of Rejuvenation Science»

Рецензент проверит журнал. Если он не индексирован в PubMed/Scopus/DOAJ — это серьёзный минус для credibility. Нужно либо: (a) подать рукопись на bioRxiv как preprint (что усиливает заявку независимо), (b) переформулировать как «preprint / institutional publication» или (c) убрать DOI из ключевых референсов и оставить только PMID 36583780.

**Рекомендация:** bioRxiv preprint CDATA до подачи заявки — это P1 в TODO и это ВАЖНО реализовать до 12 мая.

#### ⚠️ ВАЖНО-2: Ze Theory — чрезмерный акцент в разделе 1

Раздел 1.2.3 «Ze Theory Integration: Cross-Disciplinary Novelty» занимает целый подраздел в части об оригинальности. Рецензент-биогеронтолог не знаком с Ze Theory и воспримет её упоминание как отвлечение от основного нарратива. CDATA сильна сама по себе — Ze Theory в разделе Excellence ослабляет, а не усиливает.

**Как исправить:** Переместить Ze Theory из 1.2.3 в раздел 1.1.7 (WP2 биомаркеры) как «дополнительный биомаркерный кандидат». В 1.2 (Ambition) заменить на что-то более стандартное: например, multi-scale integration (от нм PTM до мс ЭКГ — уже хорошая концепция без «Ze»).

---

## 2. IMPACT (4.0/5)

### Сильные стороны

- KPI-таблица (15 измеримых показателей) — образцово конкретна.
- Widening country (Грузия) правильно и убедительно подан.
- Ссылка на EU Healthier Together + EHDS — точное попадание в политический контекст.
- Журналы-цели названы с IF (Aging Cell 9.9, npj Aging, PLOS CB, Nature Aging) — убедительно.
- Exploitation roadmap (M1→M36) структурирован.
- $600B market projection — правдоподобно со ссылкой.

### Слабые стороны Impact

#### ⚠️ ВАЖНО-3: «+18.3 years healthspan gain» — завышенное утверждение

> «CentrosomeTransplant... +18.3 years (simulation)»

Рецензент увидит: компьютерная симуляция предсказывает +18 лет. Это прозвучит как маркетинг, а не наука. EIC Pathfinder оценивает смелость, но не безответственность.

**Как исправить:** Добавить caveat в каждую строку таблицы: «*In silico prediction from Cell-DT stochastic simulation; experimental validation is the objective of WP4*». Это честно и защищает заявку.

#### ✏️ МИНОР-1: Letter of Intent от Georgian NCDC

TODO отмечает эту LoI как нужную (P1). В заявке написано: используется для Sustainability (2.2 + WP2). Если LoI не получена к подаче — удалить или заменить на «recruitment via Tbilisi polyclinics (12 sites) under WP2 ethics protocol». LoI от официальной организации здравоохранения — хорошо, но её отсутствие не критично.

---

## 3. QUALITY AND EFFICIENCY OF IMPLEMENTATION (3.4/5)

Это самый слабый раздел — и именно здесь риск провала.

### Сильные стороны

- WP-логика (WP1→WP2→WP3→WP4) очевидна и хорошо обоснована.
- Deliverables и Milestones таблицы — полные и конкретные.
- Budget breakdown по WP и статьям — детальный.
- Risk register: 5 рисков с mitigation — хорошо.
- Open Science: FAIR data, EOSC, MIT licence, Zenodo — всё правильно.
- [TBD institution] track record (2025–2028, GA #101216703) — сильный аргумент для credibility.

---

### Слабые стороны Implementation

#### ❌ КРИТИЧНО-2: «[TBD]»

Это самая очевидная формальная слабость. Рецензент видит: соисполнитель заявки, ответственный за WP1 (€800K) и WP2 (€450K) = €1.25M из €2M прямых затрат, — не назван. Это не соответствует требованиям EIC: все ключевые сотрудники должны быть идентифицированы.

**Действия (срочно):**
- Получить подтверждение от Гелы ИЛИ использовать альтернативу: Prof. Nino Lomidze (уже указана в рисках R1 как fallback).
- Вставить: полное имя, должность, 3-5 ключевых публикаций, роль в.
- Если Гела не отвечает — переключиться на Lomidze немедленно (до 15 апреля, как указано в START.md).

#### ❌ КРИТИЧНО-3: Consortium Agreement — Milestone M1 vs. Task T4.1 (M18) — ПРОТИВОРЕЧИЕ

В Milestones:
> M1: Consortium Agreement signed | WP4 | **M1**

В Task T4.1:
> T4.1 (M18–M24): Consortium Agreement finalised

Это прямое противоречие. Рецензент его заметит. По стандартам Horizon Europe Consortium Agreement должен быть подписан **до начала проекта (M0/M1)** — иначе непонятно, как регулируются IP права в месяцах 1–17.

**Как исправить:** T4.1 переформулировать: «T4.1 (M1): Consortium Agreement signed (Milestone M1). T4.1b (M3–M18): IP policy detailed implementation; Data Management Plan (M3).»

#### ❌ КРИТИЧНО-4: [Applicant Institution: TBD] — слабое описание как Lead Beneficiary

> «Founded: 1974, Poti, Georgia. Scientific-cultural organisation with interdisciplinary research tradition.»

Это единственное описание Lead Beneficiary с бюджетом €500K+. По сравнению с [TBD institution] (которому посвящён детальный параграф с оборудованием, PIC) — разительный контраст. Рецензент спросит: «Что это за организация? Какой у неё track record в научных проектах?»

**Как исправить:** Добавить параграф:
- Официальный юридический статус [Applicant Institution: TBD] в Грузии (НКО / научная организация — статья устава)
- Предыдущие проекты / гранты (даже небольшие)
- Инфраструктура (офис, серверная мощность для Cell-DT)
- Подтверждённая администрация (CFO, legal officer) для управления €500K
- Если есть письмо поддержки от президента — вставить ссылку

#### ⚠️ ВАЖНО-4: Gantt chart — milestone row некорректна

```
Milestones    M1    M3    M5  M2  M3  M4  M5,M6            M7  M8      M9
```

Milestones расположены НЕ в хронологическом порядке: «M5 M2 M3 M4» — это не имеет смысла. Рецензент увидит небрежность.

**Как исправить:** Указать milestones в формате «▼M1 ▼M2 ▼M3...» в правильных позициях на шкале времени.

#### ⚠️ ВАЖНО-5: WP4 start — M18 слишком поздно для PoC

WP4 (Therapeutic PoC) начинается в M18. Это означает, что конкретные терапевтические эксперименты начнутся через 1.5 года после старта. Рецензент может усомниться: успеет ли команда получить значимые результаты за оставшиеся 18 месяцев (M18–M36)?

**Как исправить:** Добавить в WP4 preparatory task T4.0 (M6–M18): «Cell line preparation and preliminary dose-finding for top-3 interventions» — это показывает, что работа по WP4 начинается сразу, а M18 — это начало основных экспериментов.

#### ✏️ МИНОР-2: Table 1 — ссылки на López-Otín и Blackburn

TODO отмечает: «Добавить ссылки в Table 1 сравнения теорий». В тексте ссылки [López-Otín 2023] и [Blackburn 2000] упоминаются, но в самой таблице (строки) они не стоят в формате [1][2]. Следует добавить citation markers прямо в ячейки таблицы.

#### ✏️ МИНОР-3: Estonian Biobank — убрать или подтвердить

TODO: «Подтвердить или убрать Estonian Biobank (University of Tartu) из Sustainability». В тексте Part B эта организация не найдена — возможно уже убрана. Если где-то осталась — нужно удалить, так как без официального LoI её наличие ослабляет, а не усиливает.

#### ✏️ МИНОР-4: WP2 budget — subcontracting €60K за CRO

Оправдание CRO («ICH E6(R2) GCP compliance») — правильное. Но рецензент может спросить: «Почему не включить монитора в персонал [TBD institution]?» Добавить 1 предложение: «Independent CRO monitoring is mandatory for GCP-compliant clinical studies per ICH E6(R2) §5.18 and cannot be performed by the study site.»

---

## 4. ОБЩИЕ РЕКОМЕНДАЦИИ

### 🔴 P0 — До подачи (блокирующие)

| # | Проблема | Действие |
|---|---------|---------|
| C1 | Unnamed co-PI «[TBD]» | Получить имя ИЛИ переключиться на Prof. Lomidze |
| C2 | «Biblical patriarchal dataset, n=26» | Убрать из текста заявки |
| C3 | Consortium Agreement противоречие M1 vs. M18 | Исправить T4.1 → M1 |
| C4 | [Applicant Institution: TBD] — thin description | Добавить параграф с track record и инфраструктурой |

### 🟡 P1 — До 01 мая (важные)

| # | Проблема | Действие |
|---|---------|---------|
| I1 | Ann Rejuvenation Sci — статус | Подать CDATA на bioRxiv как preprint (→ DOI) |
| I2 | Ze Theory в Excellence 1.2.3 | Переместить в WP2 биомаркерный список |
| I3 | «+18.3 years» без caveat | Добавить «*in silico prediction; WP4 objective*» |
| I4 | Gantt milestone row некорректна | Исправить хронологический порядок |
| I5 | WP4 preparatory task | Добавить T4.0 (M6–M18) подготовительный |

### 🟢 P2 — Желательно

| # | Проблема | Действие |
|---|---------|---------|
| M1 | Table 1 — добавить citation markers | [1][2] в ячейки |
| M2 | Estonian Biobank — проверить и убрать | Если осталась — удалить |
| M3 | CRO subcontracting — добавить justification | 1 предложение об ICH E6(R2) §5.18 |
| M4 | LoI от Georgian NCDC | Получить если возможно |
| M5 | CV PI — составить в Horizon формате | 2 стр., PMID + DOI + Cell-DT |

---

## 5. СРАВНЕНИЕ С ПРЕДЫДУЩИМИ ВЕРСИЯМИ

| Версия | Оценка | Ключевые изменения |
|--------|--------|-------------------|
| v1 (Application.docx) | ~3.5/5 | Первичный черновик |
| v2 | ~3.8/5 | Бюджет исправлен |
| v3 | 4.3/5 | n=288/240, PM-таблица, [TBD institution] PIC |
| **v4 (текущая)** | **3.73/5** | Более строгий peer review; выявлены C1-C4 |

> Примечание: снижение оценки с v3 (4.3) до v4 (3.73) связано не с ухудшением заявки, а с более детальным анализом раздела 3 (Implementation) по стандартам EIC.

---

## 6. СРАВНЕНИЕ С EIC PATHFINDER BENCHMARK

На основе публичных статистик EIC Pathfinder 2024:
- Порог финансирования: ~4.0/5 в среднем (конкурс ~8%)
- Заявки с 4.3/5 попадают в финансирование при отсутствии fatal flaws
- Заявки с 3.7/5 — в «reserve list» (финансируются если бюджет позволяет)

**Вывод:** Устранение C1-C4 поднимет оценку до ~4.2-4.4/5 и переведёт заявку из «reserve» в «funded» зону.

---

*Peer Review v4 | Claude Code | 2026-03-24*
*Следующий шаг: внести исправления C1-C4 в EIC_Pathfinder_CDATA_PartB.md*
