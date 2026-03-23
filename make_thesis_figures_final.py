#!/usr/bin/env python3
"""
make_thesis_figures_final.py — P64
Строит 4 фигуры для статьи CDATA из CSV-файлов.

Выходные файлы (300 dpi):
    figures/fig1_mechanism.png
    figures/fig2_ze.png
    figures/fig3_ptm.png
    figures/fig4_senescence.png
"""

import os
import sys
import warnings
import numpy as np

try:
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import matplotlib.patches as mpatches
    from matplotlib.patches import FancyArrowPatch, FancyBboxPatch
    from matplotlib.patheffects import withStroke
except ImportError:
    sys.exit("matplotlib не установлен: pip install matplotlib")

pd = None  # pandas не используется — применяем numpy-совместимый CSV-парсер

# ─── Цветовая палитра ────────────────────────────────────────────────────────
NAVY   = "#0f1f3d"
GOLD   = "#c9a84c"
RED    = "#c0392b"
BLUE   = "#2471a3"
GREEN  = "#27ae60"
PURPLE = "#8e44ad"
ORANGE = "#e67e22"
TEAL   = "#16a085"

DPI    = 300
os.makedirs("figures", exist_ok=True)

# ─────────────────────────────────────────────────────────────────────────────
# Вспомогательные функции
# ─────────────────────────────────────────────────────────────────────────────

def load_csv(path, stub_fn):
    """Загрузить CSV или создать заглушку с предупреждением."""
    if os.path.exists(path):
        if pd is not None:
            return pd.read_csv(path)
        else:
            # Fallback: простой парсинг numpy
            return _load_csv_numpy(path)
    else:
        warnings.warn(f"CSV не найден: {path} — использую заглушку", stacklevel=2)
        return stub_fn()


def _load_csv_numpy(path):
    """Минимальный парсинг CSV без pandas."""
    with open(path) as f:
        header = f.readline().strip().split(",")
        rows = [line.strip().split(",") for line in f if line.strip()]

    class _DF:
        def __init__(self, h, r):
            self._h = h
            self._r = r

        def __getitem__(self, col):
            idx = self._h.index(col)
            vals = []
            for row in self._r:
                try:
                    vals.append(float(row[idx]))
                except ValueError:
                    vals.append(row[idx])
            return np.array(vals) if all(isinstance(v, float) for v in vals) else vals

        def unique(self, col):
            return list(dict.fromkeys(self[col]))

        def filter(self, col, val):
            idx = self._h.index(col)
            rows = [r for r in self._r if r[idx] == val]
            return _DF(self._h, rows)

    return _DF(header, rows)


def _get_col(df, col):
    """Получить колонку независимо от типа (pandas или наш _DF)."""
    if pd is not None and isinstance(df, pd.DataFrame):
        return df[col].values
    return df[col]


def _filter_df(df, col, val):
    """Фильтр строк."""
    if pd is not None and isinstance(df, pd.DataFrame):
        return df[df[col] == val]
    return df.filter(col, val)


def _unique(df, col):
    """Уникальные значения."""
    if pd is not None and isinstance(df, pd.DataFrame):
        return df[col].unique().tolist()
    return df.unique(col)


# ─────────────────────────────────────────────────────────────────────────────
# Fig 1 — CDATA Mechanism Overview (схематичная диаграмма)
# ─────────────────────────────────────────────────────────────────────────────

