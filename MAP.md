# CDATA v3.0 — Карта параметров и взаимосвязей
_Дата: 2026-03-27 | Round 7 | deepseek-reasoner analysis_

---

## Часть 1 — Описание всех параметров

### 1.1 Базовые (FixedParameters)

| Параметр | Значение | Биологический смысл | Размерность |
|----------|----------|---------------------|-------------|
| `alpha` | 0.0082 | Вероятность необратимого повреждения центриоли за одно деление при нулевой защите | 1/деление |
| `hayflick_limit` | 50 | Предел Хейфлика — максимальное число делений клетки (фибробласты; для HSC ~50–100) | делений |
| `base_ros_young` | 0.12 | Базовый уровень ROS в молодом организме (нормировано 0–1) | безразм. |

### 1.2 Защита молодости (Youth Protection)

| Параметр | Значение | Биологический смысл |
|----------|----------|---------------------|
| `pi_0` | 0.87 | Амплитуда защиты в момент рождения: Π(0) = pi_0 + pi_baseline = 0.97 |
| `tau_protection` | 24.3 | Постоянная времени спада защиты (лет): в 24 лет защита снижается в e раз |
| `pi_baseline` | 0.10 | Остаточная защита в глубокой старости (не падает ниже 10%) |
| **Формула** | | `Π(age) = pi_0 × exp(-age/tau_protection) + pi_baseline` |

### 1.3 Асимметричное деление

| Параметр | Значение | Биологический смысл |
|----------|----------|---------------------|
| `p0_inheritance` | 0.94 | Вероятность унаследовать материнскую (старую) центриоль в молодости |
| `age_decline_rate` | 0.15 | Скорость снижения вероятности с возрастом |
| `fidelity_loss` | 0.10 | Потеря вероятности при снижении точности митотического веретена |
| **Формула** | | `p = p0 − 0.15×(age/100) − 0.10×(1−fidelity), clamp[0.60, 0.98]` |

### 1.4 SASP Гормезис (нелинейный ответ)

| Параметр | Значение | Биологический смысл |
|----------|----------|---------------------|
| `stim_threshold` | 0.3 | Порог SASP ниже которого воспаление стимулирует регенерацию |
| `inhib_threshold` | 0.8 | Порог выше которого воспаление подавляет регенерацию |
| `max_stimulation` | 1.5 | Максимальный стимулирующий эффект низкого SASP на деление (×1.5) |
| `max_inhibition` | 0.3 | Минимальный ингибирующий эффект высокого SASP (×0.3... не используется напрямую) |
| **Функция** | | 3-фазная: линейный рост → плато → гиперболический спад |

### 1.5 CHIP мутации

| Параметр | Значение | Биологический смысл |
|----------|----------|---------------------|
| `dnmt3a_fitness` | 0.15 | Базовое селективное преимущество DNMT3A клона (без возрастного вклада) |
| `dnmt3a_age_slope` | 0.002 | Прирост преимущества на год жизни: при 60 лет s=0.015+0.002×60=0.027/год |
| `tet2_fitness` | 0.12 | То же для TET2 |
| `tet2_age_slope` | 0.0015 | То же для TET2 |
| **Рост клона** | | Логистическая модель Морана: df = f×(1−f)×s×dt |

### 1.6 Тканевые параметры

| Ткань | ν (дел/год) | β (мультипл.) | τ (толерантность) | Эф. старение ν×β×(1−τ) |
|-------|------------|---------------|-------------------|------------------------|
| HSC (Гемопоэтические) | 12 | 1.0 | 0.3 | **8.4** |
| ISC (Кишечные) | 70 | 0.3 | 0.8 | 4.2 |
| Muscle (Мышечные) | 4 | 1.2 | 0.5 | 2.4 |
| Neural (Нейронные) | 2 | 1.5 | 0.2 | **2.4** |

_HSC стареет быстрее всего несмотря на меньшую частоту делений — парадокс разрешён через толерантность_

### 1.7 Прочие / "Dead parameters" (объявлены, не интегрированы)

| Параметр | Значение | Потенциальная роль | Статус |
|----------|----------|--------------------|--------|
| `mtor_activity` | 0.7 | Регуляция аутофагии, синтеза белка, пролиферации | ❌ Dead |
| `circadian_amplitude` | 0.2 | Суточные колебания репарации ДНК и ROS | ⚠️ Placeholder |
| `meiotic_reset` | 0.8 | Эпигенетическое омоложение зародышевой линии | ❌ Dead |
| `yap_taz_sensitivity` | 0.5 | Механотрансдукция, пролиферация стволовых клеток | ❌ Dead |

