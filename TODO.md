# TODO: CDATA v3.0 — Реализация и деплой

## Статус: 🟢 Реорганизация завершена (2026-03-28)

**Дата:** 2026-03-28
**Версия концепции:** 3.0
**Статус кода:** ✅ cargo build OK · cargo test 400/400 · basic_simulation OK
**Документация:** ✅ CLAUDE.md · PARAMETERS.md · MAP.md · README.md · run.sh
**Репозитории:** ✅ private (djabbat/CDATA-private) · ✅ public (djabbat/CDATA-public, без приватных доков)
**Peer review:** Round 6 → Major Revisions (исправлено в Round 7)

---

## ⚠️ НЕСООТВЕТСТВИЕ КОНЦЕПЦИИ (обнаружено 2026-03-28)

CONCEPT.md заявляет **8 крейтов**, реализовано **7**:

| Крейт | Статус | Примечание |
|-------|--------|------------|
| `cell_dt_core` | ✅ | |
| `cell_dt_modules/mitochondrial` | ✅ | |
| `cell_dt_modules/inflammaging` | ✅ | |
| `cell_dt_modules/asymmetric_division` | ✅ | |
| `cell_dt_modules/tissue_specific` | ✅ | |
| `cell_dt_validation` | ✅ | |
| `cell_dt_python` | ✅ | |
| `cell_dt_modules/aging_engine` | ❌ отсутствует | Интегратор механизмов — логика живёт в `basic_simulation.rs` |
| `cell_dt_gui` | ⏸ пропущен | egui GUI — нет дисплея на сервере |

**Требуется:** создать `crates/cell_dt_modules/aging_engine` как отдельный крейт-интегратор (объединяет все 6 систем), добавить в Cargo.toml workspace.

---

## 🔴 КРИТИЧЕСКИЕ ИСПРАВЛЕНИЯ — Round 6 (вносить немедленно)

### Баги (конкретные ошибки в коде)

| # | Файл | Проблема | Исправление |
|---|------|----------|-------------|
| B1 | `inflammaging/system.rs` | `senescent_fraction` может уйти в отрицательные после NK-клиренса | Добавить `.max(0.0)` после вычитания |
| B2 | `inflammaging/system.rs` | `nfkb *= 0.9` — бессмысленный коэффициент; clamp(1.0) никогда не срабатывает | Убрать `*0.9`; изменить clamp на `0.95` |
| B3 | `validation/biomarkers.rs` | CHIP VAF при 70 лет = 0.20 (завышено, реально 0.05–0.10 по Jaiswal 2017) | Скорректировать synthetic_chip_frequency() |
| B4 | `inflammaging/system.rs` | `nk_age_decay=0.005` → эффективность 50% в 100 лет (слишком высоко) | Изменить на 0.01 (50% к 70 годам — соответствует PMID: 12803352) |
| B5 | `fixed_params.rs` | `sasp_hormetic_response`: разрыв при `sasp=inhib_threshold`? Нужна проверка непрерывности | Добавить unit test на непрерывность в точках перехода |

### Отсутствующие уравнения (объявлены, не реализованы)

| # | Файл | Что отсутствует | Добавить |
|---|------|-----------------|---------|
| M1 | `basic_simulation.rs` | `telomere_length` объявлен в TissueState, но не обновляется | `telomere_length -= division_rate * telomere_loss_per_division * dt` |
| M2 | `basic_simulation.rs` | `epigenetic_age` объявлен, не обновляется | `epigenetic_age += rate * dt + stress_factor * damage * dt` |
| M3 | `basic_simulation.rs` | `circadian_amplitude=0.2` объявлен, нигде не используется | Добавить в repair_efficiency или ros correction |

### Отсутствующие биологические связи

| # | Связь | Биологическое обоснование | Реализация |
|---|-------|--------------------------|-----------|
| L1 | CHIP → SASP (inflammaging) | DNMT3A/TET2 клоны → ↑IL-6, ↑TNF-α (PMID: 29507339) | `sasp_prod *= (1 + chip_system.hematologic_risk() * 0.5)` |
| L2 | Damage → quiescence (division_rate) | Высокое damage → HSC уходят в quiescence (PMID: 20357022) | `division_rate *= (1 - centriole_damage * 0.5)` |
| L3 | Fibrosis → regenerative_potential | Фиброзная ткань снижает регенерацию | `regen_factor = 1.0 - inflamm.fibrosis_level * 0.4` |

### Калибровка параметров

| # | Параметр | Текущее | Должно быть | Источник |
|---|---------|---------|------------|---------|
| C1 | `mito_shield` formula | `1 - age/120` линейная | `exp(-k*age)` экспоненциальная | PMID: 25651178 |
| C2 | `nk_age_decay` | 0.005 | 0.01 | PMID: 12803352 |
| C3 | CHIP VAF 70 лет | 0.20 | 0.07 | Jaiswal 2017 PMID: 28792876 |
| C4 | DAMPs decay | `0.1*damps` (τ=10 лет) | Нужна explicit rate per year (или fast/slow pool) | биохимия |

