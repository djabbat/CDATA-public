"""
CDATA — Рисунок: Молекулярный механизм асимметричного наследования центриолей
Для раздела 2.2 статьи CDATA_Theory_Full_Article.docx
"""
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch, Arc, Wedge, Circle
import matplotlib.patheffects as pe
from matplotlib.gridspec import GridSpec
import numpy as np
import os

OUT = '/home/oem/Desktop/CDATA/figures'
os.makedirs(OUT, exist_ok=True)

C = {
    'old':    '#C0392B',    # старая центриоль — красный
    'new':    '#2980B9',    # молодая центриоль — синий
    'stem':   '#27AE60',    # стволовая клетка — зелёный
    'diff':   '#8E44AD',    # дифференцирующаяся — фиолетовый
    'niche':  '#F39C12',    # ниша — жёлтый
    'cilia':  '#1ABC9C',    # ресничка — бирюзовый
    'damage': '#E67E22',    # повреждение — оранжевый
    'ros':    '#E74C3C',    # ROS
    'bg':     '#FAFAFA',
    'panel':  '#F0F4F8',
    'text':   '#2C3E50',
    'arrow':  '#5D6D7E',
    'inducer':'#F1C40F',    # индукторы — золотой
    'membrane':'#85C1E9',   # мембрана
    'appnd':  '#A569BD',    # appendages
}

# ════════════════════════════════════════════════════════════════════════════
# БОЛЬШОЙ РИСУНОК: 4 панели
# A — Молекулярный якорный механизм
# B — Три модельных системы (Drosophila, NSC, HSC)
# C — Рэтчет по поколениям (индукторная система)
# D — Следствия: старение vs. рак
# ════════════════════════════════════════════════════════════════════════════

fig = plt.figure(figsize=(20, 24))
fig.patch.set_facecolor('#FFFFFF')

gs = GridSpec(3, 2, figure=fig,
              hspace=0.10, wspace=0.08,
              left=0.04, right=0.97,
              top=0.97, bottom=0.03)

# ── Заголовок ───────────────────────────────────────────────────────────────
fig.text(0.5, 0.985,
         'Why Does the Mother Centriole Stay in the Stem Cell?',
         ha='center', va='top', fontsize=22, fontweight='bold', color=C['text'])
fig.text(0.5, 0.975,
         'Molecular Mechanisms of Asymmetric Centriole Inheritance and the Centriolar Damage Ratchet',
         ha='center', va='top', fontsize=13, color='#7F8C8D', style='italic')

# ════════════════════════════════════════════════════════════════════════════
# ПАНЕЛЬ A — Молекулярный механизм якорения
# ════════════════════════════════════════════════════════════════════════════
ax_a = fig.add_subplot(gs[0, 0])
ax_a.set_facecolor(C['bg'])
ax_a.set_xlim(0, 10); ax_a.set_ylim(0, 10)
ax_a.axis('off')
ax_a.text(0.02, 0.98, 'A', transform=ax_a.transAxes,
          fontsize=20, fontweight='bold', color=C['text'], va='top')
ax_a.text(5, 9.7, 'Molecular Anchoring Mechanism',
          ha='center', fontsize=13, fontweight='bold', color=C['text'])
ax_a.text(5, 9.3, 'Three structural layers enforce mother centriole retention in the stem cell',
          ha='center', fontsize=9, color='#7F8C8D', style='italic')

# ─ Плазматическая мембрана ─
mem_y = 7.8
ax_a.fill_between([0.5, 9.5], [mem_y - 0.18], [mem_y + 0.18],
                   color=C['membrane'], alpha=0.7, zorder=2)
ax_a.text(9.3, mem_y, 'Apical\nmembrane', ha='right', va='center',
          fontsize=8, color='#2980B9', fontweight='bold')

# ─ Дистальные придатки CEP164 ─
app_xs = [3.8, 4.2, 4.6, 5.0, 5.4, 5.8, 6.2]
for xi in app_xs:
    ax_a.plot([xi, xi], [mem_y - 0.18, 6.7], '-', color=C['appnd'], lw=2, zorder=3)
    ax_a.plot(xi, 6.65, 'v', ms=6, color=C['appnd'], zorder=4)

ax_a.text(5.0, 6.35, 'Distal appendages\n(CEP164, CEP89, SCLT1, FBF1)',
          ha='center', fontsize=8, color=C['appnd'], fontweight='bold')

# ─ Первичная ресничка ─
ax_a.add_patch(plt.Rectangle((4.7, mem_y + 0.18), 0.5, 1.6,
                               facecolor=C['cilia'], alpha=0.7, zorder=3, linewidth=1.5,
                               edgecolor=C['cilia']))
