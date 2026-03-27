# CDATA v3.0 — Карта взаимовлияния параметров
_Дата: 2026-03-27 | Модель: deepseek-reasoner | Round 7_

---

## Анализ параметров симулятора CDATA v3.0

### 1. Матрица взаимовлияния параметров (ключевые связи)

```
| ПАРАМЕТР              | ВЛИЯЕТ НА                            | ЗНАК  | СИЛА  | МЕХАНИЗМ                             |
|-----------------------|--------------------------------------|-------|-------|--------------------------------------|
| age                   | protection (Π)                      | –     | сильн | exp(-age/tau)                        |
| pi_0, pi_baseline     | protection                          | +     | средн | прямое слагаемое                     |
| tau_protection        | protection                          | –     | средн | скорость спада                      |
| protection            | damage_rate                         | –     | сильн | (1-protection)                       |
| nu (тканевый)         | division_rate                       | +     | сильн | линейный множитель                  |
| division_rate         | damage_rate                         | +     | сильн | прямое влияние                       |
| damage_rate           | damage, senescent                   | +     | сильн | интеграция по времени                |
| damage                | mtdna (косвенно), frailty           | +     | сильн | через ROS и прямо в frailty          |
| mtdna                 | ros                                 | +     | сильн | квадратичный вклад                  |
| ros_steepness         | ros                                 | –     | средн | крутизна сигмоиды                    |
| mitophagy_threshold   | ros                                 | –     | средн | порог активации                     |
| ros                   | damage_rate (ros_damage_factor)     | +     | сильн | множитель повреждения               |
| sasp                  | ros, nfkb, frailty                  | +     | средн | прямое слагаемое                    |
| senescent             | sasp, damps                         | +     | средн | источник SASP                        |
| nk_age_decay          | nk (неявно) → senescent             | –     | слабо | иммунное очищение                   |
| fibrosis_rate         | fibrosis                            | +     | слабо | линейный коэффициент                |
| cgas_sensitivity      | cgas                                | +     | средн | множитель для damps                 |
| damps_rate            | damps                               | +     | средн | скорость накопления                 |
| sasp_decay            | sasp                                | –     | средн | экспоненциальный распад             |
| telomere              | division_rate (через age_factor?)   | –     | средн | лимит делений                       |
| hayflick_limit        | telomere (инициализация)            | –     | средн | начальная длина теломер             |
| stim_threshold        | sasp_factor (гормезис)              | +/-   | слабо | порог стимуляции                    |
| max_stimulation       | sasp_factor                         | +     | слабо | амплитуда гормезиса                 |
| dnmt3a_fitness        | эпиг. возраст (косвенно)            | –     | слабо | влияние на эпиг. стабильность       |
| tet2_fitness          | эпиг. возраст (косвенно)            | –     | слабо | аналогично DNMT3A                   |
| circadian_amplitude   | не используется в уравнениях        | –     | –     | dead parameter                       |
| mtor_activity         | не используется в уравнениях        | –     | –     | dead parameter                       |
| meiotic_reset         | не используется в уравнениях        | –     | –     | dead parameter                       |
| yap_taz_sensitivity   | не используется в уравнениях        | –     | –     | dead parameter                       |
| beta (тканевый)       | damage_rate                         | +     | сильн | линейный множитель                  |
| tau (тканевый)        | damage_rate                         | –     | средн | tolerance (1-tau)                    |
```

### 2. Петли обратной связи

```
ПОЛОЖИТЕЛЬНЫЕ ОБРАТНЫЕ СВЯЗИ (ускоряющие старение):
1. damage → mtdna → ros → damage_rate → damage
   Усиление: повреждения → митохондриальная дисфункция → ROS → ещё больше повреждений.

2. damage → senescent → sasp → nfkb → sasp → damage
   Усиление: повреждения → сенесцентные клетки → SASP → воспаление → больше SASP.

3. sasp → ros → damage → senescent → sasp
   Усиление: SASP → окислительный стресс → повреждения → сенесценция.

4. damps → cgas → nfkb → sasp → inflammation → damps
   Воспалительная петля: DAMPs → cGAS-STING → NF-κB → SASP → повреждение тканей → DAMPs.

ОТРИЦАТЕЛЬНЫЕ ОБРАТНЫЕ СВЯЗИ (стабилизирующие):
1. sasp → nk → senescent → sasp
   Иммунное очищение: SASP → активация NK-клеток → элиминация сенесцентных клеток → снижение SASP.

2. telomere shortening → division_rate → damage_rate → damage
   Защитная: укорочение теломер → снижение пролиферации → уменьшение повреждений.

3. protection → damage_rate → damage → (косвенно) senescence → inflammation
   Возрастной спад защиты: защита спадает с возрастом → рост повреждений → воспаление.
```

### 3. Центральные узлы системы

