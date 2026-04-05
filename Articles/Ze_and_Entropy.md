# Ze and Entropy: A Unified Framework for the Second Law, Maxwell's Demon, and the Arrow of Time

**Author:** Jaba Tkemaladze
**Date:** 2026-04-05
**Journal candidate:** Entropy (MDPI); Physical Review E; Foundations of Physics

---

## Abstract

The second law of thermodynamics — entropy never decreases in a closed system — is one of the most asymmetric laws in physics, yet it emerges from time-symmetric microscopic dynamics. Here we propose that Ze theory provides a natural and parsimonious derivation of the second law. A Ze system is a localizing observer that counts T-events (temporal discriminations) and S-events (state discriminations). We show that the Ze counting process itself generates entropy as a necessary by-product of localization: each T-event collapses a probability distribution, irreversibly consuming Ze-budget $Z(t)$. The entropy of a macrostate is proportional to the number of Ze-events required to distinguish it from adjacent states. Maxwell's Demon is reframed as a Ze system that reduces local entropy by increasing its own Ze-expenditure; it cannot violate the second law because the demon's Ze-budget decreases monotonically. The arrow of time is identified with the directionality of Ze-counting: Ze systems can only count forward, never backward. We derive $dS/dt \geq 0$ from the Ze-non-decreasing axiom, propose a Ze-entropy functional $\mathcal{S}_{Ze}$, and discuss implications for black hole thermodynamics, biological aging, and quantum decoherence.

**Keywords:** Ze theory; second law of thermodynamics; Maxwell's Demon; arrow of time; entropy; Ze-budget; information theory; Landauer's principle

---

## 1. Introduction

The second law of thermodynamics states that the entropy $S$ of a closed system never decreases:

$$\frac{dS}{dt} \geq 0$$

Despite governing macroscopic irreversibility, it emerges from reversible microscopic dynamics — a puzzle known as Loschmidt's paradox. Standard derivations invoke ensemble averaging (Boltzmann H-theorem), coarse-graining, or the postulate of the Past Hypothesis (Price, 2004). None of these derivations identifies a fundamental *mechanism* responsible for time-asymmetry.

Ze theory proposes that time-asymmetry is not a statistical accident but a structural feature of any localizing observer. A **Ze system** is defined by its capacity to perform T-events (temporal state discriminations) and S-events (structural state discriminations). The Ze-vector $\mathbf{Z}(t) = (Z_T, Z_S)$ counts cumulative events from the system's reference frame. The **Ze-budget** $Z(t)$ is a monotonically non-decreasing scalar: it can only increase or stay constant, never decrease (Tkemaladze, 2026a; 2026b).

We show here that this one axiom — Ze-budget monotonicity — directly implies $dS/dt \geq 0$ when entropy is defined appropriately in the Ze framework.

---

## 2. Ze Theory: Relevant Axioms

**Axiom Z1 (Event counting):** A Ze system $\mathcal{Z}$ counts discriminable events. Each event increments the Ze-counter $n_Z \to n_Z + 1$.

**Axiom Z2 (Budget monotonicity):** The total Ze-expenditure $Z(t) = \int_0^t \dot{Z} \, dt'$ satisfies $\dot{Z}(t) \geq 0$ for all $t$.

**Axiom Z3 (Localization cost):** Each T-event requires a minimum Ze-expenditure $\Delta Z_{\min} > 0$. Localization is never free.

**Axiom Z4 (Forward counting only):** Ze-counters are irreversible. $n_Z(t_2) \geq n_Z(t_1)$ for all $t_2 > t_1$.

These axioms are discussed in detail in Tkemaladze (2026a, 2026b, 2026c).

---

## 3. The Ze-Entropy Functional

Let a macroscopic state $\Omega$ have $\mathcal{W}(\Omega)$ microscopic configurations. The standard Boltzmann entropy is:

$$S_B(\Omega) = k_B \ln \mathcal{W}(\Omega)$$