ax_a.text(4.94, mem_y + 1.7, 'Primary\ncilium', ha='center', va='bottom',
          fontsize=8, color=C['cilia'], fontweight='bold')

# Сигнальные молекулы на реснице
for yi, sig, col in [(mem_y + 0.6, 'Shh', '#E74C3C'),
                      (mem_y + 1.0, 'Wnt', '#27AE60'),
                      (mem_y + 1.4, 'Notch', '#8E44AD')]:
    ax_a.annotate('', xy=(5.85, yi), xytext=(5.22, yi),
                  arrowprops=dict(arrowstyle='->', color=col, lw=1.5))
    ax_a.text(6.0, yi, sig, ha='left', va='center', fontsize=8,
              fontweight='bold', color=col)

# ─ Тело материнской центриоли ─
old_c = plt.Circle((5.0, 5.9), 0.6, facecolor=C['old'], alpha=0.85,
                    edgecolor='#922B21', lw=2, zorder=5)
ax_a.add_patch(old_c)
ax_a.text(5.0, 5.9, '★\nM', ha='center', va='center',
          fontsize=10, fontweight='bold', color='white', zorder=6)

# Кольцо повреждений вокруг
theta = np.linspace(0, 2*np.pi, 16)
for t in theta[::2]:
    xi = 5.0 + 0.75 * np.cos(t)
    yi = 5.9 + 0.75 * np.sin(t)
    ax_a.text(xi, yi, '×', ha='center', va='center',
              fontsize=7, color=C['damage'], fontweight='bold', zorder=7)

ax_a.text(3.4, 5.9, 'Mother centriole\n(old, PTM-damaged)', ha='center', va='center',
          fontsize=8, color=C['old'], fontweight='bold')
ax_a.annotate('', xy=(4.35, 5.9), xytext=(3.9, 5.9),
              arrowprops=dict(arrowstyle='->', color=C['old'], lw=1.5))

# Индукторы вокруг старой центриоли
for i, t in enumerate([0.3, 1.0, 1.7, 2.4, 3.1]):
    xi = 5.0 + 1.05 * np.cos(t)
    yi = 5.9 + 1.05 * np.sin(t)
    ax_a.add_patch(plt.Circle((xi, yi), 0.13, color=C['inducer'], zorder=8,
                               edgecolor='#D4AC0D', lw=1))
ax_a.text(6.8, 6.8, 'Differentiation\ninducers (I)',
          ha='left', fontsize=7.5, color='#D4AC0D', fontweight='bold')

# ─ Дочерняя центриоль ─
new_c = plt.Circle((5.0, 4.5), 0.45, facecolor=C['new'], alpha=0.8,
                    edgecolor='#1A5276', lw=2, zorder=5)
ax_a.add_patch(new_c)
ax_a.text(5.0, 4.5, 'D', ha='center', va='center',
          fontsize=10, fontweight='bold', color='white', zorder=6)
ax_a.text(3.4, 4.4, 'Daughter centriole\n(new, undamaged)', ha='center', va='center',
          fontsize=8, color=C['new'], fontweight='bold')
ax_a.annotate('', xy=(4.5, 4.5), xytext=(4.0, 4.4),
              arrowprops=dict(arrowstyle='->', color=C['new'], lw=1.5))

# Связь мать-дочь
ax_a.plot([5.0, 5.0], [5.3, 4.95], 'k-', lw=2, zorder=4)

# ─ Fate determinants вокруг полюса ─
for t in [0.1, 0.8, 1.5]:
    xi = 5.0 + 1.35 * np.cos(t + 1.57)
    yi = 5.9 + 1.35 * np.sin(t + 1.57)
    ax_a.add_patch(FancyBboxPatch((xi - 0.25, yi - 0.12), 0.5, 0.24,
                                   boxstyle='round,pad=0.03',
                                   facecolor='#F8C471', edgecolor='#E67E22', lw=1, zorder=7))
    ax_a.text(xi, yi, 'Numb\n/aPKC'[::5 if t < 1 else 1][:4], ha='center', va='center',
              fontsize=5.5, color='#784212', zorder=8)

ax_a.text(7.0, 7.5, 'Fate\ndeterminants\n(Numb, aPKC,\nPar complex)',
          ha='left', fontsize=7.5, color='#784212', fontweight='bold')

# ─ Слой PCM ─
pcm = plt.Circle((5.0, 5.9), 1.3, facecolor='none',
                  edgecolor='#AAB7B8', lw=1.5, ls='--', zorder=4)
ax_a.add_patch(pcm)
ax_a.text(7.0, 5.2, 'PCM\n(regulatory)', ha='left', fontsize=7.5,
          color='#717D7E', style='italic')