def make_fig1():
    fig, ax = plt.subplots(figsize=(14, 5), facecolor=NAVY)
    ax.set_facecolor(NAVY)
    ax.set_xlim(0, 14)
    ax.set_ylim(0, 5)
    ax.axis("off")

    # Заголовок
    ax.text(7, 4.5, "CDATA: Centriolar Damage Accumulation Theory of Aging",
            ha="center", va="center", fontsize=13, fontweight="bold",
            color=GOLD, fontfamily="sans-serif")

    # Блоки (x_center, y_center, label, sublabel)
    blocks = [
        (1.2,  2.5, "O₂ →\nCentriole",     "Oxygen exposure",    BLUE),
        (3.4,  2.5, "PTM\nAccumulation",    "CEP164↓ CEP89↓",    GOLD),
        (5.6,  2.5, "Cilia↓\nSpindle↓",    "Track A + B",        ORANGE),
        (7.8,  2.5, "Stem Cell\nExhaustion","SASP↑ ROS↑",        RED),
        (10.0, 2.5, "Inflammaging",         "Myeloid shift",     PURPLE),
        (12.2, 2.5, "Aging /\nDeath",       "Frailty > 0.95",    "#888888"),
    ]

    box_w, box_h = 1.7, 1.1

    for (cx, cy, label, sublabel, color) in blocks:
        # Рамка
        rect = FancyBboxPatch(
            (cx - box_w / 2, cy - box_h / 2), box_w, box_h,
            boxstyle="round,pad=0.05",
            facecolor=color, edgecolor="white", linewidth=1.5,
            alpha=0.85
        )
        ax.add_patch(rect)
        ax.text(cx, cy + 0.18, label, ha="center", va="center",
                fontsize=9, fontweight="bold", color="white", fontfamily="sans-serif")
        ax.text(cx, cy - 0.28, sublabel, ha="center", va="center",
                fontsize=7, color="#dddddd", fontfamily="sans-serif")

    # Стрелки между блоками
    arrow_kw = dict(arrowstyle="-|>", color=GOLD,
                    lw=1.8, mutation_scale=14,
                    connectionstyle="arc3,rad=0.0")
    xs = [b[0] for b in blocks]
    for i in range(len(xs) - 1):
        x0 = xs[i] + box_w / 2 + 0.05
        x1 = xs[i + 1] - box_w / 2 - 0.05
        ax.annotate("", xy=(x1, 2.5), xytext=(x0, 2.5),
                    arrowprops=arrow_kw)

    # Ze Theory аннотация
    ax.text(7, 0.7, "Ze Theory: v* = 0.456 — критическая скорость биологического времени",
            ha="center", va="center", fontsize=8, color=GOLD,
            fontstyle="italic", fontfamily="sans-serif")
    ax.text(7, 0.25, "PTM accumulation = irreversible translation of biological time into space (Tkemaladze)",
            ha="center", va="center", fontsize=7.5, color="#aaaaaa", fontfamily="sans-serif")

    plt.tight_layout()
    path = "figures/fig1_mechanism.png"
    plt.savefig(path, dpi=DPI, bbox_inches="tight", facecolor=NAVY)
    plt.close()
    print(f"Fig 1 saved: {path}")


# ─────────────────────────────────────────────────────────────────────────────
# Fig 2 — Ze-velocity trajectory
# ─────────────────────────────────────────────────────────────────────────────

def _stub_ze():
    ages = np.arange(5, 106, 5)
    v = 0.456 + (ages - 20) * 0.004 + (ages / 100) ** 2 * 0.05
    v = np.clip(v, 0.3, 0.99)
    if pd is not None:
        return pd.DataFrame({"age_years": ages, "v_consensus": v})
    class _DF2:
        def __getitem__(self, col):
            return {"age_years": ages, "v_consensus": v}[col]
    return _DF2()