```
1. DAMAGE (повреждение)
   • Влияет на: senescent, frailty, epigenetic_age, mtdna (через ROS), damps.
   • Приёмник от: damage_rate, protection, ros.
   • Хаб для интеграции стрессовых воздействий.

2. SASP (сенесцентный секреторный фенотип)
   • Влияет на: ros, nfkb, fibrosis, frailty.
   • Приёмник от: senescent, cgas, nfkb.
   • Ключевой медиатор воспалительного старения.

3. ROS (реактивные формы кислорода)
   • Влияет на: damage_rate, mtdna (косвенно).
   • Приёмник от: mtdna, sasp.
   • Центр окислительного стресса.

4. PROTECTION (защита Π(t))
   • Влияет на: damage_rate.
   • Приёмник от: age, pi_0, pi_baseline, tau_protection.
   • Определяет возрастную уязвимость.

5. DIVISION_RATE (скорость деления)
   • Влияет на: damage_rate, telomere.
   • Приёмник от: nu, sasp_factor, возрастные факторы.
   • Связь между пролиферацией и накоплением повреждений.
```

### 4. Dead Parameters (не влияют ни на что в текущих уравнениях)

```
1. mtor_activity (активность mTOR) — объявлен, но не интегрирован.
2. circadian_amplitude (амплитуда циркадных ритмов) — нет связи.
3. meiotic_reset (мейотический сброс) — не используется.
4. yap_taz_sensitivity (чувствительность YAP/TAZ) — не используется.

Примечание: dnmt3a_fitness и tet2_fitness упомянуты для CHIP, но их влияние на эпигенетический возраст только предполагается (нет явных уравнений).
```

### 5. Предложения по интеграции неиспользуемых параметров

```
1. MTOR_ACTIVITY:
   • Биологическая роль: регуляция аутофагии, синтеза белка, метаболизма.
   • Внедрение в модель:
     a) Влиять на damage_rate: damage_rate *= (1 + mtor_activity * age_factor)
     b) Влиять на proteostasis: добавить уравнение protein_aggregates += (mtor_activity - basal_autophagy)*dt
     c) Влиять на регенерацию: division_rate *= (1 - mtor_activity * inhibition_factor)

2. YAP_TAZ_SENSITIVITY:
   • Биологическая роль: механотрансдукция, регуляция пролиферации стволовых клеток.
   • Внедрение в модель:
     a) Влиять на тканевой regeneration_potential: regen_potential *= (1 + yap_taz_sensitivity * mechanical_stress)
     b) Влиять на фиброз: fibrosis_rate *= (1 + yap_taz_sensitivity * stiffness_feedback)

3. MEIOTIC_RESET:
   • Биологическая роль: эпигенетическое омоложение в зародышевой линии.
   • Внедрение в модель:
     a) Влиять на эпигенетический возраст: epigenetic_age -= meiotic_reset * germline_factor * dt
     b) Влиять на защиту: protection += meiotic_reset * (1 - age/hayflick_limit)

4. CIRCADIAN_AMPLITUDE:
   • Биологическая роль: суточные колебания репарации, метаболизма.
   • Внедрение в модель:
     a) Модулировать protection: protection *= (1 + circadian_amplitude * sin(2π*t/24))
     b) Влиять на ROS: ros_baseline *= (1 - circadian_amplitude * circadian_phase)
```

### Общая карта взаимовлияний (ASCII-схема):

```
AGE
  │
  ▼
PROTECTION (Π) ───┐
  │               ▼
  │         DAMAGE_RATE ←─ ROS ←─ MTDNA ←─ DAMAGE
  ▼               │          ▲       │        │
DIVISION_RATE ────┘          │       │        │
  │                          │       │        ▼
  ▼                          │       │    SENESCENT
TELOMERE                     │       │        │
                             │       │        ▼
                       SASP ←─ NF-κB ←─ CGAS ←─ DAMPS
                         │               ▲        ▲
                         ▼               │        │
                      FRAILTY            │        │
                         │               │        │
                         ▼               │        │
                    FIBROSIS ←───────────┘        │
                                                   │
                                         NK_CELLS →─┘
```

**Выводы:**
1. Система центрирована на **повреждениях** и **воспалении** с доминированием положительных обратных связей.
2. Критические точки контроля: защита (Π), скорость деления, пороги ROS и SASP.
3. Неиспользуемые параметры требуют биологически обоснованной интеграции через метаболические, механические и эпигенетические каналы.
4. Модель чувствительна к начальным условиям защиты и тканеспецифичным коэффициентам (β, τ, ν).

Для дальнейшего анализа рекомендую:
- Количественную оценку силы связей через частичные производные.
- Анализ устойчивости системы при вариации ключевых параметров.
- Эксперименты in silico по модуляции центральных узлов (например, искусственное усиление защиты или подавление SASP).