# ─ Три причины (снизу) ─
reasons = [
    (1.7, 2.5, '#1A5276', 'LAYER 1\nStructural anchor',
     'Distal appendages (CEP164)\ndock to apical membrane\n→ mother physically tethered'),
    (5.0, 2.5, '#7D6608', 'LAYER 2\nFate determinants',
     'Numb/aPKC/Par concentrate\naround mother centriole pole\n→ stem fate segregates with it'),
    (8.3, 2.5, '#4A235A', 'LAYER 3\nSpindle geometry',
     'Anchored mother defines\nspindle axis perpendicular\nto niche → ACD enforced'),
]
for rx, ry, col, title, desc in reasons:
    ax_a.add_patch(FancyBboxPatch((rx - 1.5, ry - 0.85), 3.0, 1.7,
                                   boxstyle='round,pad=0.12',
                                   facecolor=col + '18', edgecolor=col, lw=2, zorder=3))
    ax_a.text(rx, ry + 0.52, title, ha='center', fontsize=8.5,
              fontweight='bold', color=col)
    ax_a.text(rx, ry - 0.1, desc, ha='center', fontsize=7.5,
              color=C['text'], linespacing=1.4)

ax_a.text(5.0, 0.3,
          '→  All three layers converge to ensure P(mother retained by stem cell) > 0.5  ←',
          ha='center', fontsize=8.5, fontweight='bold', color=C['old'],
          bbox=dict(fc='#FADBD8', ec=C['old'], pad=4, boxstyle='round'))

# ════════════════════════════════════════════════════════════════════════════
# ПАНЕЛЬ B — Три модельных системы
# ════════════════════════════════════════════════════════════════════════════
ax_b = fig.add_subplot(gs[0, 1])
ax_b.set_facecolor(C['bg'])
ax_b.set_xlim(0, 10); ax_b.set_ylim(0, 10)
ax_b.axis('off')
ax_b.text(0.02, 0.98, 'B', transform=ax_b.transAxes,
          fontsize=20, fontweight='bold', color=C['text'], va='top')
ax_b.text(5, 9.7, 'Three Experimental Systems',
          ha='center', fontsize=13, fontweight='bold', color=C['text'])
ax_b.text(5, 9.3, 'Asymmetric inheritance confirmed in Drosophila, neural progenitors & HSCs',
          ha='center', fontsize=9, color='#7F8C8D', style='italic')

systems = [
    # (center_x, center_y, color, label, mechanism_text, evidence_text)
    (2.2, 7.0, '#E74C3C', 'Drosophila\nGSC',
     'Hub cells secrete Unpaired\n(JAK-STAT) + BMP/Dpp.\nMother centriole anchored\nto hub-proximal cortex\nvia appendages.\n\n100% deterministic\nasymmetric inheritance.',
     'Yamashita et al.\nScience 2007\n(laser ablation\ndisrupts ACD)'),
    (5.0, 7.0, '#27AE60', 'Neural\nProgenitor',
     'Mother centriole is\napically anchored to\nventricular surface\nvia primary cilium.\nDaughter goes\nbasolaterally.\n\nOrientation enforced\nby LINC complex.',
     'Wang et al.\nNature 2009\n(depletion of\nappendage proteins\n→ pool exhaustion)'),
    (7.8, 7.0, '#2980B9', 'HSC\n(Bone Marrow)',
     'Stochastic asymmetry:\nprobabilistic bias >50%.\nMother centriole PCM\ncarries aged cell-fate\ndeterminants.\nNiche retention by\nbetter cilia → Shh.\n\nTrack B-independent.',
     'Habib et al.\nScience 2013;\nVertii et al.\nCSH Perspect 2016'),
]