### 1.8 Системные параметры (InflammagingParams, MitochondrialParams)

| Параметр | Значение | Смысл |
|----------|----------|-------|
| `damps_rate` | 0.05 | Скорость накопления DAMPs от сенесцентных клеток и повреждений |
| `cgas_sensitivity` | 0.8 | Чувствительность cGAS-STING к DAMPs |
| `sasp_decay` | 0.1 | Скорость распада SASP (τ = 10 лет в dt=1год; биологически слишком медленно) |
| `nk_age_decay` | **0.010** | Скорость снижения NK-эффективности с возрастом (Round 7 fix: 0.005→0.010) |
| `fibrosis_rate` | 0.02 | Скорость накопления фиброза под влиянием SASP |
| `mitophagy_threshold` | 0.35 | Порог ROS, выше которого митофагия активируется |
| `ros_steepness` | 10.0 | Крутизна сигмоидной кривой ROS |

---

## Часть 2 — Взаимосвязи параметров

### 2.1 Матрица влияния (кто → что → знак → сила)

```
ПАРАМЕТР/ПЕРЕМЕННАЯ     → ВЛИЯЕТ НА              ЗНАК  СИЛА    МЕХАНИЗМ
─────────────────────────────────────────────────────────────────────────
age                     → protection (Π)          –     сильн   exp(-age/tau)
pi_0, tau, pi_baseline  → protection               +     средн   прямые слагаемые
protection              → damage_rate              –     сильн   (1-protection)
alpha                   → damage_rate              +     сильн   прямой множитель
nu (тканевый)           → division_rate            +     сильн   линейный
beta (тканевый)         → damage_rate              +     сильн   линейный
tolerance (tau)         → damage_rate              –     средн   (1-tolerance)
division_rate           → damage_rate              +     сильн   прямое
division_rate           → telomere                 –     средн   укорочение на деление
damage                  → senescent_fraction       +     средн   *0.05 per year
damage                  → epigenetic_age            +     средн   стресс-ускорение
damage                  → quiescence_factor         –     средн   (1-damage*0.5) Round7
ros                     → damage_rate              +     сильн   ros_damage_factor=1+ros*0.5
ros                     → mtdna                    +     средн   0.001*ros²*dt
mtdna                   → ros                      +     сильн   sigmoid(mtdna+sasp*0.3)
sasp                    → ros (indirect)           +     средн   через oxidative input
sasp                    → nfkb                     +     средн   вес 0.3
sasp                    → division_rate             +/–   средн   гормезис (3-фазный)
sasp                    → frailty                   +     средн   вес 0.3
sasp                    → fibrosis                  +     слабо   via fibrosis_rate
senescent               → sasp_prod                +     сильн   тройное произведение
senescent               → damps                    +     средн   источник DAMPs
damps                   → cgas_sting               +     средн   *cgas_sensitivity
cgas_sting              → nfkb                     +     средн   вес 0.6
cgas_sting              → sasp_prod                +     сильн   тройное произведение
nfkb                    → sasp_prod                +     сильн   тройное произведение
nk_efficiency           → senescent (clearance)    –     средн   *0.1*senescent*dt
nk_age_decay            → nk_efficiency            –     средн   (1-age*decay)
fibrosis                → regen_factor              –     средн   (1-fibrosis*0.4) Round7
telomere (short)        → frailty                   +     слабо   вес 0.1
```

### 2.2 Петли обратной связи

#### Положительные (+): ускоряют старение

```
Петля A: ROS-MTDNA замкнутый цикл
  ros → mtdna_mutations → ros
  (ros_level=sigmoid(mtdna+sasp*0.3); mtdna+=0.001*ros²*dt)
  Характер: квадратичный — при высоком ros нарастает стремительно

Петля B: Главная воспалительная петля
  damage → senescent → cgas → nfkb → sasp → nfkb (autostimulation *0.3)
  → саsp → ros → damage
  Характер: петля может застрять в "воспалительном фенотипе"

Петля C: DAMPs-SASP автоусиление
  damps → cgas → nfkb → sasp → senescent → damps
  Характер: положительная, со временем насыщается из-за clamp(0..1)

Петля D: Прогрессия CHIP под действием воспаления
  sasp → sasp_boost в CHIP fitness → clone_expansion → hematologic_risk
  (после Round 7 L1: CHIP→SASP coupling добавлено в TODO)
```

#### Отрицательные (–): стабилизируют, замедляют

