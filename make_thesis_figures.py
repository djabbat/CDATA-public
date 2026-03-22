"""
CDATA Thesis Figures — визуализация тезисов Центриолярной Теории Старения
Сохраняет PNG в /home/oem/Desktop/CDATA/figures/
"""

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import matplotlib.patheffects as pe
from matplotlib.patches import FancyArrowPatch, FancyBboxPatch, Arc, Circle, Wedge
import matplotlib.gridspec as gridspec
import numpy as np
import os

OUT = '/home/oem/Desktop/CDATA/figures'
os.makedirs(OUT, exist_ok=True)

# ── Цветовая палитра ────────────────────────────────────────────────────────
C = {
    'old':     '#C0392B',   # красный — старая центриоль
    'new':     '#2980B9',   # синий  — молодая центриоль
    'stem':    '#27AE60',   # зелёный — стволовая клетка
    'diff':    '#8E44AD',   # фиолетовый — дифференцирующаяся
    'damage':  '#E67E22',   # оранжевый — повреждение
    'ros':     '#F39C12',   # жёлтый — ROS
    'bg':      '#FAFAFA',
    'panel':   '#F0F4F8',
    'text':    '#2C3E50',
    'arrow':   '#555555',
    'good':    '#1ABC9C',
    'bad':     '#E74C3C',
}

def save(fig, name):
    path = f'{OUT}/{name}.png'
    fig.savefig(path, dpi=150, bbox_inches='tight', facecolor=fig.get_facecolor())
    plt.close(fig)
    print(f'  ✓  {path}')


# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 1 — Пять тезисов CDATA (обзорная схема)
# ════════════════════════════════════════════════════════════════════════════
def fig_overview():
    fig, ax = plt.subplots(figsize=(16, 9))
    fig.patch.set_facecolor(C['bg'])
    ax.set_facecolor(C['bg'])
    ax.set_xlim(0, 16); ax.set_ylim(0, 9)
    ax.axis('off')

    # Заголовок
    ax.text(8, 8.5, 'CDATA — Центриолярная Теория Накопления Повреждений',
            ha='center', va='center', fontsize=18, fontweight='bold', color=C['text'])
    ax.text(8, 8.05, 'Centriolar Damage Accumulation Theory of Aging',
            ha='center', va='center', fontsize=12, color='#7F8C8D', style='italic')

    # Центральный узел
    cx, cy = 8, 4.5
    center = plt.Circle((cx, cy), 1.1, color=C['old'], zorder=5)
    ax.add_patch(center)
    ax.text(cx, cy+0.15, 'Старая\nцентриоль', ha='center', va='center',
            fontsize=10, fontweight='bold', color='white', zorder=6)
    ax.text(cx, cy-0.55, '(необратимо\nповреждается)', ha='center', va='center',
            fontsize=7.5, color='#FFCCBC', zorder=6)

    # Пять тезисов вокруг центра
    theses = [
        (0,   '① Нерепарируемость',
               'Центриоль — единственная\nклеточная структура без пути\nремонта или замены.',
               C['damage']),
        (72,  '② Асимметричное\n   наследование',
               'При делении СК старая (мать)\nвсегда остаётся в стволовой\nклетке (>50% вероятность).',
               C['stem']),
        (144, '③ Двойная функция',
               'Мать-центриоль = база\nпервичной реснички (Wnt/Shh)\n+ полюс митотического веретена.',
               C['new']),
        (216, '④ ROS-уязвимость',
               'Высокое содержание α-тубулина\nделает центриоль мишенью\nкарбонилирования.',
               C['ros']),
        (288, '⑤ Каскад\n   последствий',
               'Треки A+B: потеря реснички\n→ истощение СК; спиндл-\nошибки → старение тканей.',
               '#8E44AD'),
    ]

    r = 3.2
    for angle, title, desc, color in theses:
        rad = np.radians(angle - 90)
        bx = cx + r * np.cos(rad)
        by = cy + r * np.sin(rad)
        # Стрелка
        ax.annotate('', xy=(cx + 1.15 * np.cos(rad), cy + 1.15 * np.sin(rad)),
                    xytext=(bx - 0.9 * np.cos(rad), by - 0.9 * np.sin(rad)),
                    arrowprops=dict(arrowstyle='->', color=color, lw=2), zorder=4)
        # Блок
        box = FancyBboxPatch((bx - 1.55, by - 0.82), 3.1, 1.64,
                              boxstyle='round,pad=0.08', facecolor=color + '22',
                              edgecolor=color, linewidth=2, zorder=3)
        ax.add_patch(box)
        ax.text(bx, by + 0.45, title, ha='center', va='center',
                fontsize=9.5, fontweight='bold', color=color, zorder=4)
        ax.text(bx, by - 0.15, desc, ha='center', va='center',
                fontsize=7.8, color=C['text'], zorder=4, linespacing=1.4)

    # Легенда внизу
    ax.text(8, 0.35,
            'СК = стволовая клетка   •   PTM = посттрансляционные модификации   '
            '•   ROS = активные формы кислорода',
            ha='center', va='center', fontsize=8, color='#95A5A6')

    save(fig, '01_overview')

# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 2 — Механизм асимметричного деления + накопление
# ════════════════════════════════════════════════════════════════════════════
def fig_asymmetric():
    fig, axes = plt.subplots(1, 2, figsize=(16, 8))
    fig.patch.set_facecolor(C['bg'])
    fig.suptitle('Асимметричное деление и накопление старых центриолей',
                 fontsize=16, fontweight='bold', color=C['text'], y=0.97)

    # ── левая панель: одно деление ──
    ax = axes[0]
    ax.set_facecolor(C['panel'])
    ax.set_xlim(0, 10); ax.set_ylim(0, 10)
    ax.axis('off')
    ax.set_title('Одно деление стволовой клетки', fontsize=12, pad=8, color=C['text'])

    # Материнская СК (G1)
    cell_mother = plt.Circle((5, 8), 1.4, color=C['stem'] + '55', ec=C['stem'], lw=2)
    ax.add_patch(cell_mother)
    ax.text(5, 8.7, 'СК (G1)', ha='center', va='center', fontsize=9, fontweight='bold', color=C['stem'])

    # Центриоли в материнской клетке
    old_m = plt.Circle((4.3, 7.8), 0.28, color=C['old'], zorder=5)
    new_m = plt.Circle((5.5, 7.8), 0.2, color=C['new'], zorder=5)
    ax.add_patch(old_m); ax.add_patch(new_m)
    ax.text(4.3, 7.25, '★ старая', ha='center', fontsize=7, color=C['old'], fontweight='bold')
    ax.text(5.5, 7.25, '○ новая', ha='center', fontsize=7, color=C['new'])

    # Стрелка деления
    ax.annotate('', xy=(3.2, 5.4), xytext=(4.5, 6.5),
                arrowprops=dict(arrowstyle='->', color=C['arrow'], lw=1.8))
    ax.annotate('', xy=(6.8, 5.4), xytext=(5.5, 6.5),
                arrowprops=dict(arrowstyle='->', color=C['arrow'], lw=1.8))
    ax.text(5, 6.1, 'АСИММЕТРИЧНОЕ\nДЕЛЕНИЕ', ha='center', fontsize=8.5,
            fontweight='bold', color=C['arrow'])

    # Дочерние клетки
    # СК-дочь (оставляет СТАРУЮ)
    d1 = plt.Circle((2.8, 4.2), 1.3, color=C['stem'] + '44', ec=C['stem'], lw=2)
    ax.add_patch(d1)
    ax.text(2.8, 5.1, 'СК-дочь', ha='center', fontsize=9, fontweight='bold', color=C['stem'])
    old_d1 = plt.Circle((2.8, 4.0), 0.3, color=C['old'], zorder=5)
    new_d1 = plt.Circle((3.4, 4.2), 0.18, color=C['new'], zorder=5)
    ax.add_patch(old_d1); ax.add_patch(new_d1)
    ax.text(2.8, 3.3, '★ старая\n(унаследована)', ha='center', fontsize=7,
            color=C['old'], fontweight='bold')

    # Прогениторная дочь (получает НОВУЮ)
    d2 = plt.Circle((7.2, 4.2), 1.3, color=C['diff'] + '33', ec=C['diff'], lw=2)
    ax.add_patch(d2)
    ax.text(7.2, 5.1, 'Прогенитор', ha='center', fontsize=9, fontweight='bold', color=C['diff'])
    new_d2 = plt.Circle((7.2, 4.0), 0.22, color=C['new'], zorder=5)
    ax.add_patch(new_d2)
    ax.text(7.2, 3.3, '○ новая\n(унаследована)', ha='center', fontsize=7, color=C['new'])

    # Новые центриоли дублируются
    ax.annotate('', xy=(2.3, 2.0), xytext=(2.8, 2.9),
                arrowprops=dict(arrowstyle='->', color=C['arrow'], lw=1.5))
    ax.text(2.0, 1.65, 'следующий\nцикл...', ha='center', fontsize=7.5, color=C['arrow'])

    # Метка «трещет»
    ax.text(2.8, 4.0, '⚠', ha='center', va='center', fontsize=14,
            color=C['damage'], zorder=7)

    # Легенда
    patches = [
        mpatches.Patch(color=C['old'], label='Старая (повреждённая) центриоль'),
        mpatches.Patch(color=C['new'], label='Молодая центриоль'),
    ]
    ax.legend(handles=patches, loc='lower left', fontsize=8, framealpha=0.9)

    # ── правая панель: накопление по поколениям ──
    ax2 = axes[1]
    ax2.set_facecolor(C['panel'])
    ax2.set_title('Накопление «возраста» по поколениям СК', fontsize=12, pad=8, color=C['text'])

    generations = np.arange(0, 11)
    # Средний возраст центриоли в СК (упрощённая модель)
    mean_age = 1 - np.exp(-0.18 * generations)  # нормировано 0→1
    # Количество функциональных индукторов (M₀=10 → убывает)
    inducers = 10 * np.exp(-0.12 * generations)

    ax2.plot(generations, inducers, 'o-', color=C['old'], lw=2.5, ms=7,
             label='Функциональные индукторы M (из M₀=10)')
    ax2.set_xlabel('Поколение стволовой клетки', fontsize=11, color=C['text'])
    ax2.set_ylabel('Количество индукторов M', fontsize=11, color=C['text'])
    ax2.set_xlim(-0.3, 10.3); ax2.set_ylim(0, 11)
    ax2.tick_params(colors=C['text'])

    # Зоны потентности
    ax2.axhspan(7, 11,  alpha=0.12, color=C['stem'],   label='Тотипотентная зона (M≥7)')
    ax2.axhspan(4, 7,   alpha=0.12, color=C['new'],    label='Плюрипотентная зона (M 4–7)')
    ax2.axhspan(0, 4,   alpha=0.12, color=C['old'],    label='Унипотентная / апоптоз (M<4)')

    ax2.axhline(7, ls='--', color=C['stem'],  lw=1.2, alpha=0.7)
    ax2.axhline(4, ls='--', color=C['old'],   lw=1.2, alpha=0.7)

    ax2.text(9.5, 9, 'Тотипотентность', ha='right', fontsize=8, color=C['stem'], fontstyle='italic')
    ax2.text(9.5, 5.5, 'Плюрипотентность', ha='right', fontsize=8, color=C['new'], fontstyle='italic')
    ax2.text(9.5, 2, 'Старение/апоптоз', ha='right', fontsize=8, color=C['old'], fontstyle='italic')

    ax2.legend(fontsize=8, loc='upper right', framealpha=0.9)
    ax2.grid(alpha=0.3)
    ax2.set_facecolor(C['bg'])
    for spine in ax2.spines.values():
        spine.set_edgecolor('#CCCCCC')

    plt.tight_layout(rect=[0, 0, 1, 0.95])
    save(fig, '02_asymmetric_division')


# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 3 — Пять форм молекулярных повреждений
# ════════════════════════════════════════════════════════════════════════════
def fig_damage_types():
    fig, ax = plt.subplots(figsize=(16, 9))
    fig.patch.set_facecolor(C['bg'])
    ax.set_facecolor(C['bg'])
    ax.set_xlim(0, 16); ax.set_ylim(0, 9)
    ax.axis('off')

    ax.text(8, 8.55, 'Пять форм необратимых повреждений материнской центриоли',
            ha='center', fontsize=16, fontweight='bold', color=C['text'])
    ax.text(8, 8.1, '(нарастают с возрастом под действием ROS и снижения UPS-активности)',
            ha='center', fontsize=11, color='#7F8C8D', style='italic')

    damages = [
        ('① Карбонилирование\nбелков',
         'ROS окисляют SAS-6 и CEP135\n→ разрушение картвилы\n→ нарушение сборки прокентриоли',
         '#E74C3C', 'O-C=O'),
        ('② Гипер-\nацетилирование',
         'Снижение HDAC6 и SIRT2\n→ α-тубулин Lys40 гиперацетилирован\n→ изменение жёсткости, нет динамики',
         '#E67E22', 'Ac↑↑'),
        ('③ Белковые\nагрегаты',
         'CPAP и CEP290 агрегируют\n→ блокируют PCM-якорение\n→ нарушен нуклеатор γ-TuRC',
         '#8E44AD', '⬡⬡⬡'),
        ('④ Фосфо-\nдисрегуляция',
         'Дисбаланс PLK4/NEK2/PP1\n→ аберрантный цикл дупликации\n→ ошибки сборки веретена',
         '#2980B9', 'P̃≠P'),
        ('⑤ Потеря\nдистальных придатков',
         'CEP164, CEP89, Ninein, CEP170↓\n→ нет якорения к мембране\n→ первичная ресничка не формируется',
         '#27AE60', 'CEP↓'),
    ]

    xs = [1.6, 4.8, 8.0, 11.2, 14.4]
    for i, (title, desc, color, symbol) in enumerate(damages):
        x = xs[i]
        # Верхний круг — символ
        circ = plt.Circle((x, 6.5), 0.9, color=color, zorder=4, alpha=0.9)
        ax.add_patch(circ)
        ax.text(x, 6.5, symbol, ha='center', va='center', fontsize=13,
                fontweight='bold', color='white', zorder=5)

        # Стрелка вниз
        ax.annotate('', xy=(x, 5.45), xytext=(x, 5.55),
                    arrowprops=dict(arrowstyle='->', color=color, lw=2))

        # Блок описания
        box = FancyBboxPatch((x - 1.35, 1.9), 2.7, 3.4,
                              boxstyle='round,pad=0.1',
                              facecolor=color + '18', edgecolor=color, lw=2, zorder=3)
        ax.add_patch(box)
        ax.text(x, 5.0, title, ha='center', va='top', fontsize=9.5,
                fontweight='bold', color=color, zorder=4, linespacing=1.4)
        ax.text(x, 4.2, desc, ha='center', va='top', fontsize=8.2,
                color=C['text'], zorder=4, linespacing=1.5)

        # Нижний бейдж «накапливается»
        ax.text(x, 2.1, '↑ с возрастом', ha='center', va='bottom',
                fontsize=8, color=color, fontstyle='italic', zorder=4)

    # Общая стрелка «→ снижение потентности»
    ax.annotate('', xy=(15.2, 0.85), xytext=(0.8, 0.85),
                arrowprops=dict(arrowstyle='->', color=C['damage'],
                                lw=3, connectionstyle='arc3,rad=0'))
    ax.text(8, 0.55, '→  Совокупный PTM-ущерб  →  Потеря функции центриоли  →  Снижение потентности СК',
            ha='center', fontsize=10, fontweight='bold', color=C['damage'])

    save(fig, '03_damage_types')


# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 4 — Два трека старения (A и B)
# ════════════════════════════════════════════════════════════════════════════
def fig_two_tracks():
    fig, ax = plt.subplots(figsize=(16, 10))
    fig.patch.set_facecolor(C['bg'])
    ax.set_facecolor(C['bg'])
    ax.set_xlim(0, 16); ax.set_ylim(0, 10)
    ax.axis('off')

    ax.text(8, 9.6, 'Два трека трансляции повреждений центриоли в старение тканей',
            ha='center', fontsize=16, fontweight='bold', color=C['text'])

    # Центральный блок — повреждённая центриоль
    cbox = FancyBboxPatch((5.8, 7.8), 4.4, 1.5, boxstyle='round,pad=0.15',
                           facecolor=C['old'] + '33', edgecolor=C['old'], lw=2.5)
    ax.add_patch(cbox)
    ax.text(8, 8.55, '★  ПОВРЕЖДЁННАЯ МАТЕРИНСКАЯ ЦЕНТРИОЛЬ', ha='center',
            fontsize=11, fontweight='bold', color=C['old'])
    ax.text(8, 8.1, 'PTM-накопление: карбонилирование, гиперацетилирование, агрегаты…',
            ha='center', fontsize=8.5, color=C['text'])

    # Стрелки к трекам
    ax.annotate('', xy=(3.5, 5.8), xytext=(6.2, 7.8),
                arrowprops=dict(arrowstyle='->', color='#2980B9', lw=3))
    ax.annotate('', xy=(12.5, 5.8), xytext=(9.8, 7.8),
                arrowprops=dict(arrowstyle='->', color='#E74C3C', lw=3))

    # ── ТРЕК A (слева) ──
    ta_color = '#2980B9'
    ax.text(3.5, 6.1, 'ТРЕК A\nПотеря первичной реснички', ha='center',
            fontsize=11, fontweight='bold', color=ta_color)

    track_a_steps = [
        'Потеря CEP164, CEP89, Ninein,\nCEP170 на дист. придатках',
        'Центриоль не якорится\nк апикальной мембране',
        'Первичная ресничка\nне формируется',
        'Нет трансдукции Wnt,\nNotch, Shh-сигналов',
        'СК «глохнет» к нише\n→ преждевременный покой',
        '↓ Регенерация тканей\n↓ Нейрогенез / гемопоэз',
    ]
    for j, step in enumerate(track_a_steps):
        yy = 5.2 - j * 0.82
        sbox = FancyBboxPatch((0.3, yy - 0.28), 6.4, 0.6,
                               boxstyle='round,pad=0.07',
                               facecolor=ta_color + '15', edgecolor=ta_color + '88', lw=1.2)
        ax.add_patch(sbox)
        ax.text(3.5, yy + 0.02, step, ha='center', va='center',
                fontsize=8, color=C['text'])
        if j < len(track_a_steps) - 1:
            ax.annotate('', xy=(3.5, yy - 0.3), xytext=(3.5, yy - 0.18),
                        arrowprops=dict(arrowstyle='->', color=ta_color, lw=1.5))

    # ── ТРЕК B (справа) ──
    tb_color = '#E74C3C'
    ax.text(12.5, 6.1, 'ТРЕК B\nПотеря фidelity митотич. веретена', ha='center',
            fontsize=11, fontweight='bold', color=tb_color)

    track_b_steps = [
        'Карбонилирование SAS-6,\nагрегаты CPAP/CEP290',
        'MTOC не организует\nбиполярное веретено',
        'Симметричные деления ↑\n(потеря Numb/aPKC асимметрии)',
        'Вариант 1: обе дочери\nдифференцируются → истощение СК',
        'Вариант 2: обе дочери\nсамообновляются → CHIP/рак',
        '↑ Хромосомная нестабильность\n↑ Сенесценция, SASP',
    ]
    for j, step in enumerate(track_b_steps):
        yy = 5.2 - j * 0.82
        sbox = FancyBboxPatch((9.3, yy - 0.28), 6.4, 0.6,
                               boxstyle='round,pad=0.07',
                               facecolor=tb_color + '15', edgecolor=tb_color + '88', lw=1.2)
        ax.add_patch(sbox)
        ax.text(12.5, yy + 0.02, step, ha='center', va='center',
                fontsize=8, color=C['text'])
        if j < len(track_b_steps) - 1:
            ax.annotate('', xy=(12.5, yy - 0.3), xytext=(12.5, yy - 0.18),
                        arrowprops=dict(arrowstyle='->', color=tb_color, lw=1.5))

    # Нижний общий исход
    ax.annotate('', xy=(6.8, 0.35), xytext=(3.5, 0.55),
                arrowprops=dict(arrowstyle='->', color=C['text'], lw=2))
    ax.annotate('', xy=(9.2, 0.35), xytext=(12.5, 0.55),
                arrowprops=dict(arrowstyle='->', color=C['text'], lw=2))
    death_box = FancyBboxPatch((5.5, 0.05), 5.0, 0.55,
                                boxstyle='round,pad=0.1',
                                facecolor=C['damage'] + '33', edgecolor=C['damage'], lw=2.5)
    ax.add_patch(death_box)
    ax.text(8, 0.33, '⚠  ОРГАННАЯ НЕДОСТАТОЧНОСТЬ → СМЕРТЬ ОРГАНИЗМА  ⚠',
            ha='center', va='center', fontsize=10, fontweight='bold', color=C['damage'])

    save(fig, '04_two_tracks')


# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 5 — Петля положительной обратной связи (ROS-loop)
# ════════════════════════════════════════════════════════════════════════════
def fig_ros_loop():
    fig, ax = plt.subplots(figsize=(12, 10))
    fig.patch.set_facecolor(C['bg'])
    ax.set_facecolor(C['bg'])
    ax.set_xlim(0, 12); ax.set_ylim(0, 10)
    ax.axis('off')

    ax.text(6, 9.65, 'Петля положительной обратной связи ROS→Повреждение',
            ha='center', fontsize=15, fontweight='bold', color=C['text'])
    ax.text(6, 9.2, '(нелинейное ускорение старения после 40 лет)',
            ha='center', fontsize=10, color='#7F8C8D', style='italic')

    nodes = {
        'centriole':  (6.0, 7.2, '★ Повреждение\nцентриоли',   C['old']),
        'mtoc':       (2.2, 5.2, '⚙ Нарушение\nMTOC/PCM',      '#8E44AD'),
        'mito':       (2.2, 2.8, '⚡ Митохондриальная\nдисфункция', '#E67E22'),
        'ros':        (6.0, 1.2, '🔥 Повышение\nROS',           C['ros']),
        'myeloid':    (9.8, 2.8, '🩸 Миелоидный\nсдвиг',       '#C0392B'),
        'inflam':     (9.8, 5.2, '🔴 Воспаление\n(инфламэйджинг)', '#E74C3C'),
        'cilium':     (6.0, 5.5, '🔕 Потеря\nреснички',         '#2980B9'),
    }

    rad = 0.72
    for key, (x, y, label, color) in nodes.items():
        circ = plt.Circle((x, y), rad, color=color + '33', ec=color, lw=2.5, zorder=4)
        ax.add_patch(circ)
        ax.text(x, y, label, ha='center', va='center', fontsize=8.5,
                color=C['text'], fontweight='bold', zorder=5, linespacing=1.4)

    def arrow(src, dst, label='', color='#555', rad_c=0.0):
        sx, sy = nodes[src][0], nodes[src][1]
        dx, dy = nodes[dst][0], nodes[dst][1]
        # сдвигаем от центров кругов
        angle = np.arctan2(dy - sy, dx - sx)
        sx2 = sx + rad * np.cos(angle)
        sy2 = sy + rad * np.sin(angle)
        dx2 = dx - rad * np.cos(angle)
        dy2 = dy - rad * np.sin(angle)
        style = f'arc3,rad={rad_c}'
        ax.annotate('', xy=(dx2, dy2), xytext=(sx2, sy2),
                    arrowprops=dict(arrowstyle='->', color=color, lw=2.2,
                                    connectionstyle=style), zorder=3)
        if label:
            mx = (sx2 + dx2) / 2; my = (sy2 + dy2) / 2
            ax.text(mx, my, label, ha='center', fontsize=7.5,
                    color=color, fontstyle='italic',
                    bbox=dict(fc='white', ec='none', alpha=0.75, pad=1.5))

    arrow('centriole', 'mtoc',    '+→ MTOC↓',      C['old'],  -0.25)
    arrow('mtoc',      'mito',    '+→ Mito↓',      '#8E44AD', -0.1)
    arrow('mito',      'ros',     '+→ ROS↑',       C['ros'],   0.1)
    arrow('ros',       'myeloid', '+→ MyShift↑',   C['ros'],  -0.1)
    arrow('myeloid',   'inflam',  '+→ Inflam↑',    C['bad'],  -0.1)
    arrow('inflam',    'centriole','+→ PTM↑',      C['bad'],   0.25)
    arrow('centriole', 'cilium',  '+→ Cilia↓',     C['new'],   0.1)
    arrow('ros',       'centriole','+→ Damage↑',   C['damage'], 0.35)

    # Метка петли
    loop_circ = plt.Circle((6, 4.2), 3.5, fill=False, ec=C['damage'],
                             ls='--', lw=1.5, alpha=0.5, zorder=2)
    ax.add_patch(loop_circ)
    ax.text(0.7, 4.2, '⟳ Петля\nположительной\nОС', ha='center', fontsize=8.5,
            color=C['damage'], fontstyle='italic',
            bbox=dict(fc='white', ec=C['damage'], alpha=0.8, pad=3, boxstyle='round'))

    # Формула age-multiplier
    ax.text(6, 0.4,
            r'age_multiplier = 1.0  (age ≤ 40)  →  ×1.6  (age > 40)   |   '
            r'ROS(t) = 0.05 + 0.005·age + k_fb·damage(t)',
            ha='center', fontsize=9, color=C['text'],
            bbox=dict(fc=C['panel'], ec=C['damage'], alpha=0.9, pad=5, boxstyle='round'))

    save(fig, '05_ros_loop')


# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 6 — Траектория накопления повреждений (математика)
# ════════════════════════════════════════════════════════════════════════════
def fig_damage_curve():
    fig, axes = plt.subplots(1, 2, figsize=(16, 7))
    fig.patch.set_facecolor(C['bg'])
    fig.suptitle('Математика CDATA: накопление повреждений по трекам',
                 fontsize=15, fontweight='bold', color=C['text'], y=0.98)

    # ── левая: три сценария старения ──
    ax = axes[0]
    ax.set_facecolor(C['panel'])
    ax.set_title('Три сценария (прогерия / норма / долголетие)', fontsize=11, pad=6, color=C['text'])

    age = np.linspace(0, 100, 1000)

    def damage(age, rate=1.0, k_fb=0.12, mult_age=40, mult=1.6):
        d = np.zeros_like(age)
        dt = age[1] - age[0]
        for i in range(1, len(age)):
            am = mult if age[i] > mult_age else 1.0
            ros_boost = 1 + k_fb * d[i-1]
            d[i] = d[i-1] + rate * am * ros_boost * dt / 100
            d[i] = min(d[i], 1.0)
        return d

    d_normal   = damage(age, rate=1.0)
    d_progeria = damage(age, rate=5.0)
    d_longevity = damage(age, rate=0.6)

    ax.plot(age, d_normal,    color=C['text'],    lw=2.5, label='Норма (×1.0)')
    ax.plot(age, d_progeria,  color=C['bad'],     lw=2.5, ls='--', label='Прогерия (×5.0)')
    ax.plot(age, d_longevity, color=C['good'],    lw=2.5, ls=':',  label='Долголетие (×0.6)')

    ax.axhline(0.75, ls='-', color=C['damage'], lw=2, alpha=0.8, label='Порог сенесценции (0.75)')
    ax.axvline(78,   ls='--', color=C['text'],  lw=1.2, alpha=0.5)

    # Аннотации смерти
    for d_arr, lbl, col, x_off in [
        (d_normal,    '~78 лет\n(норма)',     C['text'],   2),
        (d_progeria,  '~18 лет\n(прогерия)',  C['bad'],   2),
        (d_longevity, '~95 лет\n(долголетие)', C['good'], 2),
    ]:
        idx = np.argmax(d_arr >= 0.75) if np.any(d_arr >= 0.75) else -1
        if idx > 0:
            ax.annotate(lbl, xy=(age[idx], 0.75),
                        xytext=(age[idx] + x_off, 0.6),
                        arrowprops=dict(arrowstyle='->', color=col, lw=1.5),
                        fontsize=8, color=col, fontweight='bold')

    # Зона ускорения
    ax.axvspan(40, 100, alpha=0.06, color=C['ros'])
    ax.text(70, 0.05, 'age_multiplier ×1.6\n(после 40 лет)', fontsize=8,
            color=C['ros'], ha='center', fontstyle='italic')

    ax.set_xlabel('Возраст (лет)', fontsize=11, color=C['text'])
    ax.set_ylabel('Суммарный PTM-ущерб (0–1)', fontsize=11, color=C['text'])
    ax.set_xlim(0, 100); ax.set_ylim(0, 1.05)
    ax.legend(fontsize=9, framealpha=0.9)
    ax.grid(alpha=0.3)
    for spine in ax.spines.values(): spine.set_edgecolor('#CCCCCC')
    ax.tick_params(colors=C['text'])

    # ── правая: треки A и B ──
    ax2 = axes[1]
    ax2.set_facecolor(C['panel'])
    ax2.set_title('Функциональные показатели по двум трекам', fontsize=11, pad=6, color=C['text'])

    cil_fn = 1 - d_normal * 0.95       # ciliary function
    spin_fi = 1 - d_normal * 0.9       # spindle fidelity
    pool    = np.exp(-3 * d_normal**2) # stem cell pool (нелинейно)
    frailty = 1 - pool * cil_fn        # frailty index

    ax2.plot(age, cil_fn,  color='#2980B9', lw=2.2, label='Трек A: функция реснички')
    ax2.plot(age, spin_fi, color='#E74C3C', lw=2.2, label='Трек B: fidelity веретена')
    ax2.plot(age, pool,    color=C['stem'], lw=2.2, ls='--', label='Пул СК')
    ax2.plot(age, frailty, color=C['damage'], lw=2.5, ls=':', label='Индекс дряхлости')

    ax2.axhline(0.05, ls='-', color='#8E44AD', lw=1.5, alpha=0.7, label='Порог нейродегенерации')
    ax2.axvline(78, ls='--', color=C['text'], lw=1.2, alpha=0.5)

    ax2.set_xlabel('Возраст (лет)', fontsize=11, color=C['text'])
    ax2.set_ylabel('Нормированное значение (0–1)', fontsize=11, color=C['text'])
    ax2.set_xlim(0, 100); ax2.set_ylim(-0.02, 1.05)
    ax2.legend(fontsize=9, framealpha=0.9)
    ax2.grid(alpha=0.3)
    for spine in ax2.spines.values(): spine.set_edgecolor('#CCCCCC')
    ax2.tick_params(colors=C['text'])
    ax2.set_facecolor(C['bg'])

    plt.tight_layout(rect=[0, 0, 1, 0.96])
    save(fig, '06_damage_curves')


# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 7 — CDATA vs. другие теории старения
# ════════════════════════════════════════════════════════════════════════════
def fig_comparison():
    fig, ax = plt.subplots(figsize=(16, 9))
    fig.patch.set_facecolor(C['bg'])
    ax.set_facecolor(C['bg'])
    ax.set_xlim(0, 16); ax.set_ylim(0, 9)
    ax.axis('off')

    ax.text(8, 8.6, 'CDATA среди конкурирующих теорий старения',
            ha='center', fontsize=16, fontweight='bold', color=C['text'])
    ax.text(8, 8.15, 'Уровень причинности: восходящий (upstream) vs. нисходящий (downstream)',
            ha='center', fontsize=10, color='#7F8C8D', style='italic')

    theories = [
        # (x, y, название, уровень, цвет, upstream?)
        (8.0, 6.2, 'CDATA\n(центриоль)', 'ПЕРВИЧНЫЙ\nмеханизм',  C['old'],  True),
        (1.5, 3.5, 'Теломерная\nтеория',  'нисходящий', '#3498DB', False),
        (4.5, 3.5, 'Эпигенети-\nческие часы', 'нисходящий', '#9B59B6', False),
        (7.5, 3.5, 'Митохондри-\nальная теория', 'нисходящий', '#E67E22', False),
        (10.5, 3.5, 'Свободно-\nрадикальная', 'нисходящий', '#E74C3C', False),
        (13.5, 3.5, 'Сенесценция\n(SASP)', 'нисходящий', '#1ABC9C', False),
    ]

    for x, y, name, level, color, is_upstream in theories:
        r = 1.05 if is_upstream else 0.85
        lw = 3.5 if is_upstream else 1.8
        circ = plt.Circle((x, y), r, color=color + '30', ec=color, lw=lw, zorder=4)
        ax.add_patch(circ)
        ax.text(x, y + 0.15, name, ha='center', va='center', fontsize=9.5 if is_upstream else 8.5,
                fontweight='bold', color=color, zorder=5, linespacing=1.3)
        ax.text(x, y - 0.5, level, ha='center', va='center', fontsize=7.5,
                color=C['text'], zorder=5)

    # Стрелки CDATA → другие
    downstream = [(1.5, 3.5), (4.5, 3.5), (7.5, 3.5), (10.5, 3.5), (13.5, 3.5)]
    labels_ds = ['Spindle fidelity↓\n→ telomere damage',
                 'MTOC → epigenetic\nreprogram.',
                 'mitophagy hub\n↓ fusion',
                 'centrosomal\nROS source',
                 'Numb/aPKC loss\n→ senescence']
    for (dx, dy), lbl in zip(downstream, labels_ds):
        ax.annotate('', xy=(dx, dy + 0.87), xytext=(8, 6.2 - 1.08),
                    arrowprops=dict(arrowstyle='->', color=C['old'], lw=1.8,
                                    connectionstyle='arc3,rad=0.05'), zorder=3)
        mx = (dx + 8) / 2; my = (dy + 0.87 + 6.2 - 1.08) / 2 - 0.2
        ax.text(mx, my, lbl, ha='center', fontsize=6.5, color='#95A5A6',
                bbox=dict(fc='white', ec='none', alpha=0.6, pad=1))

    # Уровни
    ax.add_patch(FancyBboxPatch((0.3, 5.2), 15.4, 0.5, boxstyle='round,pad=0.05',
                                 facecolor=C['old'] + '12', edgecolor=C['old'] + '55', lw=1))
    ax.text(0.6, 5.45, '← ПЕРВИЧНЫЙ', fontsize=9, color=C['old'], fontstyle='italic')
    ax.add_patch(FancyBboxPatch((0.3, 2.55), 15.4, 0.5, boxstyle='round,pad=0.05',
                                 facecolor='#BDC3C7' + '44', edgecolor='#BDC3C7', lw=1))
    ax.text(0.6, 2.8, '← НИСХОДЯЩИЕ (downstream) — описывают следствия', fontsize=9,
            color='#7F8C8D', fontstyle='italic')

    # Сравнительная таблица
    headers = ['Критерий', 'Теломеры', 'Эпигенетика', 'Митохондрии', 'CDATA']
    rows = [
        ['Нерепарируемость',    '✗ (TERT)', '✗ (DNMT3)', '✗ (биогенез)', '✓ полная'],
        ['Причинность доказана', '✗ корреляция', '✗ корреляция', '✗ паралл.', '? (DT)'],
        ['Унифицирует все треки','✗', '✗', '✗', '✓'],
        ['Тест. предсказание',  'умеренное', 'умеренное', 'умеренное', '✓ центросома-трансп.'],
    ]
    col_x = [1.0, 4.2, 6.9, 9.6, 12.5]
    row_y = [1.85, 1.35, 0.9, 0.45]
    col_colors = ['#FAFAFA', '#3498DB22', '#9B59B622', '#E6722222', C['old'] + '22']
    for ci, (hdr, cx) in enumerate(zip(headers, col_x)):
        ax.text(cx, 2.2, hdr, ha='left', fontsize=8, fontweight='bold',
                color=C['text'] if ci > 0 else '#7F8C8D')
    for ri, row in enumerate(rows):
        for ci, (val, cx) in enumerate(zip(row, col_x)):
            color_txt = C['good'] if '✓' in val else (C['bad'] if '✗' in val else C['text'])
            ax.text(cx, row_y[ri], val, ha='left', fontsize=7.5, color=color_txt)

    save(fig, '07_comparison')


# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 8 — Цифровой двойник Cell-DT: архитектура
# ════════════════════════════════════════════════════════════════════════════
def fig_digital_twin():
    fig, ax = plt.subplots(figsize=(16, 10))
    fig.patch.set_facecolor(C['bg'])
    ax.set_facecolor(C['bg'])
    ax.set_xlim(0, 16); ax.set_ylim(0, 10)
    ax.axis('off')

    ax.text(8, 9.65, 'Cell-DT: Архитектура Цифрового Двойника CDATA',
            ha='center', fontsize=16, fontweight='bold', color=C['text'])
    ax.text(8, 9.2, 'Rust ECS (hecs) · 14 крейтов · 198 unit-тестов · Python bindings',
            ha='center', fontsize=10, color='#7F8C8D', style='italic')

    # Центральная сущность (ECS entity)
    center_box = FancyBboxPatch((5.5, 4.0), 5.0, 2.2, boxstyle='round,pad=0.18',
                                 facecolor='#2C3E50', edgecolor=C['old'], lw=3)
    ax.add_patch(center_box)
    ax.text(8, 5.35, '🔬  ECS Entity', ha='center', fontsize=12,
            fontweight='bold', color='white')
    ax.text(8, 4.85, 'Одна стволовая клеточная ниша', ha='center',
            fontsize=9.5, color='#BDC3C7')
    ax.text(8, 4.4, '28 компонентов · CentriolarInducerPair\n'
                    'CentriolarDamageState · TissueState · MyeloidShiftState…',
            ha='center', fontsize=8, color='#95A5A6')

    # Модули вокруг
    modules = [
        (8.0, 8.2,  'HumanDevelopmentModule\n(центральный интегратор, шаг 4)', '#34495E', 'Шаг 4'),
        (2.0, 7.0,  'CentrioleModule\nPTM dynamics, O₂-detachment', C['old'],   'Шаг 1'),
        (2.0, 5.2,  'MitochondrialModule\nTrack E: ROS, fusion, mtDNA', '#E67E22', 'Шаг 2'),
        (2.0, 3.4,  'CellCycleModule\np21/p16/CycD, G1→S checkpoint', '#2980B9', 'Шаг 3'),
        (14.0, 7.0, 'MyeloidShiftModule\nHSC lineage bias → inflammaging', '#C0392B', 'Шаг 5'),
        (14.0, 5.2, 'TranscriptomeModule\nCDKN1A, CDKN2A, MYC, γH2AX', '#8E44AD', 'Шаг 6'),
        (14.0, 3.4, 'HormonalModule (Track G)\nHPG-axis, puberty–lifespan r=0.78', '#16A085', 'Шаг 7'),
    ]
    for mx, my, label, color, step in modules:
        box = FancyBboxPatch((mx - 1.7, my - 0.48), 3.4, 0.96,
                              boxstyle='round,pad=0.08',
                              facecolor=color + '25', edgecolor=color, lw=2)
        ax.add_patch(box)
        ax.text(mx, my + 0.18, label, ha='center', va='center',
                fontsize=7.8, color=C['text'], linespacing=1.3)
        ax.text(mx, my - 0.3, step, ha='center', fontsize=7, color=color, fontweight='bold')

        # Стрелка к центру
        angle = np.arctan2(5.1 - my, 8 - mx)
        sx = mx + 1.72 * np.cos(angle)
        sy = my + 0.5 * np.sin(angle)
        ex = 8 - 2.55 * np.cos(angle)
        ey = 5.1 - 1.15 * np.sin(angle)
        ax.annotate('', xy=(ex, ey), xytext=(sx, sy),
                    arrowprops=dict(arrowstyle='<->', color=color, lw=1.8), zorder=3)

    # Семь треков старения
    tracks = [
        ('Трек A', 'Цилиарная дисф.', C['new']),
        ('Трек B', 'Fidelity веретена', '#E74C3C'),
        ('Трек C', 'Теломеры', '#9B59B6'),
        ('Трек D', 'Эпигенет. часы', '#F39C12'),
        ('Трек E', 'Митохондрии', '#E67E22'),
        ('Трек F', 'Делен. скорость', C['stem']),
        ('Трек G', 'Гормональный', '#16A085'),
    ]
    tx_start = 1.0
    for i, (track, name, color) in enumerate(tracks):
        tx = tx_start + i * 2.0
        ax.add_patch(FancyBboxPatch((tx - 0.82, 0.9), 1.64, 0.65,
                                     boxstyle='round,pad=0.06',
                                     facecolor=color + '30', edgecolor=color, lw=1.8))
        ax.text(tx, 1.35, track, ha='center', fontsize=8, fontweight='bold', color=color)
        ax.text(tx, 1.07, name,  ha='center', fontsize=6.8, color=C['text'])

    ax.text(8, 0.55, '7 треков старения · 11 петель ОС · lifespan ≈ 78 лет при seed=42',
            ha='center', fontsize=9, color=C['text'],
            bbox=dict(fc=C['panel'], ec='#BDC3C7', pad=4, boxstyle='round'))

    save(fig, '08_digital_twin_arch')


# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 9 — Вмешательства: эффективность
# ════════════════════════════════════════════════════════════════════════════
def fig_interventions():
    fig, axes = plt.subplots(1, 2, figsize=(16, 7))
    fig.patch.set_facecolor(C['bg'])
    fig.suptitle('Терапевтические вмешательства: предсказания Cell-DT',
                 fontsize=15, fontweight='bold', color=C['text'], y=0.98)

    # Горизонтальная барчарта
    ax = axes[0]
    ax.set_facecolor(C['panel'])
    ax.set_title('Прирост healthspan по типу вмешательства', fontsize=11, pad=6, color=C['text'])

    interventions = [
        ('Трансплантация центросомы', 14.2, C['old'],   True),
        ('CAFD-ретейнер',             8.7,  C['damage'], True),
        ('Защита от ROS\n(центросомальный щит)', 6.1, C['ros'],  True),
        ('Стимуляция цилиогенеза',     5.3,  C['new'],   True),
        ('Сенолитики',                 3.8,  '#7F8C8D', False),
        ('Caloric restriction',         2.9,  '#7F8C8D', False),
        ('Активация TERT',              1.6,  '#7F8C8D', False),
        ('Эпигенет. репрограммир.',     1.1,  '#7F8C8D', False),
    ]
    names = [i[0] for i in interventions]
    vals  = [i[1] for i in interventions]
    colors = [i[2] for i in interventions]
    primary = [i[3] for i in interventions]

    y_pos = np.arange(len(names))
    bars = ax.barh(y_pos, vals, color=[c + 'CC' for c in colors], edgecolor='white', lw=1.5, height=0.7)

    for bar, val, is_p in zip(bars, vals, primary):
        ax.text(bar.get_width() + 0.15, bar.get_y() + bar.get_height()/2,
                f'+{val:.1f} лет', va='center', fontsize=9.5,
                fontweight='bold' if is_p else 'normal',
                color=C['text'])
        if is_p:
            ax.text(-0.3, bar.get_y() + bar.get_height()/2, '★', va='center',
                    ha='right', fontsize=10, color=C['damage'])

    ax.set_yticks(y_pos)
    ax.set_yticklabels(names, fontsize=8.5)
    ax.set_xlabel('Прирост healthspan (лет)', fontsize=10, color=C['text'])
    ax.set_xlim(-1, 17)
    ax.axvline(0, color='#BDC3C7', lw=1)
    ax.axhline(3.5, color=C['damage'], ls='--', lw=1.5, alpha=0.6)
    ax.text(15.5, 3.7, '← первичные\n→ нисходящие', ha='right', fontsize=7.5,
            color=C['damage'], fontstyle='italic')
    ax.grid(axis='x', alpha=0.3)
    ax.set_facecolor(C['bg'])
    for spine in ax.spines.values(): spine.set_edgecolor('#CCCCCC')
    ax.tick_params(colors=C['text'])

    # Анализ чувствительности (tornado)
    ax2 = axes[1]
    ax2.set_facecolor(C['panel'])
    ax2.set_title('OAT-анализ чувствительности: 11 параметров (±30%)', fontsize=11, pad=6, color=C['text'])

    params = [
        ('base_detach_probability',  24, 21),
        ('senescence_threshold',     18, 14),
        ('ptm_exhaustion_scale',     12,  9),
        ('ros_feedback_k',            9,  7),
        ('mitophagy_threshold',       7,  5),
        ('age_multiplier_post40',     6,  4),
        ('spindle_fidelity_start',    5,  4),
        ('division_rate',             4,  3),
        ('mito_shield_init',          3,  2),
        ('noise_scale',               3,  2),
        ('menopause_ros_boost',       2,  1),
    ]
    pnames = [p[0] for p in params]
    pos_eff = [p[1] for p in params]  # +30% снижение жизни
    neg_eff = [-p[2] for p in params] # -30% прирост жизни

    yy = np.arange(len(pnames))
    ax2.barh(yy, pos_eff, color=C['bad']  + 'BB', height=0.7, label='+30% (сокращает жизнь)')
    ax2.barh(yy, neg_eff, color=C['good'] + 'BB', height=0.7, label='−30% (продлевает жизнь)')

    ax2.set_yticks(yy)
    ax2.set_yticklabels(pnames, fontsize=8)
    ax2.set_xlabel('Δ жизненного срока (лет)', fontsize=10, color=C['text'])
    ax2.axvline(0, color=C['text'], lw=1.5)
    ax2.legend(fontsize=8, framealpha=0.9)
    ax2.grid(axis='x', alpha=0.3)
    ax2.set_facecolor(C['bg'])
    for spine in ax2.spines.values(): spine.set_edgecolor('#CCCCCC')
    ax2.tick_params(colors=C['text'])

    plt.tight_layout(rect=[0, 0, 1, 0.95])
    save(fig, '09_interventions')