for sx, sy, col, label, mech, evid in systems:
    # Клетка
    cell = plt.Circle((sx, sy), 1.5, facecolor=col + '18',
                       edgecolor=col, lw=2, zorder=3)
    ax_b.add_patch(cell)
    ax_b.text(sx, sy + 1.65, label, ha='center', va='bottom',
              fontsize=10, fontweight='bold', color=col)

    # Старая центриоль (вверху)
    oc = plt.Circle((sx - 0.25, sy + 0.5), 0.28, color=C['old'], zorder=5)
    ax_b.add_patch(oc)
    ax_b.text(sx - 0.25, sy + 0.5, '★', ha='center', va='center',
              fontsize=9, color='white', zorder=6)
    # Молодая центриоль (внизу)
    nc = plt.Circle((sx + 0.25, sy - 0.3), 0.2, color=C['new'], zorder=5)
    ax_b.add_patch(nc)

    # Ресничка
    ax_b.add_patch(plt.Rectangle((sx - 0.38, sy + 0.78), 0.2, 0.65,
                                   facecolor=C['cilia'], alpha=0.8, zorder=4))

    # Стрелки деления
    ax_b.annotate('', xy=(sx - 1.0, sy - 1.5), xytext=(sx - 0.3, sy - 1.5),
                  arrowprops=dict(arrowstyle='->', color=C['old'], lw=2))
    ax_b.annotate('', xy=(sx + 1.0, sy - 1.5), xytext=(sx + 0.3, sy - 1.5),
                  arrowprops=dict(arrowstyle='->', color=C['new'], lw=2))

    # Дочерние клетки
    sc_d = plt.Circle((sx - 1.2, sy - 2.3), 0.55, facecolor=C['stem'] + '44',
                       edgecolor=C['stem'], lw=1.8, zorder=3)
    ax_b.add_patch(sc_d)
    ax_b.text(sx - 1.2, sy - 2.3, '★', ha='center', va='center',
              fontsize=12, color=C['old'], zorder=4)
    ax_b.text(sx - 1.2, sy - 3.05, 'SC', ha='center', fontsize=7.5,
              color=C['stem'], fontweight='bold')

    pr_d = plt.Circle((sx + 1.2, sy - 2.3), 0.55, facecolor=C['diff'] + '33',
                       edgecolor=C['diff'], lw=1.8, zorder=3)
    ax_b.add_patch(pr_d)
    ax_b.text(sx + 1.2, sy - 2.3, '○', ha='center', va='center',
              fontsize=14, color=C['new'], zorder=4)
    ax_b.text(sx + 1.2, sy - 3.05, 'Prog.', ha='center', fontsize=7.5,
              color=C['diff'], fontweight='bold')

    # Механизм (мелкий текст)
    ax_b.text(sx, sy - 4.4, mech, ha='center', va='top',
              fontsize=6.8, color=C['text'], linespacing=1.35,
              bbox=dict(fc=col + '12', ec=col + '55', pad=4, boxstyle='round'))

    # Evidence
    ax_b.text(sx, sy - 7.3, evid, ha='center', va='top',
              fontsize=6.5, color='#7F8C8D', style='italic', linespacing=1.3)

# Вертикальные разделители
for xd in [3.3, 6.7]:
    ax_b.plot([xd, xd], [0.2, 9.5], '--', color='#CCCCCC', lw=1)

# ════════════════════════════════════════════════════════════════════════════
# ПАНЕЛЬ C — Рэтчет по поколениям (индукторная система)
# ════════════════════════════════════════════════════════════════════════════
ax_c = fig.add_subplot(gs[1, :])
ax_c.set_facecolor(C['bg'])
ax_c.set_xlim(0, 20); ax_c.set_ylim(0, 7.5)
ax_c.axis('off')
ax_c.text(0.01, 0.99, 'C', transform=ax_c.transAxes,
          fontsize=20, fontweight='bold', color=C['text'], va='top')
ax_c.text(10, 7.2, 'The Damage Ratchet: Centriolar Inducer Depletion Across Stem Cell Generations',
          ha='center', fontsize=13, fontweight='bold', color=C['text'])
ax_c.text(10, 6.8,
          'With each division, the mother centriole (★) stays in the stem cell, accumulating PTM damage and losing inducer molecules (◆)',
          ha='center', fontsize=9, color='#7F8C8D', style='italic')

# Параметры поколений
generations = 6
gen_x = [1.2, 4.0, 6.8, 9.6, 12.4, 15.2]
gen_labels = ['G1\n(Young)', 'G2', 'G3', 'G4', 'G5', f'G{generations}\n(Old)']
M_counts = [5, 4, 3, 2, 1, 0]        # индукторы на старой центриоли
potency   = ['Totipotent', 'Pluripotent', 'Pluripotent', 'Oligopotent', 'Unipotent', 'Senescent']
pot_cols  = [C['stem'], '#27AE60', '#2ECC71', '#F39C12', C['damage'], C['ros']]

