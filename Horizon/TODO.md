# EIC Pathfinder Open 2026 — CDATA Project TODO

## 📌 Правило: язык программирования по умолчанию

**Если нет явного указания на конкретный язык — писать код на Rust.**
Если другой язык объективно лучше для задачи — сначала предложить и обосновать, и только после подтверждения писать код.

---

## 📌 Правило: DeepSeek для текстовых задач

**Если задача подходит DeepSeek — использовать DeepSeek API, не делать вручную.**

| Категория | Примеры |
|-----------|---------|
| **Текст / статьи** | написать статью, раздел, введение, обсуждение |
| **Перевод** | научный, медицинский, художественный текст |
| **Рецензирование** | peer review, ответ рецензентам, cover letter |
| **Гранты / документы** | грант, питч, меморандум, резюме, абстракт |
| **Редактура** | полировка текста, стиль, академический английский |
| **Пациенты (AIM)** | объяснить диагноз, назначение, анализы — понятным языком |
| **Код** | объяснить код, docstrings, code review, тесты, SQL |
| **kSystem** | статьи лексикона на 8 языках |
| **Kartvely** | главы книги, анализ исторических источников |
| **Space** | описания упражнений на 4 языках |
| **Regenesis** | протоколы, клинические обоснования |
| **ŠamnuAzuzi** | либретто на др. языках, программные заметки |
| **Переписка** | письма инвесторам, деловые email, ответы на замечания EIC |

**Ключ:** `~/.aim_env → DEEPSEEK_API_KEY` · **Модели:** `deepseek-chat` (быстро) · `deepseek-reasoner` (сложно)


---

**Дедлайн подачи: 12 мая 2026**
**Обновлено: 2026-03-23**

---

## ✅ Сделано

- [x] Стратегия Horizon Europe (HORIZON_GRANT_STRATEGY.md)
- [x] **Part B задрафтован** — EIC_Pathfinder_CDATA_PartB.md (полный, ~30 стр.)
  - Раздел 1: Excellence (научная основа CDATA, методология WP1-4)
  - Раздел 2: Impact (KPIs, EU policy, dissemination)
  - Раздел 3: Quality (Work Plan, Gantt, таблицы Deliverables/Milestones, бюджет €2.5M, Риски)
  - Раздел 4: Ethics (informed consent, GDPR, Helsinki)
- [x] EIC_Pathfinder_CDATA_PartB.docx (62 KB — финальный документ)
- [x] EIC_Pathfinder_CDATA_PartB.pdf (119 KB — для просмотра)
- [x] EIC_Pathfinder_CDATA_Application.docx (предыдущая версия заявки, Peer Review 4.3/5)
- [x] - [x] Бюджет €2,500K (прямые €2,000K + 25% косвенные €500K)
- [x] WP структура: WP1 €800K / WP2 €450K / WP3 €400K / WP4 €350K
- [x] Когорта n=288 enrolled / n=240 evaluable
- [x] Peer Review v3 выполнен: 4.3/5 — «Рекомендуется к финансированию»
- [x] **Peer Review v4 выполнен: 3.73/5** — 4 критических проблемы (C1-C4) → `Materials/PeerReview_v4_2026-03-24.md`
- [x] **Part B v5 написан 2026-03-24** → `EIC_Pathfinder_CDATA_PartB_v5.md`
  - Ze Theory убрана из Section 1 (остаётся только как secondary endpoint WP2)
  - Consortium Agreement исправлен: M1 (не M18)
  - [Applicant Institution: TBD] description расширен
  - WP4: добавлена T4.0 preparatory task (M6-M18)
  - In silico caveats добавлены к intervention table

---

## 🔴 P0 — КРИТИЧНО (блокирует подачу)
*(по результатам Peer Review v4, 2026-03-24)*

- ⏸ **PIC регистрация [Applicant Institution: TBD]** — *не зависит от Dr. Jaba, административный процесс*
  → Когда будет готово: сообщить Claude Code для обновления Part A

