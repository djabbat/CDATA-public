#!/usr/bin/env python3
"""
CDATA v3.0 — Streamlit GUI
Симулятор старения на основе CDATA (Centriolar Damage Accumulation Theory of Aging)
Запуск: streamlit run cdata_gui.py
"""

import streamlit as st
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
from dataclasses import dataclass

# ─── Page config ──────────────────────────────────────────────────────────────
st.set_page_config(
    page_title="CDATA v3.0 Simulator",
    page_icon="🧬",
    layout="wide",
    initial_sidebar_state="expanded",
)

# ─── Parameters ───────────────────────────────────────────────────────────────
@dataclass
class Params:
    # Core
    alpha: float = 0.0082
    hayflick_limit: float = 50.0
    # Youth protection
    pi_0: float = 0.87
    tau_protection: float = 24.3
    pi_baseline: float = 0.10
    # Tissue (HSC default)
    nu: float = 12.0
    beta: float = 1.0
    tolerance: float = 0.3
    regen_potential: float = 0.8
    # SASP
    stim_threshold: float = 0.3
    inhib_threshold: float = 0.8
    max_stimulation: float = 1.5
    # Mitochondrial
    ros_steepness: float = 10.0
    mitophagy_threshold: float = 0.35
    # Inflammaging
    damps_rate: float = 0.05
    cgas_sensitivity: float = 0.8
    sasp_decay: float = 0.1
    nk_age_decay: float = 0.010
    fibrosis_rate: float = 0.02
    # Interventions
    caloric_restriction: float = 0.0  # 0..1 (0=none, 1=full CR)
    senolytics: float = 0.0           # 0..1 (clearance boost)
    antioxidants: float = 0.0         # 0..1 (ROS reduction)
    mtor_inhibition: float = 0.0      # 0..1 (rapamycin-like)
    telomerase: float = 0.0           # 0..1 (telomere extension)
    nk_boost: float = 0.0             # 0..1 (immune therapy)
    stem_cell_therapy: float = 0.0    # 0..1 (pool replenishment)
    epigenetic_reprog: float = 0.0    # 0..1 (yamanaka-like reset)


def youth_protection(age, p):
    return p.pi_0 * np.exp(-age / p.tau_protection) + p.pi_baseline


def sasp_hormetic(sasp, p):
    if sasp < p.stim_threshold:
        return 1.0 + (p.max_stimulation - 1.0) / p.stim_threshold * sasp
    elif sasp <= p.inhib_threshold:
        r = p.inhib_threshold - p.stim_threshold
        t = (sasp - p.stim_threshold) / r
        return p.max_stimulation - (p.max_stimulation - 1.0) * t
    else:
        return 1.0 / (1.0 + 3.0 * (sasp - p.inhib_threshold))


def sigmoid_ros(x, steepness, threshold):
    return 1.0 / (1.0 + np.exp(-steepness * (x - threshold)))