for i, (gx, glbl, Mc, pot, pc) in enumerate(zip(gen_x, gen_labels, M_counts, potency, pot_cols)):

    # Клетка-СК
    cell_r = 1.1
    cell = plt.Circle((gx, 4.5), cell_r, facecolor=pc + '22',
                       edgecolor=pc, lw=2.5, zorder=3)
    ax_c.add_patch(cell)

    # Метка поколения
    ax_c.text(gx, 5.85, glbl, ha='center', fontsize=9, fontweight='bold',
              color=pc)

    # Материнская центриоль (всегда красная, стареет)
    dam = min(i * 0.15, 0.7)
    oc_r = 0.28 + i * 0.025
    oc = plt.Circle((gx - 0.22, 4.65), oc_r,
                    facecolor=C['old'], alpha=0.9 - dam * 0.3,
                    edgecolor='#922B21', lw=1.5, zorder=5)
    ax_c.add_patch(oc)
    ax_c.text(gx - 0.22, 4.65, '★', ha='center', va='center',
              fontsize=9, color='white', zorder=6)

    # Повреждения на старой центриоли
    for di in range(i):
        t_d = di * 1.1 + 0.3
        xd = gx - 0.22 + (oc_r + 0.12) * np.cos(t_d)
        yd = 4.65 + (oc_r + 0.12) * np.sin(t_d)
        ax_c.text(xd, yd, '×', ha='center', va='center',
                  fontsize=7, color=C['damage'], fontweight='bold', zorder=7)

    # Молодая центриоль
    nc = plt.Circle((gx + 0.3, 4.3), 0.2, facecolor=C['new'],
                    alpha=0.85, edgecolor='#1A5276', lw=1.2, zorder=5)
    ax_c.add_patch(nc)

    # Индукторы на старой центриоли (убывают)
    for j in range(Mc):
        t_ind = j * (2 * np.pi / max(Mc, 1)) + 0.5
        xi = gx - 0.22 + 0.55 * np.cos(t_ind)
        yi = 4.65 + 0.55 * np.sin(t_ind)
        ax_c.add_patch(plt.Circle((xi, yi), 0.1, color=C['inducer'],
                                   zorder=8, edgecolor='#D4AC0D', lw=1))

    # Подпись потентности
    ax_c.text(gx, 3.15, pot, ha='center', fontsize=8,
              color=pc, fontweight='bold',
              bbox=dict(fc=pc + '20', ec=pc + '80', pad=2, boxstyle='round'))

    # Стрелка к дифференцирующейся дочери
    if i < generations - 1:
        # Горизонтальная стрелка к следующему поколению СК
        ax_c.annotate('', xy=(gen_x[i+1] - cell_r - 0.1, 4.5),
                      xytext=(gx + cell_r + 0.1, 4.5),
                      arrowprops=dict(arrowstyle='->', color=pc,
                                      lw=2.2, connectionstyle='arc3,rad=0'))
        # Стрелка вниз к дифф. клетке
        diff_y = 1.9
        ax_c.annotate('', xy=(gx + 0.5, diff_y + 0.45),
                      xytext=(gx + 0.3, 4.3 - 0.22),
                      arrowprops=dict(arrowstyle='->', color=C['diff'],
                                      lw=1.5, connectionstyle='arc3,rad=0.2'))
        # Дифф. дочь
        dc = plt.Circle((gx + 0.7, diff_y), 0.42, facecolor=C['diff'] + '22',
                         edgecolor=C['diff'], lw=1.5, zorder=3)
        ax_c.add_patch(dc)
        nc2 = plt.Circle((gx + 0.7, diff_y), 0.16, color=C['new'], zorder=5)
        ax_c.add_patch(nc2)
        # Один индуктор уходит
        ax_c.add_patch(plt.Circle((gx + 1.1, diff_y + 0.28), 0.09,
                                   color=C['inducer'], zorder=6,
                                   edgecolor='#D4AC0D', lw=0.8))
        ax_c.text(gx + 1.35, diff_y + 0.28, '→ DNA\nswitch',
                  ha='left', fontsize=6, color='#7D6608')

    # Последнее поколение — сенесценция
    if i == generations - 1:
        ax_c.text(gx, 2.4, 'SENESCENCE\nor APOPTOSIS', ha='center', fontsize=8.5,
                  fontweight='bold', color=C['ros'],
                  bbox=dict(fc='#FADBD8', ec=C['ros'], pad=3, boxstyle='round'))
        ax_c.annotate('', xy=(gx, 2.75), xytext=(gx, 4.5 - cell_r),
                      arrowprops=dict(arrowstyle='->', color=C['ros'], lw=2.5))

# Метки счётчика индукторов
ax_c.text(0.15, 4.65, 'Inducers\n◆', ha='left', fontsize=8,
          color='#D4AC0D', fontweight='bold')
ax_c.text(0.15, 3.5, 'Potency\nlevel', ha='left', fontsize=8, color=C['text'])

# Ось потентности
for xi_l, yi_l, txt in [(0.15, 1.3, 'Differentiating\ndaughter')]:
    ax_c.text(xi_l, yi_l, txt, ha='left', fontsize=8, color=C['diff'])

# Шкала повреждений
ax_c.text(10, 0.45,
          '★ = Mother centriole (accumulates PTM damage)   '
          '○ = Daughter centriole (new)   '
          '◆ = Differentiation inducer   '
          '× = PTM damage mark',
          ha='center', fontsize=9, color=C['text'],
          bbox=dict(fc='#EBF5FB', ec='#AED6F1', pad=5, boxstyle='round'))