```
Петля E: Иммунный клиренс
  sasp → nk_activation (косвенно через age-decay) → senescent ↓ → sasp ↓
  Характер: ослабевает с возрастом из-за nk_age_decay

Петля F: Quiescence (Round 7 L2)
  damage → quiescence_factor ↓ → division_rate ↓ → damage_rate ↓
  Характер: отрицательная обратная связь, замедляет накопление при высоком damage

Петля G: Теломерное торможение
  division_rate → telomere_shortening → (hayflick → apoptosis limit)
  Характер: медленная (100+ лет), предотвращает бесконечное деление
```

### 2.3 Центральные узлы (hubs) — влияют на большинство переменных

```
1. DAMAGE            ← alpha, division_rate, ros_factor → senescent, frailty, epi_age
2. SASP              ← senescent, cgas, nfkb → ros, fibrosis, frailty, division_rate
3. ROS               ← mtdna, sasp → damage_factor, mtdna (петля A)
4. DIVISION_RATE     ← nu, sasp_factor, quiescence, regen → damage, telomere
5. PROTECTION Π(t)   ← pi_0, tau, age → damage_rate (самый важный тормоз)
```

### 2.4 Предложения по интеграции dead parameters

| Параметр | Формула интеграции | Эффект |
|----------|-------------------|--------|
| `mtor_activity` | `division_rate *= (1 + mtor_activity * (1-age/100))`; при >0.7 → ускоренное деление → больше damage | mTOR как проагeing узел |
| `yap_taz_sensitivity` | `fibrosis_rate *= (1 + yap_taz_sensitivity * fibrosis_level)` | Положительная обратная связь фиброза через механотрансдукцию |
| `meiotic_reset` | `epigenetic_age *= (1 - meiotic_reset * 0.1)` применять к зародышевой линии | Объясняет почему дети рождаются "молодыми" |
| `circadian_amplitude` | Уже добавлен placeholder в Round 7: `_circadian_repair_factor` | Требует субгодового dt для полной реализации |

---

## Часть 3 — ASCII карта системы (сводная)

```
                     ┌─────────────────────────────────────────────────┐
  AGE ──────────────▶│              PROTECTION Π(t)                    │
                     │   pi_0 × exp(-age/tau) + pi_baseline            │
                     └──────────────────────┬──────────────────────────┘
                                           │ (1-Π) → больше damage
                                           ▼
 NU×BETA×(1-TOL) ──▶ DIVISION_RATE ──────▶ DAMAGE_RATE ──▶ DAMAGE ──┐
   sasp_factor ↑      quiescence ↓(L2)      alpha          ↓   ↓    │
   regen_factor ↓(L3)  ▼                    ros_factor     ↓   ↓    │
                    TELOMERE ↓                           EPIAGE  │   │
                                                         SENESC.─┘   │
                                                            │         │
            ┌───────────────────────────────────────────────┘         │
            ▼                                                          │
        SENESCENT ──────────▶ DAMPS ──▶ cGAS-STING ──▶ NF-κB         │
            │                  ▲                          │            │
            │                  └──── MTDNA ◀── ROS ◀─────┤            │
            │                                     ▲       ▼            │
            │                                 SASP│prod  SASP ◀───────┤
            │                                             │   (autofb) │
            ◀──── NK clearance (↓ with age) ◀────────────┘            │
            ▼                                             ▼            │
       (reduced)                                      FIBROSIS         │
                                                      regen_factor ──▶─┘
                                           FRAILTY = 0.4×dmg + 0.3×sasp
                                                    + 0.2×(1-pool) + 0.1×(1-tel)
```

---

## Часть 4 — Статус Round 7

| Fix | Файл | Статус |
|-----|------|--------|
| B2: NF-κB *0.9 → убран | `inflammaging/system.rs` | ✅ |
| B3: CHIP VAF recalibrated | `validation/biomarkers.rs` | ✅ |
| B4: nk_age_decay 0.005→0.010 | `inflammaging/params.rs` | ✅ |
| M1: Telomere update | `basic_simulation.rs` | ✅ |
| M2: Epigenetic age update | `basic_simulation.rs` | ✅ |
| M3: circadian placeholder | `basic_simulation.rs` | ✅ |
| L2: Quiescence at high damage | `basic_simulation.rs` | ✅ |
| L3: Fibrosis→regen_factor | `basic_simulation.rs` | ✅ |
| L1: CHIP→SASP coupling | `chip_drift.rs + main loop` | ⏳ TODO |
| C1: mito_shield exponential | `mitochondrial/system.rs` | ⏳ TODO |
