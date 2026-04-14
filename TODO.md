# TODO — CDATA
Последнее обновление: 2026-04-15

---

## 🔴 P0 — КРИТИЧНО (до 25 апреля 2026)

- [ ] Получить письмо поддержки от **Prof. Hartmut Geiger** (hartmut.geiger@uni-ulm.de) — deadline 20.04 (ответ на письмо от 14.04.2026)
  - Plan B: Dr. Sten Eirik Jacobsen (Karolinska) если Geiger не ответит до 20.04
- [ ] Встреча с **Aubrey de Grey** — 18 апреля 2026 → получить letter of support (1–2 абзаца)
  - Шпаргалка: `~/Documents/CDATA/AUBREY_MEETING_2026-04-18/SHPARGALKA_AUBREY.md`
- [ ] LOI v21 → конвертировать в DOCX через `md_to_docx.py`
  - Исходник: `~/Documents/CDATA/AUBREY_MEETING_2026-04-18/LOI_Impetus.md`
- [ ] После получения письма от Geiger/Jacobsen → финальный peer review (deepseek-reasoner)
- [ ] Добавить Phase 0 Arm RELAPSE в LOI v21 (~$8K, сингенная co-culture, relapse prediction)
- [ ] Добавить Lavasani/Kovina/Leins/CD150low как supporting evidence в LOI v21 §Background

---

## 🟡 P1 — ВАЖНО (май 2026)

- [ ] EIC Pathfinder CommonHealth (дедлайн 12 мая 2026) — CDATA как experimental subtrack
- [ ] Обновить LOI после встречи с Aubrey — добавить его письмо в §Letters of Support
- [ ] Провести 20-минутный звонок с Liz Parrish (подтвердила co-PI 14.04.2026)
- [ ] Cell-DT v4.0: реализовать D(t)→ep_age интеграцию (ABL-2 парадокс):
  ```
  ep_age(t) = ep_rate_base × t + k_ep × ∫D(τ)dτ
  ```
  Устраняет dominance epigenetic_rate в Sobol, делает alpha явным доминантом
- [ ] Добавить budget contingency reserve 5–10% в Phase 1 бюджет LOI

---

## 🟢 P2 — Планово (Q2 2026)

- [ ] **Phase 0** (после финансирования, ~$12K):
  - [ ] Уровень 1: GT335 + Ninein co-stain → polyGlu asymmetry index (молодые vs. старые LSK)
  - [ ] Уровень 2: ARL13B + Ninein → частота первичных ресничек в молодых vs. старых LSK
  - [ ] Уровень 3: Ki67/EdU → division rate (Аксиома 3); co-culture Arm RELAPSE (P11)
- [ ] **Aging Cell submission** preparation:
  - [ ] Расширить калибровочный датасет (28 → 80+ точек, UK Biobank)
  - [ ] LOO-CV mean=-0.093 → исправить через ROS-уравнение и расширение данных
  - [ ] Измерить meiotic_reset: STED GT335 на ооцитах до/после оплодотворения
  - [ ] Cell-DT v4.0 Sobol с полной Rust-ODE (не аналитическим приближением)
- [ ] **KNOWLEDGE.md** — обновлять при каждой новой релевантной публикации (PubMed alert: TTLL6, HSC aging, centriole)
- [ ] Добавить P11 (relapse prediction) в официальный список предсказаний CONCEPT.md §P7–P10 → расширить до P11

---

## ✅ ЗАВЕРШЕНО

- [x] Три аксиомы зафиксированы в CONCEPT.md (locked, 2026-04-14)
- [x] LOI v21 создан (MAJOR REVISION 25%, 2026-04-14)
- [x] Peer review v19 (DO NOT FUND), v20 (DO NOT FUND), v21 (MAJOR REVISION 25%)
- [x] LOI v21: Phase 0 → первичные LSK (не клеточные линии)
- [x] LOI v21: Arm E (aged LSK + CCP1-OE) — тест Аксиомы 3
- [x] LOI v21: BHCA исправлен 17/27 (Prop 1) + ~10/27 (Prop 2)
- [x] R²=0.84 изъято из всей документации (синтетика, 2026-04-13)
- [x] Sobol N=16384, bootstrap CI — S4 ЗАКРЫТ (2026-04-13)
- [x] Liz Parrish подтверждена как Industry & Strategic Co-PI (2026-04-14)
- [x] Письмо Geiger отправлено (2026-04-14)
- [x] Multi-Organism Supporting Evidence добавлены в CONCEPT.md v5.0 (2026-04-15)
- [x] Relapse Prediction P11 формализовано (2026-04-15)
- [x] Шпаргалка для встречи с Aubrey de Grey создана (2026-04-14)
- [x] Все ядерные .md файлы созданы (2026-04-15)