def run_simulation(p: Params, years: int = 100):
    dt = 1.0
    TELOMERE_LOSS = 0.012

    # State variables
    damage = 0.0
    pool = 1.0
    mtdna = 0.0
    ros = p.pi_baseline  # start at young ROS
    damps = 0.0
    cgas = 0.0
    nfkb = 0.05
    sasp = 0.0
    senescent = 0.0
    nk = 1.0
    fibrosis = 0.0
    telomere = 1.0
    epigenetic = 0.0
    frailty = 0.0

    history = {k: [] for k in [
        "age", "damage", "pool", "ros", "sasp", "senescent",
        "frailty", "telomere", "epigenetic", "fibrosis", "nk", "mtdna"
    ]}

    for year in range(years + 1):
        age = float(year)

        # Interventions
        effective_nu = p.nu * (1.0 - p.caloric_restriction * 0.3)
        effective_nu *= (1.0 - p.mtor_inhibition * 0.2)

        protection = youth_protection(age, p)
        age_factor = max(1.0 - age / 120.0, 0.5)
        sasp_factor = sasp_hormetic(sasp, p)
        quiescence_factor = max(1.0 - damage * 0.5, 0.2)
        regen_factor = max(1.0 - fibrosis * 0.4, 0.3)

        division_rate = effective_nu * age_factor * sasp_factor * p.regen_potential * quiescence_factor * regen_factor

        # Stem cell therapy: replenish pool
        if p.stem_cell_therapy > 0:
            pool = min(1.0, pool + p.stem_cell_therapy * 0.05)

        ros_damage_factor = 1.0 + ros * 0.5 * (1.0 - p.antioxidants)

        damage_rate = p.alpha * division_rate * (1.0 - protection) * p.beta * (1.0 - p.tolerance) * ros_damage_factor
        damage = min(damage + damage_rate * dt, 1.0)
        pool = max(1.0 - damage * 0.8, 0.0)

        # Telomere
        if p.telomerase > 0:
            tel_extension = p.telomerase * 0.005 * dt
        else:
            tel_extension = 0.0
        telomere = max(telomere - TELOMERE_LOSS * division_rate * dt + tel_extension, 0.0)

        # Epigenetic
        epi_drift = (age - epigenetic) * 0.1 * dt
        epi_stress = 0.15 * (damage + sasp * 0.5) * dt
        epi_reset = p.epigenetic_reprog * 0.1 * dt if p.epigenetic_reprog > 0 else 0.0
        epigenetic = max(0.0, min(epigenetic + epi_drift + epi_stress - epi_reset, age + 30.0))

        # Mitochondrial
        mtdna = min(mtdna + 0.001 * ros * ros * dt, 1.0)
        ros_input = mtdna + sasp * 0.3
        ros = sigmoid_ros(ros_input, p.ros_steepness, p.mitophagy_threshold)
        ros *= (1.0 - p.antioxidants * 0.5)

        # Inflammaging
        damps_prod = p.damps_rate * (senescent + damage * 0.5)
        damps = max(0.0, min(damps + damps_prod * dt - 0.1 * damps * dt, 1.0))

        cgas = min(damps * p.cgas_sensitivity + mtdna * 0.05, 1.0)
        nfkb = min(0.05 + cgas * 0.6 + sasp * 0.3 + damps * 0.1, 0.95)

        sasp_prod = cgas * nfkb * senescent
        sasp = max(0.0, min(sasp + sasp_prod * dt - p.sasp_decay * sasp * dt, 1.0))

        # Senolytics
        senolytic_clearance = p.senolytics * 0.2 * senescent * dt

        nk_base = max(1.0 - age * p.nk_age_decay, 0.1)
        nk = max(nk_base * (1.0 - sasp * 0.3) + p.nk_boost * 0.1, 0.05)
        nk_eliminated = nk * 0.1 * senescent * dt + senolytic_clearance

        new_sen = damage * 0.05 * dt
        senescent = max(0.0, min(senescent + new_sen - nk_eliminated, 1.0))

        fibrosis = min(fibrosis + p.fibrosis_rate * sasp * dt, 1.0)

        frailty = min(damage * 0.4 + sasp * 0.3 + (1.0 - pool) * 0.2 + (1.0 - telomere) * 0.1, 1.0)

        for k, v in [("age", age), ("damage", damage), ("pool", pool), ("ros", ros),
                     ("sasp", sasp), ("senescent", senescent), ("frailty", frailty),
                     ("telomere", telomere), ("epigenetic", epigenetic),
                     ("fibrosis", fibrosis), ("nk", nk), ("mtdna", mtdna)]:
            history[k].append(v)

    return {k: np.array(v) for k, v in history.items()}


# ─── Presets ──────────────────────────────────────────────────────────────────
PRESETS = {
    "Normal (HSC)": {},
    "Progeria": {"alpha": 0.025, "tau_protection": 8.0, "pi_0": 0.50},
    "Longevity": {"alpha": 0.005, "tau_protection": 35.0, "pi_0": 0.92, "nk_age_decay": 0.006},
    "ISC (Кишечные)": {"nu": 70.0, "beta": 0.3, "tolerance": 0.8, "regen_potential": 0.95},
    "Neural": {"nu": 2.0, "beta": 1.5, "tolerance": 0.2, "regen_potential": 0.2},
    "Muscle": {"nu": 4.0, "beta": 1.2, "tolerance": 0.5, "regen_potential": 0.5},
}

INTERVENTIONS = {
    "Caloric Restriction (CR)": "caloric_restriction",
    "Senolytics (ABT-263)": "senolytics",
    "Antioxidants (NAC)": "antioxidants",
    "mTOR Inhibition (Rapamycin)": "mtor_inhibition",
    "Telomerase Activation": "telomerase",
    "NK Boost (Immunotherapy)": "nk_boost",
    "Stem Cell Therapy": "stem_cell_therapy",
    "Epigenetic Reprogramming (Yamanaka)": "epigenetic_reprog",
}

# ─── UI ───────────────────────────────────────────────────────────────────────
st.title("🧬 CDATA v3.0 — Centriolar Damage Accumulation Theory of Aging")
st.markdown("**EIC Pathfinder demo** · Round 7 fixes · [PMID 36583780](https://pubmed.ncbi.nlm.nih.gov/36583780/)")

col_side, col_main = st.columns([1, 3])

