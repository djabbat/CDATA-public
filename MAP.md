# MAP.md — CDATA
Карта проекта: структура, гранты, публикации, конкуренты.
Дата: 2026-04-15 | CONCEPT v5.0

---

## Структура файлов

```
CommonHealth/CDATA/
├── CONCEPT.md          ← АВТОРИТЕТ (v5.0, 2026-04-15)
├── README.md           ← публичное описание
├── CLAUDE.md           ← инструкции для Claude Code
├── TODO.md             ← текущие задачи
├── PARAMETERS.md       ← все числовые параметры
├── MAP.md              ← этот файл
├── MEMORY.md           ← ключевые решения и контекст
├── LINKS.md            ← внешние ссылки, PMID, DOI
├── KNOWLEDGE.md        ← база знаний (научные данные)
├── UPGRADE.md          ← план версий и прогресс
│
├── Cargo.toml          ← Rust workspace
├── run.sh              ← запуск симулятора
│
├── crates/             ← Rust исходный код
│   ├── cell_dt_core/
│   ├── cell_dt_calibration/
│   ├── cell_dt_sobol/
│   ├── cell_dt_modules/
│   │   └── mitochondrial/  ← гипоксийный модуль (v3.4)
│   ├── cell_dt_simulation/
│   ├── cell_dt_validation/
│   ├── cell_dt_export/
│   └── cell_dt_cli/
│
├── gui/                ← Three.js / WebGL визуализация
├── scripts/            ← Python ETL, анализ, md_to_docx
└── data/               ← сырые и нормализованные данные
    ├── calibration/    ← реальные литературные данные (28 точек)
    └── synthetic/      ← синтетика (НЕ для валидации!)
```

**Gitignore (public repo):** CONCEPT.md, CLAUDE.md, TODO.md, PARAMETERS.md, MAP.md, MEMORY.md, LINKS.md, KNOWLEDGE.md, UPGRADE.md, docs/

---

## Грантовая карта

```
2026-04-25  Longevity Impetus LOI v21
            $75K, фаза 0 + фаза 1
            PI: Jaba Tkemaladze
            Co-PI (стратегический): Liz Parrish (BioViva) ✅ CONFIRMED
            Co-PI (экспериментальный): Hartmut Geiger (Ulm) ⏳ PENDING
            Хост: Georgia Longevity Alliance + TSMU vivarium
            Поддержка: Aubrey de Grey (встреча 18.04.2026)

2026-05-12  EIC Pathfinder (CommonHealth)
            CDATA как experimental subtrack
            Бюджет CDATA-компонента: TBD
```

---

## Публикационная карта

```
СЕЙЧАС:
  LOI v21 (Impetus) ──► peer review ──► submission 25.04.2026
  CDATA Theory (Longevity Horizon, 2026) — опубликовано DOI 10.65649/cynzx718

СЛЕДУЮЩИЙ ШАГ (после Phase 0 данных):
  Phase 0 данные (GT335+Ninein, ARL13B, Ki67, co-culture) ──► rapid communication
  Цель: Cell Stem Cell Brief Report или Aging Cell Short Report

ЦЕЛЬ (после Phase 1):
  Full paper ──► Aging Cell (IF ~8, Q1)
  BHCA: нужны C1+C2 у HSC + manipulation test

ПАРАЛЛЕЛЬНО:
  Cell-DT v4.0 ──► Methods paper ──► Bioinformatics или PLOS Comput Biol
```

---

## Конкурентная карта

| Лаборатория | Риск | Уникальность CDATA |
|-------------|------|--------------------|
| **Morrison lab** (UTSouthwestern) | ВЫСОКИЙ — C2 у HSC | ODE-модель + CCP1/AGBL5 терапевтическая ось |
| **Bhatt lab** (Stanford) | СРЕДНИЙ — GT335-STED | Математическая формализация, multi-scale |
| **Yamashita lab** (UCSF) | СРЕДНИЙ — ACD механизм | PTM-количественный аспект, Phase 0 HSC |
| **Altos Labs** | НИЗКИЙ — reprogramming | Relapse prediction (P11) = тестируемая альтернатива |

---

## Трёхуровневая модель

```
Уровень 1 — Клеточный:
  D_cell(t) = 1 − exp(−r·n_divisions)

Уровень 2 — Популяционный:
  P(failure|D) = σ(D − D_crit, k=5) → Fraction_failed(t)

Уровень 3 — Тканевый:
  Stem_pool(t) = N₀ × (1 − Fraction_failed(t))
  Renewal_rate(t) = Stem_pool(t) × ν_eff(t)

Уровень 4 — Организменный:
  MCAI(t) = 0.40·D_norm + 0.25·SASP_norm + 0.20·(1−SC_pool_norm)
           + 0.10·(1−Telo_diff_norm) + 0.05·CHIP_VAF_norm
```

---

## Experimental Roadmap

```
Phase 0 (~$12K, месяцы 1–3):
  Уровень 1: GT335 + Ninein → polyGlu asymmetry index (молодые vs. старые LSK)
  Уровень 2: ARL13B → частота первичных ресничек
  Уровень 3: Ki67/EdU → division rate; Arm RELAPSE (co-culture, P11)
  Результат: первые HSC-специфичные данные CDATA → rapid communication

Phase 1 (~$63K, месяцы 4–12):
  Arm A: Молодые LSK + TTLL6-OE → ↑polyGlu → ↓химеризм?
  Arm B: Молодые LSK + CCP1-OE → ↓polyGlu → ↑химеризм?
  Arm C: TTLL6-dead-enzyme → контроль специфичности
  Arm D: TTLL6-OE + SMO-M2 → rescue через Hh?
  Arm E: Старые LSK + CCP1-OE → rescue? (ГЛАВНЫЙ ТЕСТ АКСИОМЫ 3)
  Control-young, Control-aged, TERT-KO (counter-control)

Дизайн: serial BMT chimera; химеризм CD45.1/CD45.2; 16 недель
Статистика: LMM; n=15/arm; power=0.89 для d≥1.05
```