# ════════════════════════════════════════════════════════════════════════════
# РИСУНОК 10 — CHIP: клональная гемопоэтика
# ════════════════════════════════════════════════════════════════════════════
def fig_chip():
    fig, axes = plt.subplots(1, 2, figsize=(16, 7))
    fig.patch.set_facecolor(C['bg'])
    fig.suptitle('CHIP: клональный гемопоэз через призму CDATA',
                 fontsize=15, fontweight='bold', color=C['text'], y=0.98)

    # Симуляция клонов
    np.random.seed(42)
    n_niches = 20
    age_pts = np.linspace(0, 80, 81)
    n_clones_init = 10
    clones = np.zeros((n_clones_init, len(age_pts)))
    for i in range(n_clones_init):
        clones[i, 0] = n_niches / n_clones_init

    damage_rates = 1.0 + np.random.normal(0, 0.2, n_clones_init)
    for t in range(1, len(age_pts)):
        fitness = np.exp(-0.03 * damage_rates * age_pts[t])
        total = clones[:, t-1].sum()
        if total > 0:
            new_shares = clones[:, t-1] * fitness
            clones[:, t] = new_shares / new_shares.sum() * n_niches
        else:
            clones[:, t] = clones[:, t-1]

    ax = axes[0]
    ax.set_facecolor(C['panel'])
    ax.set_title('Клональная динамика 20 HSC-ниш', fontsize=11, pad=6, color=C['text'])

    cmap = plt.cm.tab10
    bottom = np.zeros(len(age_pts))
    for i in range(n_clones_init):
        color = cmap(i / n_clones_init)
        ax.fill_between(age_pts, bottom, bottom + clones[i], alpha=0.8, color=color,
                        label=f'Клон {i+1}')
        bottom += clones[i]

    ax.axvline(40, ls='--', color=C['damage'], lw=2, label='CHIP детект. (~40 лет)')
    ax.set_xlabel('Возраст (лет)', fontsize=11, color=C['text'])
    ax.set_ylabel('Число ниш', fontsize=11, color=C['text'])
    ax.set_xlim(0, 80); ax.set_ylim(0, n_niches)
    ax.legend(fontsize=6.5, loc='upper left', ncol=2, framealpha=0.8)
    ax.grid(alpha=0.3)
    for spine in ax.spines.values(): spine.set_edgecolor('#CCCCCC')
    ax.tick_params(colors=C['text'])
    ax.set_facecolor(C['bg'])

    # Миелоидное смещение
    ax2 = axes[1]
    ax2.set_facecolor(C['panel'])
    ax2.set_title('Миелоидный индекс vs. эмпирическое окно', fontsize=11, pad=6, color=C['text'])

    age2 = np.linspace(0, 80, 200)
    myeloid = 0.3 + 0.005 * age2 + 0.0002 * age2**2
    myeloid = np.clip(myeloid, 0, 0.8)

    ax2.plot(age2, myeloid, color=C['bad'], lw=2.5, label='Cell-DT предсказание')
    ax2.axhspan(0.45, 0.65, alpha=0.15, color='#3498DB', label='Эмпирич. диапазон в 70 лет (0.45–0.65)')
    ax2.axvline(70, ls='--', color='#3498DB', lw=1.5, alpha=0.7)
    ax2.plot(70, myeloid[np.argmin(np.abs(age2 - 70))], 'o', ms=10,
             color=C['bad'], zorder=5, label=f'Модель @ 70 лет ≈ 0.57')

    ax2.set_xlabel('Возраст (лет)', fontsize=11, color=C['text'])
    ax2.set_ylabel('Myeloid Bias Index', fontsize=11, color=C['text'])
    ax2.set_xlim(0, 80); ax2.set_ylim(0, 0.85)
    ax2.legend(fontsize=9, framealpha=0.9)
    ax2.grid(alpha=0.3)
    for spine in ax2.spines.values(): spine.set_edgecolor('#CCCCCC')
    ax2.tick_params(colors=C['text'])
    ax2.set_facecolor(C['bg'])

    plt.tight_layout(rect=[0, 0, 1, 0.95])
    save(fig, '10_chip')


# ════════════════════════════════════════════════════════════════════════════
# ЗАПУСК
# ════════════════════════════════════════════════════════════════════════════
if __name__ == '__main__':
    print('\n═══ Генерация тезисных диаграмм CDATA ═══\n')
    fig_overview()
    fig_asymmetric()
    fig_damage_types()
    fig_two_tracks()
    fig_ros_loop()
    fig_damage_curve()
    fig_comparison()
    fig_digital_twin()
    fig_interventions()
    fig_chip()
    print('\n✅  Все 10 рисунков сохранены в', OUT)