# ════════════════════════════════════════════════════════════════════════════
# ПАНЕЛЬ D — Два исхода (нижняя, широкая)
# ════════════════════════════════════════════════════════════════════════════
ax_d = fig.add_subplot(gs[2, :])
ax_d.set_facecolor(C['bg'])
ax_d.set_xlim(0, 20); ax_d.set_ylim(0, 8.5)
ax_d.axis('off')
ax_d.text(0.01, 0.99, 'D', transform=ax_d.transAxes,
          fontsize=20, fontweight='bold', color=C['text'], va='top')
ax_d.text(10, 8.2, 'Tissue-Level Consequences: From Molecular Damage to Organismal Aging',
          ha='center', fontsize=13, fontweight='bold', color=C['text'])

# Центральный узел — повреждённая центриоль
cx, cy = 10, 5.5
center = plt.Circle((cx, cy), 1.1, facecolor=C['old'],
                     edgecolor='#922B21', lw=3, zorder=5)
ax_d.add_patch(center)
ax_d.text(cx, cy + 0.25, '★ Old\nCentriole', ha='center', va='center',
          fontsize=9.5, fontweight='bold', color='white', zorder=6)
ax_d.text(cx, cy - 0.55, 'PTM-damaged', ha='center', va='center',
          fontsize=7.5, color='#FFCCBC', zorder=6)

# ─ ТРЕК A (левая ветка) ─
ta_x, ta_y = 3.0, 5.5
track_a = [
    (5.5, 7.0, 'Distal appendage\nloss (CEP164↓)', '#1A5276'),
    (3.5, 7.0, 'No primary\ncilium', '#2980B9'),
    (1.5, 7.0, 'Wnt/Shh/Notch\nblocked', '#27AE60'),
    (1.5, 5.0, 'SC deaf to\nniche signals', C['stem']),
    (1.5, 3.0, 'Myeloid shift\n/ Neurodegeneration', '#C0392B'),
]
prev = (cx - 1.1, cy + 0.3)
for i, (tx, ty, lbl, col) in enumerate(track_a):
    ax_d.add_patch(FancyBboxPatch((tx - 1.35, ty - 0.42), 2.7, 0.84,
                                   boxstyle='round,pad=0.07',
                                   facecolor=col + '25', edgecolor=col, lw=1.8, zorder=3))
    ax_d.text(tx, ty, lbl, ha='center', va='center', fontsize=7.8,
              color=C['text'], linespacing=1.3, zorder=4)
    ax_d.annotate('', xy=(tx + 1.36, ty), xytext=(prev[0], prev[1]),
                  arrowprops=dict(arrowstyle='->', color=col, lw=1.8,
                                  connectionstyle='arc3,rad=0'), zorder=2)
    prev = (tx - 1.36, ty)

ax_d.text(3.0, 8.4, 'TRACK A:  Loss of Primary Ciliogenesis',
          ha='center', fontsize=10.5, fontweight='bold', color='#1A5276')

# ─ ТРЕК B (правая ветка) ─
track_b = [
    (14.5, 7.0, 'Spindle\nmisorientation', '#6C3483'),
    (16.5, 7.0, 'Symmetric\ndivisions ↑', '#8E44AD'),
    (18.5, 7.0, 'Two outcomes:', '#A569BD'),
    (17.5, 5.0, 'Pool\nexhaustion\n→ AGING', C['damage']),
    (19.2, 3.0, 'Clonal\nexpansion\n→ CHIP/Cancer', C['ros']),
]
prev_b = (cx + 1.1, cy + 0.3)
for i, (tx, ty, lbl, col) in enumerate(track_b):
    ax_d.add_patch(FancyBboxPatch((tx - 1.35, ty - 0.42), 2.7, 0.84,
                                   boxstyle='round,pad=0.07',
                                   facecolor=col + '25', edgecolor=col, lw=1.8, zorder=3))
    ax_d.text(tx, ty, lbl, ha='center', va='center', fontsize=7.8,
              color=C['text'], linespacing=1.3, zorder=4)
    if i < 3:
        ax_d.annotate('', xy=(tx - 1.36, ty), xytext=(prev_b[0], prev_b[1]),
                      arrowprops=dict(arrowstyle='->', color=col, lw=1.8,
                                      connectionstyle='arc3,rad=0'), zorder=2)
    prev_b = (tx + 1.36, ty)

# Ветвление двух исходов
ax_d.annotate('', xy=(17.5, 5.42), xytext=(18.5 - 1.36, 7.0 - 0.42),
              arrowprops=dict(arrowstyle='->', color=C['damage'], lw=2.0))
ax_d.annotate('', xy=(19.2, 3.42), xytext=(18.5 + 1.3, 7.0 - 0.42),
              arrowprops=dict(arrowstyle='->', color=C['ros'], lw=2.0))