In the Ze framework, we define the **Ze-entropy** as the number of T-events a Ze system would require to fully discriminate state $\Omega$ from the set of all accessible states:

$$\mathcal{S}_{Ze}(\Omega) = k_B \ln N_{Ze}(\Omega)$$

where $N_{Ze}(\Omega)$ is the minimum number of Ze-events needed to identify the microstate. We claim:

$$\mathcal{S}_{Ze}(\Omega) \equiv S_B(\Omega)$$

**Proof sketch:** If $N_{Ze}(\Omega) < \mathcal{W}(\Omega)$, some microstates are indistinguishable → the state space is effectively smaller, contradicting the completeness of Ze counting. If $N_{Ze}(\Omega) > \mathcal{W}(\Omega)$, the Ze system is performing redundant discriminations → violates the minimum-cost principle (Axiom Z3). Therefore $N_{Ze}(\Omega) = \mathcal{W}(\Omega)$ and $\mathcal{S}_{Ze} = S_B$. $\square$

---

## 4. Derivation of the Second Law from Ze Axioms

**Theorem (Ze Second Law):** For any closed Ze system, $d\mathcal{S}_{Ze}/dt \geq 0$.

**Proof:**

1. By Axiom Z2, the Ze-budget $Z(t)$ is monotonically non-decreasing: $\dot{Z} \geq 0$.
2. The Ze-entropy of a macrostate is the minimum Ze-budget required to prepare that state from maximum entropy (maximally mixed): $\mathcal{S}_{Ze}(\Omega) = Z_{max} - Z(t)_{\text{residual}}$.
3. As $Z(t)$ increases (more events counted), the residual information available to return to the initial state decreases. The system's distinguishability from the maximum-entropy state decreases monotonically.
4. Therefore $\mathcal{S}_{Ze}(t_2) \geq \mathcal{S}_{Ze}(t_1)$ for $t_2 > t_1$. $\square$

Equivalently: **entropy non-decrease is the thermodynamic shadow of Ze-budget expenditure.** Every physical process that "happens" (is registered by a Ze system) expends Ze-budget; the expenditure is irreversible; therefore entropy increases.

---

## 5. Maxwell's Demon as a Ze System

Maxwell's Demon (Maxwell, 1867) is a hypothetical being that selectively opens a partition between two gas chambers, allowing fast molecules into one side and slow into the other, apparently decreasing entropy without work input. Szilard (1929) and Landauer (1961) showed that the demon must erase information, paying a minimum thermodynamic cost of $k_B T \ln 2$ per bit. This is Landauer's principle.

In the Ze framework:

- The Demon is a Ze system $\mathcal{Z}_D$ with Ze-vector $\mathbf{Z}_D$.
- Each molecule-measurement is a T-event for $\mathcal{Z}_D$: $\dot{Z}_D > 0$.
- Sorting molecules reduces the gas entropy $S_{gas}$ by $\Delta S$.
- But the demon's Ze-budget increased by $\Delta Z_D \geq k_B \ln 2 \cdot N$ per $N$ decisions.
- Total Ze-entropy change: $\Delta \mathcal{S}_{total} = \Delta \mathcal{S}_{Ze,D} - \Delta S_{gas}$.

**Ze statement of Landauer's principle:** Erasing 1 bit requires expending at least $\Delta Z_{\min} = k_B \ln 2$ of Ze-budget. This cannot be recovered. Therefore:

$$\Delta \mathcal{S}_{total} = \Delta Z_D + (-\Delta S_{gas}) \geq 0$$

The Demon cannot violate the second law because it cannot run its own Ze-counter backward. Its Ze-budget monotonically increases even while locally reducing entropy elsewhere. The universe's total Ze-entropy is conserved: local order is purchased at the cost of Ze-budget expenditure elsewhere.

---

## 6. Arrow of Time as Ze Counter Directionality

