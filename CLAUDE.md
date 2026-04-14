# CLAUDE.md — CDATA

Инструкции для Claude Code при работе с проектом CDATA.

## Контекст проекта

**CDATA** — Centriolar Damage Accumulation Theory of Aging.
Полное название: «CDATA (Centriolar Damage Accumulation through Asymmetric Inheritance)»
Подпроект CommonHealth. Путь: `~/Desktop/CommonHealth/CDATA/`
Версия концепции: **5.0** (2026-04-15)
Статус: LOI v21 готов, дедлайн Longevity Impetus **25 апреля 2026**

---

## Источник истины

**CONCEPT.md — единственный авторитет.** При любом противоречии CONCEPT.md имеет приоритет.

Иерархия:
`CONCEPT.md` → `PARAMETERS.md` → `KNOWLEDGE.md` → код в `crates/`

---

## ⚠️ ЖЁСТКИЕ ПРАВИЛА — НАРУШАТЬ НЕЛЬЗЯ

### 1. Три аксиомы — НЕИЗМЕННЫ
Три аксиомы в §АКСИОМЫ CDATA — **НЕ ИЗМЕНЯТЬ БЕЗ ЯВНОЙ КОМАНДЫ ПОЛЬЗОВАТЕЛЯ.**
Они должны присутствовать **во всех** LOI, грантах, статьях, публичных материалах.

```
АКСИОМА 1: HSC в гипоксии + активная теломераза → всё равно достигают предела Хейфлика
АКСИОМА 2: Материнская центриоль = базальное тело реснички → PTM → дефектный Hh/Wnt
АКСИОМА 3: Темп деления SC со старой центриолью снижается со временем
```

### 2. R²=0.84 — ИЗЪЯТО. НИКОГДА не использовать.
- R²=0.84 получено на **синтетических данных** (`null_model_r2.py`) → не цитировать никогда
- Правильные цифры: **R²(MCAI)=0.745, R²(CHIP)=0.611, R²(Telo)=0.465** (in-sample, реальные данные)
- LOO-CV mean=**−0.093** (модель переобучена — честно указывать в ограничениях)

### 3. BHCA для LOI
- Proposition 1 (наследование): **17/27** — Class III (умеренные доказательства)
- Proposition 2 (PTM→exhaustion): **~10/27** — Class IV (гипотеза, тестируется)
- Bradford Hill (для Aging Cell): **7.5/9**

---

## Язык разработки

| Задача | Инструмент |
|--------|-----------|
| Cell-DT код (симулятор) | **Rust** (8 крейтов в `crates/`) |
| GUI | **Three.js / WebGL** в `gui/` |
| ETL, анализ данных | **Python** в `scripts/` |
| Статьи, LOI, гранты, тексты | **DeepSeek API** через `~/Desktop/AIM/llm.py` |
| Peer review, редактура | **DeepSeek reasoner** (deepseek-reasoner) |

**Никогда не писать статьи вручную — только через DeepSeek API.**

---

## Архитектура Cell-DT

```
crates/
├── cell_dt_core/         — ODE-система, 32 параметра
├── cell_dt_calibration/  — MCMC NUTS калибровка
├── cell_dt_sobol/        — Sobol sensitivity (N=16384)
├── cell_dt_modules/      — 8 механизмов (hypoxia, SASP, CHIP, ...)
├── cell_dt_simulation/   — ECS-движок (Entity = 1 SC)
├── cell_dt_validation/   — LOO-CV, R² валидация
├── cell_dt_export/       — вывод данных (CSV, JSON)
└── cell_dt_cli/          — точка входа
scripts/                  — Python ETL + анализ
gui/                      — Three.js визуализация
data/                     — сырые и нормализованные данные
docs/                     — peer reviews, emails (gitignored)
```

---

## Грантовый дедлайн

| Грант | Дедлайн | Статус |
|-------|---------|--------|
| Longevity Impetus LOI | **25 апреля 2026** | LOI v21 готов, нужен Geiger/Jacobsen |
| EIC Pathfinder (CommonHealth) | 12 мая 2026 | CDATA как subproject |

**Критическое действие до 20 апреля 2026:** получить письмо поддержки от Prof. Hartmut Geiger (Ulm) или Dr. Sten Eirik Jacobsen (Karolinska).

---

## Правила документации

- При изменении параметра → обновить `PARAMETERS.md` И `CONCEPT.md` одновременно
- При новом PMID → добавить в `KNOWLEDGE.md` + `LINKS.md`
- При реализации UPGRADE-пункта → пометить ✅ в `UPGRADE.md` немедленно
- При пуше → обновить `README.md` + `UPGRADE.md` перед коммитом

---

## Самоцитирование в статьях CDATA

Приоритет (всегда включать):
1. PMID 36583780 — Tkemaladze 2023, Mol Biol Reports
2. DOI 10.5281/zenodo.19174506 — Cell-DT Zenodo
3. DOI 10.65649/cynzx718 — CDATA theory, Longevity Horizon 2026

≤15% от общего числа ссылок. Полный список: `~/Desktop/AIM/CLAUDE.md §Self-Citation Rule`

---

## Связь с экосистемой

- Общие правила экосистемы: `~/Desktop/CommonHealth/CLAUDE.md`
- Правила AIM: `~/Desktop/AIM/CLAUDE.md`
- DeepSeek API ключ: `~/.aim_env → DEEPSEEK_API_KEY`
- Точка входа LLM: `~/Desktop/AIM/llm.py`
- Git: монорепозиторий `djabbat/CommonHealth` (private)