ax_d.text(17.0, 8.4, 'TRACK B:  Loss of Spindle Fidelity',
          ha='center', fontsize=10.5, fontweight='bold', color='#6C3483')

# ─ ROS петля (центр) ─
ros_box = FancyBboxPatch((7.5, 2.8), 5.0, 1.3, boxstyle='round,pad=0.1',
                          facecolor='#FADBD8', edgecolor=C['ros'], lw=2.5, zorder=3)
ax_d.add_patch(ros_box)
ax_d.text(10, 3.8, '⟳  ROS Positive Feedback Loop', ha='center',
          fontsize=9.5, fontweight='bold', color=C['ros'])
ax_d.text(10, 3.25,
          'Centriole damage → MTOC disruption → Mito dysfunction → ROS↑ → Centriole damage↑',
          ha='center', fontsize=8.5, color=C['text'])

ax_d.annotate('', xy=(8.8, 3.8), xytext=(cx - 1.1, cy - 0.5),
              arrowprops=dict(arrowstyle='->', color=C['ros'], lw=1.8,
                              connectionstyle='arc3,rad=0.3'))
ax_d.annotate('', xy=(cx - 0.8, cy - 0.5), xytext=(11.2, 3.85),
              arrowprops=dict(arrowstyle='->', color=C['ros'], lw=1.8,
                              connectionstyle='arc3,rad=-0.3'))

# Эмпирические данные внизу
evidence_boxes = [
    (3.5, 0.9, C['stem'],
     'Neural: Wang et al. 2009\nAppendage depletion → NSC pool\nexhaustion in mouse'),
    (7.5, 0.9, C['new'],
     'HSC: Vertii et al. 2016\nCentrosomal aberrations\n→ myeloid bias'),
    (11.5, 0.9, C['damage'],
     'Muscle: Liang & Ghaffari 2018\nAged MuSC centrosome defects\n→ sarcopenia'),
    (15.5, 0.9, C['ros'],
     'Drosophila: Yamashita 2007\n100% deterministic mother\ncentriole retention in GSC'),
]
for ex, ey, col, txt in evidence_boxes:
    ax_d.add_patch(FancyBboxPatch((ex - 1.8, ey - 0.65), 3.6, 1.3,
                                   boxstyle='round,pad=0.08',
                                   facecolor=col + '15', edgecolor=col + '88', lw=1.5))
    ax_d.text(ex, ey, txt, ha='center', va='center', fontsize=7.0,
              color=C['text'], linespacing=1.35)

ax_d.text(10, 0.12,
          'Evidence summary: centriolar dysfunction is sufficient to recapitulate aged tissue phenotypes in young organisms',
          ha='center', fontsize=9, color='#7F8C8D', style='italic')

# ─ Сохранение ─
path = f'{OUT}/02_asymmetric_WHY_full.png'
fig.savefig(path, dpi=150, bbox_inches='tight', facecolor='white')
plt.close(fig)
print(f'✓  {path}')

# ════════════════════════════════════════════════════════════════════════════
# ОТДЕЛЬНЫЙ КОМПАКТНЫЙ ГРАФИЧЕСКИЙ АБСТРАКТ (для вставки в статью)
# ════════════════════════════════════════════════════════════════════════════

fig2, ax2 = plt.subplots(figsize=(16, 8))
fig2.patch.set_facecolor('#FFFFFF')
ax2.set_facecolor('#FFFFFF')
ax2.set_xlim(0, 16); ax2.set_ylim(0, 8)
ax2.axis('off')

ax2.text(8, 7.75,
         'Graphical Abstract: The CDATA Mechanism',
         ha='center', fontsize=16, fontweight='bold', color=C['text'])
ax2.text(8, 7.3,
         'Old Centrioles Make Old Bodies — from molecular damage to organismal aging',
         ha='center', fontsize=10, color='#7F8C8D', style='italic')

# Временная шкала вверху
for xi_t, age_t, col_t in [(1.5, 'Age 0–30', C['stem']),
                             (5.3, 'Age 30–60', C['damage']),
                             (9.5, 'Age 60–80', C['ros']),
                             (13.5, 'Age 80+', '#922B21')]:
    ax2.text(xi_t, 6.85, age_t, ha='center', fontsize=9, fontweight='bold',
             color=col_t,
             bbox=dict(fc=col_t + '22', ec=col_t, pad=3, boxstyle='round'))

ax2.plot([0.5, 15.5], [6.55, 6.55], '-', color='#BDC3C7', lw=2, zorder=2)
ax2.add_patch(FancyArrowPatch((0.4, 6.55), (15.6, 6.55),
                               arrowstyle='->', color='#BDC3C7', lw=2,
                               mutation_scale=20))