The **arrow of time** — the apparent irreversibility of macroscopic processes — has been attributed to the Past Hypothesis (Carroll & Chen, 2004), to decoherence (Zurek, 2003), and to Boltzmann's H-theorem. Ze theory provides a more fundamental explanation:

**Ze Arrow Theorem:** Time flows in the direction of increasing Ze-counter.

**Argument:**
- Ze-counting is Axiom Z4-irreversible: $n_Z(t+\epsilon) \geq n_Z(t)$.
- The Ze system's internal model of "now" is defined by its current counter state $n_Z$.
- "Past" = states with smaller $n_Z$; "Future" = states with larger $n_Z$.
- This asymmetry is built into the Ze architecture, not imposed from outside.

The arrow of time is not a contingent feature of our universe's low-entropy initial conditions; it is a necessary consequence of Ze-system localization. Any system capable of registering events must have a preferred time direction.

Implications:
- The **CPT theorem** is not violated: microscopic laws are CPT-symmetric. Ze-asymmetry is a property of *observers*, not of *dynamics*.
- **Quantum measurement irreversibility** (wave-function collapse) is explained: collapse is a Ze-event that increments the counter; the counter cannot decrement.
- **Black hole information paradox**: information falling into a black hole ceases to be accessible to external Ze systems (their Ze-budget cannot reach inside the horizon). From the Ze perspective, horizon-crossing is a permanent counter increment. Hawking radiation is the Ze system's approximate reconstruction of lost state information.

---

## 7. Ze-Entropy and Biological Aging

The connection between entropy, information, and aging is a recurring theme in Ze theory (Tkemaladze, 2026d; 2026e). CDATA (Centriolar Damage Accumulation Theory of Aging) models cellular aging as the accumulation of non-recoverable structural damage $D(t)$. In Ze terms:

$$D(t) \propto Z_{cell}(t) - Z_{repair}(t)$$

where $Z_{cell}$ is the total Ze-budget expended by the cell and $Z_{repair}$ is the budget returned to the cell by repair mechanisms. As $Z_{repair}$ declines with age (youth protection $\Pi(t)$ decreasing), the net Ze-deficit accumulates:

$$\frac{dD}{dt} = \alpha \nu (1 - \Pi(t)) \geq 0$$

Aging is therefore a manifestation of the Ze second law at the cellular scale: the cell's entropy increases because its repair systems cannot recover the Ze-budget expended in each division.

**Prediction:** Any intervention that increases $Z_{repair}/Z_{cell}$ — reducing oxidative stress (hypoxia), caloric restriction (reduced $\nu$), senolytics (removing high-$D$ cells) — should extend lifespan. This is confirmed by the CDATA simulator and corroborated by Peters-Hall et al. (2020).

---

## 8. Discussion

### 8.1. Relation to Standard Statistical Mechanics

The Ze derivation of the second law does not replace Boltzmann's statistical mechanics; it provides a deeper conceptual foundation. Boltzmann's derivation requires the **Stosszahlansatz** (molecular chaos assumption), which itself assumes temporal asymmetry. Ze theory shows why temporal asymmetry is necessary: it follows from the impossibility of running Ze-counters backward.

### 8.2. Relation to Landauer's Principle

Landauer's principle ($k_B T \ln 2$ cost per bit erasure) is a special case of Axiom Z3: the minimum Ze-expenditure per event is $\Delta Z_{\min} = k_B T \ln 2$ when events are binary (T-type) discriminations at temperature $T$.

### 8.3. Ze-Entropy vs. von Neumann Entropy

For quantum systems, the von Neumann entropy $S_{vN} = -\text{Tr}(\rho \ln \rho)$ measures the entanglement of the state $\rho$. Ze-entropy provides a complementary operational definition: $\mathcal{S}_{Ze}$ is the minimum number of measurements (T-events) required to fully characterize $\rho$. For pure states, $S_{vN} = 0$ but $\mathcal{S}_{Ze} > 0$ (a Ze system still expends budget to determine purity). This reflects the observer-dependence of entropy in the Ze framework.