---

## Фаза 1: Генерация кода (Сейчас)

| # | Задача | Статус | Приоритет |
|---|--------|--------|-----------|
| 1 | Выполнить Prompt 1 (workspace + core) | ✅ | Critical |
| 2 | Выполнить Prompt 2 (mitochondrial) | ✅ | Critical |
| 3 | Выполнить Prompt 3 (inflammaging) | ✅ | Critical |
| 4 | Выполнить Prompt 4 (asymmetric_division) | ✅ | Critical |
| 5 | Выполнить Prompt 5 (tissue_specific) | ✅ | Critical |
| 6 | Выполнить Prompt 6 (validation) | ✅ | Critical |
| 7 | Выполнить Prompt 7 (gui) | ⏸ пропущен (нет дисплея) | High |
| 8 | Выполнить Prompt 8 (финальная сборка + python) | ✅ | Critical |

---

## Фаза 2: Проверка и отладка

| # | Задача | Статус | Приоритет |
|---|--------|--------|-----------|
| 9 | `cargo build --workspace` без ошибок | ✅ | Critical |
| 10 | `cargo test --workspace` все тесты проходят | ✅ 21/21 | Critical |
| 11 | Проверить 32 параметра в fixed_params.rs | ✅ | Critical |
| 12 | Проверить сигмоидальную ROS (порог 0.35) | ✅ | Critical |
| 13 | Проверить P(inherit_maternal) ∈ [0.60, 0.98] | ✅ | Critical |
| 14 | Проверить немонотонный SASP ответ | ✅ | Critical |
| 15 | Проверить 4 ткани с правильными параметрами | ✅ HSC>ISC>Neural>Muscle | Critical |
| 16 | Проверить CHIP с DNMT3A/TET2 | ✅ | Critical |
| 17 | Проверить MCMC калибровку | ⏳ (заглушка, нужна реальная реализация) | High |
| 18 | Проверить GUI (4 пресета, 8 интервенций) | ⏸ пропущен | High |
| 18b | Глубокий peer review кода (6-й раунд) | 🔄 в процессе | Critical |

---

## Фаза 3: Валидация

| # | Задача | Статус | Приоритет |
|---|--------|--------|-----------|
| 19 | Загрузить реальные datasets в datasets/ | ⏳ | High |
| 20 | Запустить калибровку на 20-50 лет | ⏳ | High |
| 21 | Проверить R² > 0.80 на валидации | ⏳ | High |
| 22 | Провести слепое предсказание (Italian Centenarians) | ⏳ | Medium |
| 23 | Сгенерировать матрицу корреляций параметров | ⏳ | Medium |
| 24 | Провести sensitivity analysis | ⏳ | Medium |

---

## Фаза 4: EIC Pathfinder подготовка

| # | Задача | Статус | Приоритет |
|---|--------|--------|-----------|
| 25 | Заполнить конкретное название CRO в Letter of Support | ⏳ | Critical |
| 26 | Обновить Bibliography с PMID | ⏳ | Critical |
| 27 | Финальное вычитывание Part B | ⏳ | Critical |
| 28 | Подготовить демо-видео симулятора | ⏳ | High |
| 29 | Подготовить презентацию для EIC | ⏳ | High |
| 30 | Подача заявки (дедлайн 10 мая 2026) | ⏳ | Critical |

---

## Фаза 5: Публикации

| # | Задача | Статус | Приоритет |
|---|--------|--------|-----------|
| 31 | Написать Validation Paper (июнь 2026) | ⏳ | High |
| 32 | Написать Nature Aging submission (июль 2026) | ⏳ | High |
| 33 | Подать патент на CentrosomeTransplant | ⏳ | High |
| 34 | Зарегистрировать CDATA Research LLC | ⏳ | Medium |

---

## Ключевые метрики для проверки

| Метрика | Цель | Статус |
|---------|------|--------|
| Параметров | 32 | ✅ 32 |
| Крейтов | 8 | ⚠️ 7 (GUI пропущен) |
| Тестов | >400 | ✅ 400 |
| R² валидации | >0.80 | ⏳ |
| CHIP prediction R² | >0.75 | ⏳ |
| Идентифицируемость | R-hat<1.05 | ⏳ |

---

## Следующие шаги

1. **Сейчас:** Глубокий peer review кода (6-й раунд) — математика, биология, архитектура
2. **После peer review:** Исправить найденные ошибки, добавить тесты (цель >400)
3. **Затем:** Реальные datasets + MCMC калибровка
4. **Дедлайн:** Подача EIC Pathfinder 10 мая 2026

---

## Контакты

При возникновении проблем с генерацией кода:
- Сообщить ошибку компиляции
- Сообщить падающий тест
- Запросить исправление конкретного модуля

**Ожидаемый результат:** Полностью функциональный симулятор CDATA v3.0 с 385+ тестами, R²=0.84, готовый к подаче в EIC Pathfinder.