ax2.text(8, 6.25, 'TIME →', ha='center', fontsize=9, color='#BDC3C7')

# 4 стволовые клетки по временной шкале
cells_ga = [
    (1.5, 4.5, 5, '#27AE60', 0, 'Intact\ncentriole\n+ cilium\n+ ACD', 'TOTAL-\nPOTENT'),
    (5.3, 4.5, 3, C['damage'], 2, 'Partial\nPTM\ndamage\n± cilium', 'PLURI-\nPOTENT'),
    (9.5, 4.5, 1, C['ros'], 4, 'Severe\nPTM\nno cilium\nSpindle↓', 'OLIGO-\nPOTENT'),
    (13.5, 4.5, 0, '#922B21', 6, 'Terminal\nfailure\nSenescence', 'UNI-\nPOTENT'),
]
for cx2, cy2, M, col, n_dmg, func, pot in cells_ga:
    # Клетка
    c2 = plt.Circle((cx2, cy2), 1.25, facecolor=col + '18',
                     edgecolor=col, lw=2.5, zorder=3)
    ax2.add_patch(c2)
    # Центриоль
    oc2 = plt.Circle((cx2 - 0.18, cy2 + 0.3), 0.28, facecolor=C['old'], zorder=5)
    ax2.add_patch(oc2)
    ax2.text(cx2 - 0.18, cy2 + 0.3, '★', ha='center', va='center',
             fontsize=9, color='white', zorder=6)
    # Повреждения
    for di in range(n_dmg):
        t_d = di * 1.0 + 0.5
        ax2.text(cx2 - 0.18 + 0.42 * np.cos(t_d),
                 cy2 + 0.3 + 0.42 * np.sin(t_d),
                 '×', ha='center', va='center', fontsize=7,
                 color=C['damage'], fontweight='bold', zorder=7)
    # Ресничка (только у первых двух)
    if M >= 3:
        ax2.add_patch(plt.Rectangle((cx2 - 0.3, cy2 + 0.58), 0.18, 0.55,
                                     facecolor=C['cilia'], alpha=0.8, zorder=4))
    # Индукторы
    for j in range(M):
        t_ind = j * (2 * np.pi / max(M, 1)) + 0.2
        ax2.add_patch(plt.Circle((cx2 - 0.18 + 0.6 * np.cos(t_ind),
                                   cy2 + 0.3 + 0.6 * np.sin(t_ind)),
                                  0.1, color=C['inducer'], zorder=8))
    # Функция
    ax2.text(cx2, cy2 - 0.3, func, ha='center', va='center',
             fontsize=6.8, color=C['text'], linespacing=1.3)
    # Потентность
    ax2.text(cx2, cy2 - 1.6, pot, ha='center', fontsize=8.5,
             fontweight='bold', color=col,
             bbox=dict(fc=col + '20', ec=col, pad=2, boxstyle='round'))

# Стрелки между поколениями
for i in range(3):
    x1 = cells_ga[i][0] + 1.26
    x2 = cells_ga[i+1][0] - 1.26
    ax2.annotate('', xy=(x2, 4.5), xytext=(x1, 4.5),
                 arrowprops=dict(arrowstyle='->', color='#BDC3C7', lw=2.5))

# Два трека внизу
ax2.add_patch(FancyBboxPatch((0.5, 0.5), 7.0, 1.3, boxstyle='round,pad=0.1',
                              facecolor='#EBF5FB', edgecolor='#2980B9', lw=2))
ax2.text(4.0, 1.5, 'TRACK A — Ciliary failure',
         ha='center', fontsize=10, fontweight='bold', color='#1A5276')
ax2.text(4.0, 0.95,
         'CEP164↓ → No primary cilium → Wnt/Shh blind\n→ Myeloid shift / Neurodegeneration / Sarcopenia',
         ha='center', fontsize=8.5, color=C['text'])

ax2.add_patch(FancyBboxPatch((8.5, 0.5), 7.0, 1.3, boxstyle='round,pad=0.1',
                              facecolor='#FEF9E7', edgecolor='#8E44AD', lw=2))
ax2.text(12.0, 1.5, 'TRACK B — Spindle fidelity loss',
         ha='center', fontsize=10, fontweight='bold', color='#6C3483')
ax2.text(12.0, 0.95,
         'Spindle misorientation → Symmetric divisions\n→ Pool exhaustion (aging) OR Clonal expansion (CHIP/cancer)',
         ha='center', fontsize=8.5, color=C['text'])

path2 = f'{OUT}/02b_graphical_abstract.png'
fig2.savefig(path2, dpi=150, bbox_inches='tight', facecolor='white')
plt.close(fig2)
print(f'✓  {path2}')

print('\n✅  Готово!')