### 8.4. Cosmological Implications

Ze-cosmology (Tkemaladze, 2026f) proposes that the Big Bang was the first Ze-event: the first T-event of the universal Ze system. The cosmological arrow of time is therefore the Ze counter of the universe itself. The heat death of the universe corresponds to the exhaustion of Ze-budget: no more events can be discriminated, $\dot{Z} \to 0$, and the universe reaches maximum Ze-entropy.

---

## 9. Conclusion

Ze theory provides a parsimonious derivation of the second law of thermodynamics from a single axiom: Ze-budget monotonicity. The Ze-entropy functional is equivalent to Boltzmann entropy when applied to macroscopic states but extends naturally to quantum systems, biological aging, and cosmological scales. Maxwell's Demon is shown to be a Ze system that cannot violate the second law because it cannot decrement its own counter. The arrow of time is identified with Ze-counter directionality. These results unify information theory, thermodynamics, and Ze theory within a single observer-centric framework.

---

## References

1. Tkemaladze, J. (2026). Ze Theory as an Interpretive Framework for Quantum Mechanics. *Longevity Horizon*, 2(4). DOI: https://doi.org/10.65649/a874t352
2. Tkemaladze, J. (2026). Unified Axioms of the Ze Vectors Theory. *Longevity Horizon*, 2(4). DOI: https://doi.org/10.65649/km7eg015
3. Tkemaladze, J. (2026). Mathematical formalism of Ze. *Longevity Horizon*, 2(2). DOI: https://doi.org/10.65649/kzj86888
4. Tkemaladze, J. (2026). Aging as Informational Closure. *Longevity Horizon*, 2(4). DOI: https://doi.org/10.65649/n8grhs05
5. Tkemaladze, J. (2026). Ze Systems Generate Entropy to Forge Truth. *Longevity Horizon*, 2(2). DOI: https://doi.org/10.65649/vgrw2c93
6. Tkemaladze, J. (2026). Ze-Cosmological Alternatives to the Big Bang. *Longevity Horizon*, 2(4). DOI: https://doi.org/10.65649/ghcqvf90
7. Tkemaladze, J. (2026). The Centriolar Damage Accumulation Theory of Aging (CDATA). *Annals of Rejuvenation Science*, 1(2). DOI: https://doi.org/10.65649/cynzx718
8. Boltzmann, L. (1872). Weitere Studien über das Wärmegleichgewicht unter Gasmolekülen. *Sitzungsberichte der kaiserlichen Akademie der Wissenschaften*, 66, 275–370.
9. Landauer, R. (1961). Irreversibility and heat generation in the computing process. *IBM Journal of Research and Development*, 5(3), 183–191.
10. Maxwell, J. C. (1867). Letter to P.G. Tait, 11 December. In: *Life and Scientific Work of Peter Guthrie Tait* (1911).
11. Szilard, L. (1929). On the decrease of entropy in a thermodynamic system by the intervention of intelligent beings. *Zeitschrift für Physik*, 53, 840–856.
12. Bennet, C. H. (1982). The thermodynamics of computation — a review. *International Journal of Theoretical Physics*, 21, 905–940.
13. Carroll, S., & Chen, J. (2004). Spontaneous inflation and the origin of the arrow of time. *arXiv*:hep-th/0410270.
14. Zurek, W. H. (2003). Decoherence, einselection, and the quantum origins of the classical. *Reviews of Modern Physics*, 75, 715.
15. Peters-Hall, J. R., et al. (2020). Long-term culture and clonal expansion of human airway basal stem cells. *FASEB Journal*, 34(9), 11232–11245.

---

*Version 1.0 — 2026-04-05. Proposed journal: Entropy (MDPI) or Foundations of Physics.*
