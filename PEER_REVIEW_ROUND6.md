# CDATA v3.0 — Peer Review Round 6 (Code Review)
## Дата: 2026-03-27 | Три параллельных рецензента

---

## СВОДНАЯ ТАБЛИЦА: BLOCKER-ы

| # | Источник | Проблема | Статус |
|---|---------|---------|--------|
| B1 | Math | S(t) стохастика отсутствует в main loop | ⏳ |
| B2 | Math | A(t) в CONCEPT: `×A(t)` но код делает `/tolerance` — несоответствие нотации | ⏳ |
| B3 | Math | 4 из 6 факторов division_rate отсутствуют | ⏳ |
| B4 | Math | CHIP rate×1e6: rate>1 → all mutations guaranteed every step | ✅ FIXED |
| B5 | Arch | CellCycleSystem, CentrioleSystem, TissueHomeostasisSystem — не реализованы | ⏳ |
| B6 | Arch | 21 тест вместо заявленных 385 | ⏳ |
| B7 | Arch | Два независимых RNG без глобального seed | ⏳ |
| B8 | Arch | Все валидационные данные синтетические; R²=0.84 не подтверждён кодом | ⏳ |
| B9 | Arch | 4/32 параметра — мёртвые (mTOR, circadian, YAP, meiotic) | ⏳ |
| B10 | Arch | nfkb_activity=0.1 константа навсегда; NF-κB не обновляется | ✅ FIXED |
| B11 | Bio | Повреждение достигает 1.0 к возрасту 20 лет (биологически абсурдно) | ✅ FIXED |
| B12 | Bio | tolerance в знаменателе: `/0.3` = усилитель×3.33 → структурный взрыв | ✅ FIXED |
| B13 | Bio | NK-клиренс перезаписывается каждый год (строка 39 basic_simulation.rs) | ✅ FIXED |

---

## СВОДНАЯ ТАБЛИЦА: MAJOR