with col_side:
    st.subheader("⚙️ Параметры")

    preset_name = st.selectbox("Пресет", list(PRESETS.keys()))
    preset_vals = PRESETS[preset_name]

    p = Params()
    for k, v in preset_vals.items():
        setattr(p, k, v)

    with st.expander("🔬 Биологические параметры", expanded=False):
        p.alpha = st.slider("α (повреждение/деление)", 0.001, 0.05, p.alpha, 0.001, format="%.4f")
        p.pi_0 = st.slider("π₀ (защита молодости)", 0.3, 1.0, p.pi_0, 0.01)
        p.tau_protection = st.slider("τ (годы спада защиты)", 5.0, 50.0, p.tau_protection, 0.5)
        p.nu = st.slider("ν (делений/год)", 1.0, 100.0, p.nu, 0.5)
        p.tolerance = st.slider("τ ткани (толерантность)", 0.0, 0.95, p.tolerance, 0.05)
        p.nk_age_decay = st.slider("NK decay/год", 0.001, 0.02, p.nk_age_decay, 0.001, format="%.3f")

    st.subheader("💊 Интервенции")
    for label, field in INTERVENTIONS.items():
        val = st.slider(label, 0.0, 1.0, 0.0, 0.05, key=field)
        setattr(p, field, val)

    years = st.slider("Длительность (лет)", 50, 120, 100, 5)

    compare = st.checkbox("Сравнить с Control (без интервенций)")

with col_main:
    # Run simulation
    sim = run_simulation(p, years)
    ages = sim["age"]

    if compare:
        p_ctrl = Params()
        for k, v in preset_vals.items():
            setattr(p_ctrl, k, v)
        ctrl = run_simulation(p_ctrl, years)
    else:
        ctrl = None

    # ── Plots ────────────────────────────────────────────────────────────
    fig = plt.figure(figsize=(14, 10))
    fig.patch.set_facecolor('#0e1117')
    gs = gridspec.GridSpec(3, 3, figure=fig, hspace=0.45, wspace=0.35)

    PLOTS = [
        ("Centriole Damage", "damage", "#e74c3c"),
        ("Stem Cell Pool", "pool", "#2ecc71"),
        ("ROS Level", "ros", "#f39c12"),
        ("SASP Level", "sasp", "#e67e22"),
        ("Senescent Fraction", "senescent", "#9b59b6"),
        ("NK Efficiency", "nk", "#1abc9c"),
        ("Telomere Length", "telomere", "#3498db"),
        ("Epigenetic Age", "epigenetic", "#e91e63"),
        ("Frailty Index", "frailty", "#c0392b"),
    ]

    for i, (title, key, color) in enumerate(PLOTS):
        ax = fig.add_subplot(gs[i // 3, i % 3])
        ax.set_facecolor('#1a1a2e')
        ax.plot(ages, sim[key], color=color, linewidth=2, label="Intervention")
        if ctrl is not None:
            ax.plot(ages, ctrl[key], color='#888', linewidth=1.5, linestyle='--', label="Control")
        ax.set_title(title, color='white', fontsize=9, pad=4)
        ax.tick_params(colors='#aaa', labelsize=7)
        for spine in ax.spines.values():
            spine.set_edgecolor('#333')
        ax.set_xlabel("Age (years)", color='#aaa', fontsize=7)
        if i == 0 and ctrl is not None:
            ax.legend(fontsize=6, loc='upper left', facecolor='#222', labelcolor='white')

    st.pyplot(fig)
    plt.close(fig)

    # ── Summary metrics ──────────────────────────────────────────────────
    st.subheader("📊 Итоговые показатели")
    m1, m2, m3, m4 = st.columns(4)

    frailty_80 = sim["frailty"][min(80, years)]
    damage_100 = sim["damage"][min(100, years)]
    tel_100 = sim["telomere"][min(100, years)]
    epi_100 = sim["epigenetic"][min(100, years)]

    m1.metric("Frailty @ 80 лет", f"{frailty_80:.3f}",
              delta=f"{frailty_80 - ctrl['frailty'][80]:.3f}" if ctrl else None,
              delta_color="inverse")
    m2.metric("Damage @ 100 лет", f"{damage_100:.3f}",
              delta=f"{damage_100 - ctrl['damage'][min(100,years)]:.3f}" if ctrl else None,
              delta_color="inverse")
    m3.metric("Telomere @ 100 лет", f"{tel_100:.3f}",
              delta=f"{tel_100 - ctrl['telomere'][min(100,years)]:.3f}" if ctrl else None)
    m4.metric("Epigenetic age @ 100", f"{epi_100:.1f}",
              delta=f"{epi_100 - ctrl['epigenetic'][min(100,years)]:.1f}" if ctrl else None,
              delta_color="inverse")

    st.caption("CDATA v3.0 · Tkemaladze J. (2023) PMID 36583780 · "
               "Zenodo DOI: 10.5281/zenodo.19174506 · EIC Pathfinder 2026")