- 🔴 **[C1] co-PI — НУЖЕН НОВЫЙ** (2026-03-26: предыдущий co-PI вышел из проекта)
  → Все упоминания имени убраны из всех документов (v7, v6, v5, peer review)
  → Варианты замены: Prof. Nino Lomidze ([TBD institution] team) — уточнить у Jabы
  → До подачи: найти нового co-PI, вставить имя + CV + Letter of Support

- [x] **[C2] Удалить «biblical patriarchal dataset, n=26»** ✅ v5/v7
  → 1.2.3: Ze Theory = exploratory secondary endpoint WP2; «no prior cohort validation assumed»

- [x] **[C3] Исправить противоречие Consortium Agreement: M1 vs. T4.1** ✅ v5
  → T4.1 = M1: Consortium Agreement signed; IP policy agreed at project start

- [x] **[C4] Укрепить описание [Applicant Institution: TBD] (раздел 3.2.1)** ✅ v5
  → Добавлены: юридический статус (non-profit, Registration No. [TBC]), track record 50 лет, admin capacity (SAP-compatible accounting, certified auditor)

---

## 🟡 P1 — Важно (апрель 2026)

- [ ] Загрузить CDATA как preprint на bioRxiv
  → Без preprint заявка выглядит слабее; EIC ценит TRL evidence
  → Использовать контент из CDATA_Theory_Final.docx

- [ ] Letter of Intent от Georgian National Centre for Disease Control
  → Нужен для Sustainability (раздел 2.2 + WP2)

- [ ] Подтвердить или убрать Estonian Biobank (University of Tartu) из Sustainability

- [ ] Уточнить статус Annals of Rejuvenation Science: ISSN, индексация DOAJ/PubMed
  → В заявке указан DOI 10.65649/yx9sn772 — нужно подтвердить что журнал реальный

- [ ] Добавить ссылки в Table 1 сравнения теорий:
  → López-Otín 2023 (Cell 186:243); Blackburn 2000 (Nature 408:53)

---

## 🟢 P2 — До 1 мая 2026

- [ ] Finalize Part B: объединить EIC_Pathfinder_CDATA_PartB.docx с EIC_Pathfinder_CDATA_Application.docx
  → Part B — это техническое описание (уже готово)
  → Part A — формы (бюджет, административные данные) — заполняются в EU Funding Portal онлайн

- [ ] Заполнить Part A онлайн в EU Funding Portal
  → Нужен PIC (см. P0 выше)
  → Административные данные, бюджетная таблица, консорциум

- [ ] Ethics Self-Assessment (отдельная форма в портале)
  → Содержание из Part B Section 4 уже готово

- [ ] Написать краткое резюме проекта (Abstract, 2000 символов)
  → На основе «CDATA in a Nutshell» box из Part B

- [ ] CV Principal Investigator (Dr. Jaba Tkemaladze) — стандартный Horizon формат
  → 2 pages max; включить PMID 36583780, DOI 10.65649/yx9sn772, Cell-DT

- [ ] CV нового co-PI — после подтверждения имени

---

## 📋 Структура финальной заявки (что нужно загрузить)

| Документ | Статус | Файл |
|---------|--------|------|
| Part A (формы портала) | ❌ Нужен PIC | Заполняется онлайн |
| **Part B (Technical Description)** | ✅ ГОТОВО | EIC_Pathfinder_CDATA_PartB.docx |
| Ethics Self-Assessment | ✅ В Part B Section 4 | Часть Part B |
| CV Principal Investigator | ❌ Нужно составить | — |
| CV co-PI | ❌ После Гелы | — |
| Letter of Support  | ❌ Нужно получить | — |
| Letter of Support ([Applicant Institution: TBD]) | ❌ Нужно получить | — |

---

## 📞 Ключевые контакты