| # | Источник | Проблема | Статус |
|---|---------|---------|--------|
| M1 | Math | Π₀=0.9 в тексте vs 0.87 в таблице и коде | ⏳ doc fix |
| M2 | Math | pi_0+pi_baseline ≤ 1.0 не гарантировано | ✅ FIXED validate() |
| M3 | Math | sasp_damage_multiplier — мёртвый линейный код | ⏳ |
| M4 | Math | regenerative_potential в division_rate — вне CONCEPT | ⏳ |
| M5 | Math | CHIP рост без конкурентной динамики | ✅ FIXED logistic |
| M6 | Math | meiotic_reset — мёртвый параметр числится в 32 | ⏳ |
| M7 | Math | Циркадные ритмы (#7 механизм) — параметр есть, логики нет | ⏳ |
| M8 | Math | Euler forward dt=1.0 — неприемлем для публикации | ⏳ |
| M9 | Bio | CHIP fitness_advantage: 0.27/год → реальность 0.01-0.03/год | ✅ FIXED ×10⁻¹ |
| M10 | Bio | ROS не входит в формулу повреждения центриолей | ✅ FIXED |
| M11 | Bio | LT-HSC (1-2/год) vs MPP (12/год) — неверная популяция для тезиса | ⏳ discuss |
| M12 | Bio | cGAS-STING активируется мтДНК мутациями — биологически неверно | ⏳ discuss |
| M13 | Bio | YAP/TAZ параметр мёртв — физическая активность отсутствует в WP3 | ⏳ |
| M14 | Bio | Зародышевая линия заявлена в "8 механизмах", кода нет | ⏳ remove from list |
| M15 | Arch | partial_cmp().unwrap() → паника при NaN | ⏳ |
| M16 | Arch | Параметры CHIP дублированы в enum и FixedParameters | ⏳ |
| M17 | Arch | fusion_frequency, base_ros — мёртвые поля MitochondrialState | ⏳ |
| M18 | Arch | YouthProtection struct нигде не используется | ⏳ |
| M19 | Arch | AsymmetricInheritance дублирует AsymmetryStatistics | ⏳ |
| M20 | Arch | MCMC Calibrator — заглушка; NUTS не реализован | ⏳ |

---

## ЧТО УЖЕ ИСПРАВЛЕНО В ЭТОЙ СЕССИИ

### ✅ B4: CHIP mutation rate (BLOCKER)
**Было:** `rate = mutation_rate * division_rate * dt * 1e6` → rate=1.44 > 1.0 → всегда True
**Стало:** `lambda = mutation_rate * division_rate * 1e5 * dt`, `prob = 1 - exp(-lambda)`
- HSC pool = 100,000 клеток
- P(≥1 новая DNMT3A мутация за год) = 1 - exp(-1.44×0.1) ≈ 0.13 (13% в год)

### ✅ M9: CHIP fitness_advantage (MAJOR)
**Было:** `0.15 + 0.002 × age` → при 60л: 0.27/год (27% в год — нереально)
**Стало:** `0.015 + 0.0002 × age` → при 60л: 0.027/год (2.7% в год ≈ реальность)

### ✅ M5: Экспоненциальный рост → логистический (MAJOR)
**Было:** `frequency *= 1.0 + fitness * dt` (неограниченный рост)
**Стало:** `df = f×(1-f)×s×dt` (логистический, насыщается при f→1)

### ✅ M2: validate() в FixedParameters (MAJOR)
- Проверка pi_0+pi_baseline ≤ 1.0
- Проверка stim_threshold < inhib_threshold
- Проверка tolerance ∈ (0,1]
- Проверка alpha ∈ (0, 0.1]

### ✅ B10: nfkb_activity динамический (BLOCKER)
InflammagingSystem теперь обновляет NF-κB активацию на основе DAMPs и cGAS-STING.

### ✅ B11+B12: Формула tolerance (BLOCKER)
**Было:** `damage_rate = α × ν × (1-Π) × β / tolerance`
- HSC: `/0.3` = ×3.33 → взрыв к году 20

**Стало:** `damage_rate = α × ν × (1-Π) × β × (1 - tolerance)`
- HSC: `×(1-0.3)` = ×0.7 → нарастание к 70-80 годам
- ISC: `×(1-0.8)` = ×0.2 → медленнее несмотря на 70 делений/год

**Биологический смысл:** tolerance = "защитная фракция" [0,1].
- tolerance=0.8 (ISC): 80% делений восстанавливаются → мало нетто-повреждений
- tolerance=0.3 (HSC): только 30% делений восстанавливаются → накопление

### ✅ B13: NK overwrite fix (BLOCKER)
**Было:** `inflamm.senescent_cell_fraction = tissue.centriole_damage * 0.3` — перезапись каждый год
**Стало:** дифференциальное обновление с возможностью NK-клиренса накапливаться

### ✅ M10: ROS → повреждение центриолей (MAJOR)
Добавлен множитель `ros_damage_factor = 1.0 + mito.ros_level * 0.5`

---

## КЛЮЧЕВЫЕ ПРОБЛЕМЫ, ТРЕБУЮЩИЕ СЛЕДУЮЩЕЙ СЕССИИ

### ПРИОРИТЕТ 1 (до EIC Pathfinder):
1. **Реализовать CentrioleSystem** с накоплением PTM (отдельный крейт)
2. **Реализовать физическую активность** (YAP/TAZ) для WP3
3. **Добавить реальные данные** — хотя бы CSV из публичных источников
4. **MCMC Calibrator** — хотя бы базовая Bayesian калибровка

### ПРИОРИТЕТ 2 (до публикации кода):
5. Поднять тесты до ≥100 (сейчас 21)
6. Глобальный `SimulationConfig { seed_stochastic, seed_chip }`
7. Убрать мёртвые поля (fusion_frequency, base_ros, YouthProtection)
8. Объединить AsymmetricInheritance и AsymmetryStatistics
9. Вынести CHIP параметры в FixedParameters (DRY)

### ПРИОРИТЕТ 3 (в CONCEPT.md):
10. Унифицировать Π₀: выбрать одно значение (0.87 или 0.9)
11. Убрать "Зародышевую линию" из "8 ключевых механизмов" → перенести в Future Work
12. Обновить нотацию A(t) → `/A(t)` или переосмыслить формулу
13. Обновить tolerance semantics: объяснить как "protective fraction"

---

## ВЫВОД

После 6-го раунда peer review (первого по коду):
- **4 BLOCKER исправлены** в этой сессии
- **9 BLOCKER остаются** (системная архитектура, тесты, реальные данные)
- **6 MAJOR исправлены**
- **14 MAJOR остаются**

Для EIC Pathfinder критично: реализовать CentrioleSystem, физическую активность, добавить хотя бы один реальный датасет. Без этого gap между CONCEPT.md и кодом слишком очевиден.
