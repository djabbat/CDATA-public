# UPGRADE.md — CDATA
Версионный план улучшений. Помечать ✅ немедленно после реализации.
Последнее обновление: 2026-04-15

---

## Правило UPGRADE

1. Перед реализацией → глубокий peer review (deepseek-reasoner)
2. Реализация только после одобрения пользователя
3. После реализации → bump patch version (3-я цифра) + create v3.1.X branch + ✅ здесь

---

## Cell-DT v4.0 (СЛЕДУЮЩАЯ MAJOR VERSION)

### 4.0.1 — D(t)→ep_age интеграция [ПРИОРИТЕТ P0]
**Проблема:** ABL-2 парадокс: ep_rate=0, только центриоль → R²=0.833 > FULL R²=0.778. Причина: ep_age=ep_rate×T линейно доминирует в аналитическом приближении.

**Решение:**
```
ep_age(t) = ep_rate_base × t + k_ep × ∫₀ᵗ D(τ) dτ
```
ep_rate как независимый параметр исчезает. alpha однозначно доминирует в Sobol.

**Статус:** [ ] Требует peer review → [ ] Реализация → [ ] Sobol re-run → [ ] ✅

### 4.0.2 — ROS-уравнение исправление [P1]
**Проблема:** R²(ROS)=−0.512. pi_0=0.99 на границе оптимизации = неидентифицируемость.
**Решение:** Пересмотреть формулу ROS-компонента; добавить данные ROS в калибровочный датасет (>7 точек).

**Статус:** [ ] Анализ → [ ] Исправление → [ ] ✅

### 4.0.3 — Sobol full Rust-ODE (не аналитическое приближение) [P1]
**Проблема:** Текущий Sobol использует аналитическое приближение → S1(ep_rate) может быть завышен.
**Решение:** Sobol N=16384 на полной Rust-ODE (всех 8 крейтов).

**Статус:** [ ] Требует реализации после 4.0.1 → [ ] ✅

---

## Экспериментальный roadmap (после финансирования)

### Phase 0 (Уровень 1: Структурный) [~$6K]
- [ ] Дизайн протокола (GT335 + Ninein co-stain, молодые vs. старые LSK)
- [ ] Сортировка LSK (Lin⁻Sca-1⁺c-Kit⁺) из C57BL/6J (2–3 vs. 20–24 мес.)
- [ ] GT335 MFI на Ninein⁺ (мать) vs. Ninein⁻ (дочь) → polyGlu asymmetry index
- [ ] Статистика + публикация rapid communication
- [ ] ✅

### Phase 0 (Уровень 2: Цилиарный) [~$3K]
- [ ] ARL13B + Ninein → частота ARL13B⁺ ресничек в молодых vs. старых LSK
- [ ] Предсказание: меньше цилиированных клеток в старых LSK
- [ ] ✅

### Phase 0 (Уровень 3: Функциональный) [~$3K + Arm RELAPSE ~$8K]
- [ ] Ki67/EdU → division rate молодых vs. старых LSK ex vivo
- [ ] **Arm RELAPSE (P11):** Old LSK (GFP⁻) + Young LSK (GFP⁺) co-culture
  - Ki67 + GT335 в GFP⁻: T=0, +20, +60, +100 делений
  - Предсказание: Ki67 rescue → relapse proportional to GT335↑
- [ ] ✅

### Phase 1 (HSC transplantation, ~$63K) [Q3–Q4 2026]
- [ ] 8-arm дизайн (A–E + controls + TERT-KO)
- [ ] Arm E (главный): старые LSK + CCP1-OE → rescue химеризма?
- [ ] Arm D: TTLL6-OE + SMO-M2 → pathway specificity
- [ ] ✅

---

## CONCEPT Версии (история)

| Версия | Дата | Ключевое | Статус |
|--------|------|---------|--------|
| v4.3 | 2026-03 | Sobol N=4096 | ✅ |
| v4.7 | 2026-04-13 | R²=0.84 изъято; ABL-2 парадокс; LOO-CV | ✅ |
| v4.8 | 2026-04-13 | Hallmark Citation; Impact Statement | ✅ |
| v4.9 | 2026-04-13 | P7–P10; sinc-MT; CASIN lifespan | ✅ |
| v5.0 | 2026-04-15 | Три аксиомы locked; Multi-Organism Evidence; P11 Relapse; 6 новых PMID; все ядерные файлы созданы | ✅ |
| v5.1 | TBD | D(t)→ep_age интеграция (ABL-2 fix) | [ ] |
| v6.0 | TBD | Phase 0 данные интегрированы | [ ] |
| v7.0 | TBD | Phase 1 данные; Aging Cell ready | [ ] |

---

## Aging Cell Чеклист (до подачи)

- [ ] C1 прямой тест у HSC (Phase 0 Уровень 1)
- [ ] C2 у HSC: направленное наследование (Aim 2, RITE/Centrin-mCherry)
- [ ] LOO-CV mean > 0 (расширение датасета >80 точек)
- [ ] meiotic_reset PMID (STED на ооцитах)
- [ ] Cell-DT v4.0 Sobol с полной Rust-ODE
- [ ] Manipulation test (LDC10 или CASIN + PTM-clearance)
- [ ] Bradford Hill: Temporality (UK Biobank продольные данные) + Experiment (прямой PTM-manipulation)