| Организация | Контакт | Зачем |
|------------|---------|-------|
| Horizon Georgia NCP | a.goletiani@mes.gov.ge | Консультация, партнёры |
| EU Funding Portal | ec.europa.eu | PIC регистрация |
| EIC helpdesk | eic@eismea.eu | Вопросы по Pathfinder |
| [TBD institution] BME | Через Гелу | co-PI подтверждение |
| GNSF (Rustaveli) | gnsf.ge | Параллельный грант (track record) |

---

## 💡 Параллельно: Rustaveli Foundation

Подать внутренний грузинский грант **параллельно** с EIC — строит track record:
- Тема: «CDATA: вычислительная модель клеточного старения»
- Бюджет: ~40,000 GEL (~$15K)
- Сайт: gnsf.ge
- **Победа важна:** proof of concept для будущих EIC и Horizon заявок

---

## 🔵 ЮРИДИЧЕСКОЕ СОПРОВОЖДЕНИЕ — James Wolff / Greenspoon Marder LLP

**Статус:** ✅ Контакт установлен | Звонок: **27 марта 2026, 21:00 GMT+4 (13:00 EST)**

### Контекст переписки (март 2026)
- **James A. Wolff** — Partner, Greenspoon Marder LLP (New York)
  - Direct: +1 212.524.4978 | James.Wolff@gmlaw.com
  - Paralegal: Sophia Im, Sophia.Im@gmlaw.com | +1 212.524.5011
- Специализация: emerging tech, capital formation, IP strategy, longevity med-tech
- **Связь с A4LI** (Alliance for Longevity Initiatives, DC) — помог создать организацию
- **Сеть финансирования:** VCs + broker-dealers в longevity science

### Стратегия Dr. Jaba (подтверждена в переписке)
1. **Фаза 1 (сейчас):** Horizon Europe → validation → IP foundation
2. **Фаза 2:** Biotech/pharma licensing
3. **Фаза 3:** Seed/Pre-Seed raise ($300K+) → expanded validation

### Вопросы для звонка 27.03.2026
- [ ] IP structuring: как защитить CAII + CentrosomeTransplant ДО подачи Horizon
- [ ] Deal terms: структура biotech/pharma licensing agreement
- [ ] Fundraising: Seed/Pre-Seed структура (LLC vs C-corp для US investors)
- [ ] Horizon + IP: как условия гранта влияют на будущее private equity / licensing exit
- [ ] **Компания:** CDATA Research LLC (intended) — регистрация и юрисдикция

### Данные для conflict check (уже отправлены)
- Полное имя: Jaba Tkemaladze
- Компания: CDATA Research LLC (intended)
- Адрес: 46 Rustaveli, Resort Abastumani, Georgia

### Следующие шаги
- [ ] Подготовить brief: Horizon grant structure + IP considerations → до 27.03.2026
- [ ] **ОТЛОЖЕНО: ответное письмо James Wolff отправить в ЧЕТВЕРГ 26.03.2026** (не раньше)
- [ ] Получить prospective client letter и engagement letter от Wolff (ожидается)
- [ ] После звонка: решить юрисдикцию LLC / структуру IP
- [x] **Part B v6 написан 2026-03-24** → `EIC_Pathfinder_CDATA_PartB_v6.md`
  - [E1] Ze Theory возвращена в 1.2.3 как secondary exploratory hypothesis (без n=26)
  - [Q2] STED исправлен: "acquisition of new Leica SP8" (не upgrade от Zeiss)
  - [Q3] Blinding добавлен в WP2 T2.2 и T2.4
  - [E3] HDF клетки добавлены в WP1 T1.1 как основная модель
  - [E2] Теломерная теория: смягчена до "explains replicative senescence only"
  - [I1] KPI #10: r>0.65 → r>0.5; KPI #12: healthspan → proof-of-mechanism
  - [Q4] Rust Engineer: international recruitment добавлен в T3.1
  - [I2] CAII ELISA: план на Roche Cobas в M30-M36 добавлен в 2.3.3
