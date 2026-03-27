# External Expert Peer Review — EIC Pathfinder Open 2026
## CDATA Application — Version v5.0
### Date: 2026-03-24 | Source: External Expert (anonymous)

---

## Общая оценка
Это исключительно сильная, амбициозная и хорошо структурированная заявка, полностью соответствующая духу EIC Pathfinder Open. Гипотеза CDATA предлагает радикально новый, механистический взгляд на природу старения.

Ключевым преимуществом является наличие уже работающего вычислительного прототипа (Cell-DT). Консорциум из двух организаций (Грузия) — правильный выбор для Pathfinder.

---

## 1. Excellence — 4.5/5

### Сильные стороны
- Оригинальность гипотезы: чёткий, фальсифицируемый механизм
- Cell-DT: Rust-модуль с 439 тестами — редкость для заявок такого уровня
- Интеграция масштабов: PTM → клинические фенотипы

### Критические замечания

**[E1] Ze Theory — нужно ВЕРНУТЬ в раздел 1.2 (не удалять)**
- Проблема: удаление Ze Theory из Excellence делает проект "более плоским"
- EIC Pathfinder поощряет смелые гипотезы вне мейнстрима
- Рекомендация: вернуть в 1.2 (Scientific Originality) как вторичную инновационную гипотезу — HRV-биомаркер, будет проверяться в WP2

**[E2] Сравнительная таблица — смягчить формулировку по теломерной теории**
- "Cannot explain post-mitotic decline" → "Explains only replicative senescence, not post-mitotic dysfunction"
- Добавить сноску: CDATA дополняет теломерную теорию

**[E3] Клеточные линии WP1 — добавить первичные HDF**
- HeLa — иммортализованная раковая линия с анеуплоидией; центриолярный аппарат нефизиологичен
- Рекомендация: добавить первичные фибробласты кожи (HDF) как основную модель; HeLa — только для отработки протоколов

---

## 2. Impact — 4.0/5

### Сильные стороны
- Чёткая связь с EU Healthier Together, EHDS
- Реалистичный путь к коммерциализации (CAII, CentrosomeTransplant)
- Widening country (Грузия) обоснован

### Замечания

**[I1] KPIs — завышены**
- KPI #10: "Pearson r > 0.65" → снизить до "r > 0.5"
- KPI #12: "Healthspan extension confirmed" → "Proof-of-mechanism confirmed for at least one intervention"

**[I2] CAII → клиническое внедрение**
- Добавить в 2.3.3: план перевода CAII ELISA на автоматизированный анализатор (Roche Cobas) в M30–M36

---

## 3. Quality & Implementation — 3.5/5

### Сильные стороны
- Детальный план WP/Tasks/Milestones/Gantt
- Корректный бюджет и обоснование STED, CRO мониторинга
- Реалистичный Risk Register

### КРИТИЧЕСКИЕ ЗАМЕЧАНИЯ (требуют немедленного исправления)

**[Q1] ❌ КРИТИЧНО: Отсутствует имя co-PI**
- Эксперты EIC крайне негативно к "пустым" именам
- Действие: Prof. Lomidze (fallback) или заведующий кафедрой [TBD institution] + письмо поддержки

**[Q2] ❌ КРИТИЧНО: Техническая ошибка в описании STED микроскопа**
- "STED upgrade to Leica SP8" от Zeiss LSM 700 — технически НЕВОЗМОЖНО (разные производители)
- Исправление: "Acquisition of a new Leica SP8 STED microscope (€120K) at [TBD institution]. The existing Zeiss LSM 700 will continue to be used for standard confocal imaging."

**[Q3] ❌ КРИТИЧНО: Отсутствует ослепление (blinding) в WP2**
- Стандарт GCP для клинических исследований
- Добавить в T2.2 и T2.4: "Biomarker analysis (CAII, CEP164, telomere length) will be performed blinded to clinical phenotype data. Unblinding will occur only after the final dataset is locked."

**[Q4] Rust Engineer — найм**
- 36 PM Rust Engineer в Грузии может быть сложно
- Добавить в T3.1: "International recruitment open to EU/Associated Country candidates"

---

## Итоговые рекомендации

### 🔴 High priority (critical)
1. Указать имя co-PI в [TBD institution]
2. Исправить STED описание (Acquisition, не upgrade)
3. Добавить blinding в WP2

### 🟡 Medium priority (recommended)
4. Вернуть Ze Theory в раздел 1.2 как вторичную гипотезу
5. Скорректировать KPIs (#10: r > 0.5; #12: proof-of-mechanism)
6. Добавить HDF клетки в WP1

### 🟢 Low priority (nice to have)
7. Смягчить критику теломерной теории
8. Добавить план CAII ELISA на автоматизированной платформе (Roche Cobas)

---

## Вердикт
«Заявка имеет очень высокий потенциал. После исправления критических замечаний (особенно Q1 и Q2) документ будет соответствовать уровню успешных проектов EIC Pathfinder Open.»

---
*Сохранён: 2026-03-24 | Применить в: EIC_Pathfinder_CDATA_PartB_v6.md*