def make_fig2():
    def _stub():
        ages = np.arange(5, 106, 5)
        v = 0.456 + (ages - 20) * 0.004 + (ages / 100) ** 2 * 0.05
        v = np.clip(v, 0.3, 0.99)
        if pd is not None:
            return pd.DataFrame({"age_years": ages, "v_consensus": v})
        class _D:
            def __getitem__(self, c):
                return {"age_years": ages, "v_consensus": v}[c]
        return _D()

    df = load_csv("viz_output/ze_trajectory.csv", _stub)
    ages = _get_col(df, "age_years").astype(float)
    v    = _get_col(df, "v_consensus").astype(float)

    fig, ax = plt.subplots(figsize=(10, 6))

    # Заштрихованные зоны
    V_OPT = 0.456
    zones = [
        (0.0,  V_OPT,             "#27ae6020", "Optimal (v < v*)"),
        (V_OPT, 0.55,             "#f1c40f25", "Mild aging"),
        (0.55,  0.70,             "#e67e2225", "Moderate aging"),
        (0.70,  0.85,             "#e74c3c25", "Severe aging"),
        (0.85,  1.00,             "#8e44ad25", "Collapse"),
    ]
    for (y0, y1, color, label) in zones:
        ax.axhspan(y0, y1, alpha=1.0, color=color, label=label)

    # Ze-velocity линия
    ax.plot(ages, v, color=NAVY, linewidth=2.5, label="Ze-velocity (Normal HSC)", zorder=5)

    # v* пунктирная линия
    ax.axhline(V_OPT, color=GOLD, linestyle="--", linewidth=2.0,
               label=f"v* = {V_OPT} (optimal)", zorder=4)

    ax.set_xlabel("Age (years)", fontsize=12)
    ax.set_ylabel("v_consensus", fontsize=12)
    ax.set_title("Ze-Velocity Trajectory: Blood HSC Aging", fontsize=13, fontweight="bold")
    ax.set_xlim(0, max(ages) + 5)
    ax.set_ylim(0.3, 1.0)
    ax.legend(loc="upper left", fontsize=9, framealpha=0.85)
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    path = "figures/fig2_ze.png"
    plt.savefig(path, dpi=DPI, bbox_inches="tight")
    plt.close()
    print(f"Fig 2 saved: {path}")


# ─────────────────────────────────────────────────────────────────────────────
# Fig 3 — PTM trajectory (5 lines)
# ─────────────────────────────────────────────────────────────────────────────

def make_fig3():
    def _stub():
        ages = np.arange(0, 121, 5)
        data = {
            "tissue": ["Blood HSC"] * len(ages),
            "age":    ages,
            "carbonylation":   np.clip(ages * 0.0055, 0, 1),
            "hyperacetylation":np.clip(ages * 0.0045, 0, 1),
            "aggregation":     np.clip(ages * 0.0065, 0, 1),
            "phospho_dysreg":  np.clip(ages * 0.0042, 0, 1),
            "appendage_loss":  np.clip(ages * 0.0048, 0, 1),
        }
        if pd is not None:
            return pd.DataFrame(data)
        class _D:
            def __getitem__(self, c):
                return np.array(data[c])
        return _D()

    df = load_csv("viz_output/ptm_trajectory.csv", _stub)
    # Берём только Blood HSC для наглядности
    try:
        tissues = _unique(df, "tissue")
        tissue = "Blood HSC" if "Blood HSC" in tissues else tissues[0]
        df_t = _filter_df(df, "tissue", tissue)
    except Exception:
        df_t = df

    try:
        ages = _get_col(df_t, "age").astype(float)
    except Exception:
        ages = _get_col(df_t, "age_years").astype(float)

    tracks = [
        ("carbonylation",    "Protein Carbonylation",  RED),
        ("hyperacetylation", "Hyperacetylation",       BLUE),
        ("aggregation",      "Protein Aggregates",     NAVY),
        ("phospho_dysreg",   "Phospho Dysregulation",  PURPLE),
        ("appendage_loss",   "Appendage Loss",         ORANGE),
    ]

    fig, ax = plt.subplots(figsize=(10, 6))

    for (col, label, color) in tracks:
        try:
            vals = _get_col(df_t, col).astype(float)
            ax.plot(ages, vals, color=color, linewidth=2.0, label=label)
        except Exception as e:
            warnings.warn(f"Колонка {col} не найдена: {e}")

    ax.axhline(0.50, color="black", linestyle="--", linewidth=1.5,
               label="Threshold 0.50 (senescence risk)", zorder=5)

    ax.set_xlabel("Age (years)", fontsize=12)
    ax.set_ylabel("PTM Burden [0..1]", fontsize=12)
    ax.set_title("PTM Accumulation Trajectories — Blood HSC", fontsize=13, fontweight="bold")
    ax.set_xlim(0, max(ages) + 5)
    ax.set_ylim(-0.02, 1.05)
    ax.legend(loc="upper left", fontsize=9, framealpha=0.85)
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    path = "figures/fig3_ptm.png"
    plt.savefig(path, dpi=DPI, bbox_inches="tight")
    plt.close()
    print(f"Fig 3 saved: {path}")


# ─────────────────────────────────────────────────────────────────────────────
# Fig 4 — Senescence Cascade (Control vs Senolytic)
# ─────────────────────────────────────────────────────────────────────────────

def make_fig4():
    def _stub():
        ages = np.arange(10, 101, 10)
        ctrl = np.clip(0.01 * np.exp(ages * 0.04), 0, 0.9)
        seno = ctrl.copy()
        seno[ages >= 60] *= 0.5
        rows = (
            [{"scenario": "Control",  "age": a, "senescent_fraction": c}
             for a, c in zip(ages, ctrl)] +
            [{"scenario": "Senolytic","age": a, "senescent_fraction": s}
             for a, s in zip(ages, seno)]
        )
        if pd is not None:
            return pd.DataFrame(rows)
        class _D:
            def __getitem__(self, c):
                return np.array([r[c] for r in rows])
        return _D()

    df = load_csv("cell_cycle_output/senescence_cascade.csv", _stub)

    scenarios = _unique(df, "scenario")
    colors = {"Control": RED, "Senolytic": BLUE}

    fig, ax = plt.subplots(figsize=(10, 6))

    for sc in scenarios:
        df_s  = _filter_df(df, "scenario", sc)
        ages  = _get_col(df_s, "age").astype(float)
        frac  = _get_col(df_s, "senescent_fraction").astype(float) * 100.0
        color = colors.get(sc, NAVY)
        ax.plot(ages, frac, color=color, linewidth=2.5, label=sc, marker="o",
                markersize=4, zorder=5)

    # Аннотация «Senolytic applied» в точке 60 лет
    try:
        df_s60  = _filter_df(df, "scenario", "Senolytic")
        ages60  = _get_col(df_s60, "age").astype(float)
        frac60  = _get_col(df_s60, "senescent_fraction").astype(float) * 100.0
        idx60   = np.argmin(np.abs(ages60 - 60))
        y60     = float(frac60[idx60])
        ax.annotate(
            "Senolytic applied",
            xy=(60, y60),
            xytext=(65, y60 + 5),
            arrowprops=dict(arrowstyle="-|>", color="black", lw=1.5),
            fontsize=10, ha="left", va="center",
        )
    except Exception:
        pass

    ax.set_xlabel("Age (years)", fontsize=12)
    ax.set_ylabel("Senescent Fraction (%)", fontsize=12)
    ax.set_title("Senescence Cascade: Control vs Senolytic Intervention", fontsize=13, fontweight="bold")
    ax.set_ylim(0, None)
    ax.legend(fontsize=11, framealpha=0.85)
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    path = "figures/fig4_senescence.png"
    plt.savefig(path, dpi=DPI, bbox_inches="tight")
    plt.close()
    print(f"Fig 4 saved: {path}")


# ─────────────────────────────────────────────────────────────────────────────
# Главный блок
# ─────────────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    print("=== CDATA Thesis Figures (P64) ===")
    print()

    make_fig1()
    make_fig2()
    make_fig3()
    make_fig4()

    print()
    print("=== ALL FIGURES SAVED (300 dpi) ===")
    for f in ["fig1_mechanism", "fig2_ze", "fig3_ptm", "fig4_senescence"]:
        p = f"figures/{f}.png"
        if os.path.exists(p):
            size = os.path.getsize(p)
            print(f"  {p:40s}  {size//1024} KB")
        else:
            print(f"  {p:40s}  MISSING!")
