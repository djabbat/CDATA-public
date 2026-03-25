//! Модель A: "Ландшафт старения" — 3D визуализация деградации тканей
//!
//! X-ось: возраст (0–120 лет)
//! Y-ось: тип ткани (8 типов)
//! Z-ось: score повреждения [0..1]
//! Цвет:  frailty (зелёный → жёлтый → красный)
//!
//! Управление:
//!   R — сброс камеры
//!   C — переключить режим цвета (damage / myeloid / spindle / frailty)
//!   A — запустить/остановить анимацию
//!   ESC — закрыть

use crate::CdataSnapshot;
use kiss3d::{
    camera::ArcBall,
    event::{Key, WindowEvent},
    light::Light,
    nalgebra::{Point2, Point3, Translation3},
    text::Font,
    window::Window,
};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

// ── Типы тканей (8 для Модели A) ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TissueKind {
    Blood,
    Neural,
    Epithelial,
    Muscle,
    Skin,
    Liver,
    Kidney,
    Lung,
}

impl TissueKind {
    pub const ALL: [TissueKind; 8] = [
        TissueKind::Blood,
        TissueKind::Neural,
        TissueKind::Epithelial,
        TissueKind::Muscle,
        TissueKind::Skin,
        TissueKind::Liver,
        TissueKind::Kidney,
        TissueKind::Lung,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            TissueKind::Blood     => "Blood/HSC",
            TissueKind::Neural    => "Neural",
            TissueKind::Epithelial => "Epithelial",
            TissueKind::Muscle    => "Muscle",
            TissueKind::Skin      => "Skin",
            TissueKind::Liver     => "Liver",
            TissueKind::Kidney    => "Kidney",
            TissueKind::Lung      => "Lung",
        }
    }

    /// Множитель скорости накопления повреждений (из CDATA-модели).
    /// Neural самый быстрый, Blood наиболее устойчив.
    pub fn damage_rate_multiplier(&self) -> f32 {
        match self {
            TissueKind::Neural     => 1.35,
            TissueKind::Skin       => 1.20,
            TissueKind::Kidney     => 1.15,
            TissueKind::Lung       => 1.10,
            TissueKind::Muscle     => 1.00,
            TissueKind::Epithelial => 0.95,
            TissueKind::Liver      => 0.90,
            TissueKind::Blood      => 0.80,
        }
    }

    /// Начало значимого накопления повреждений (годы).
    pub fn damage_onset_years(&self) -> f32 {
        match self {
            TissueKind::Neural     => 25.0,
            TissueKind::Muscle     => 30.0,
            TissueKind::Skin       => 25.0,
            TissueKind::Epithelial => 20.0,
            TissueKind::Kidney     => 35.0,
            TissueKind::Lung       => 30.0,
            TissueKind::Liver      => 40.0,
            TissueKind::Blood      => 40.0,
        }
    }
}

// ── Данные ландшафта ──────────────────────────────────────────────────────────

/// Одна точка ландшафта: возраст + метрики по всем 8 тканям.
#[derive(Debug, Clone)]
pub struct LandscapeFrame {
    pub age_years: f32,
    /// damage_score [0..1] per tissue (порядок = TissueKind::ALL)
    pub damage:  [f32; 8],
    /// frailty [0..1] per tissue
    pub frailty: [f32; 8],
    /// myeloid_bias [0..1] per tissue (аппроксимация)
    pub myeloid: [f32; 8],
    /// spindle_fidelity [0..1] per tissue
    pub spindle: [f32; 8],
}

/// Вся история ландшафта (вектор кадров по времени).
#[derive(Debug, Clone, Default)]
pub struct AgingLandscapeData {
    pub frames: Vec<LandscapeFrame>,
}

impl AgingLandscapeData {
    /// Сгенерировать синтетический ландшафт из CDATA-снимков.
    /// Глобальный снимок масштабируется тканевыми мультипликаторами.
    pub fn from_snapshots(snapshots: &[CdataSnapshot]) -> Self {
        let frames = snapshots
            .iter()
            .map(|snap| {
                let base_dmg     = snap.mean_damage_score;
                let base_frailty = snap.mean_frailty;
                let base_myeloid = snap.mean_myeloid_bias;
                let base_spindle = snap.mean_spindle_fidelity;
                let age          = snap.age_years as f32;

                let mut damage  = [0f32; 8];
                let mut frailty = [0f32; 8];
                let mut myeloid = [0f32; 8];
                let mut spindle = [0f32; 8];

                for (i, tissue) in TissueKind::ALL.iter().enumerate() {
                    let rate = tissue.damage_rate_multiplier();
                    let onset = tissue.damage_onset_years();
                    // До onset повреждений почти нет
                    let onset_factor = ((age - onset) / 20.0).clamp(0.0, 1.0);
                    damage[i]  = (base_dmg * rate * onset_factor).clamp(0.0, 1.0);
                    frailty[i] = (base_frailty * rate * onset_factor).clamp(0.0, 1.0);
                    myeloid[i] = (base_myeloid * rate).clamp(0.0, 1.0);
                    spindle[i] = 1.0 - (1.0 - base_spindle) * rate * onset_factor;
                    spindle[i] = spindle[i].clamp(0.0, 1.0);
                }

                LandscapeFrame { age_years: age, damage, frailty, myeloid, spindle }
            })
            .collect();

        AgingLandscapeData { frames }
    }

    /// Сгенерировать демонстрационный ландшафт (без симуляции).
    /// Использует CDATA-калибровку: смерть ≈ 78 лет, frailty растёт сигмоидально.
    pub fn demo() -> Self {
        let snapshots: Vec<CdataSnapshot> = (0..=120)
            .step_by(2)
            .map(|year| {
                let t = year as f32;
                // CDATA сигмоид старения (midlife_transition_center ≈ 40)
                let sigmoid = |center: f32, width: f32| -> f32 {
                    1.0 / (1.0 + (-((t - center) / width)).exp())
                };
                let damage  = (sigmoid(45.0, 15.0) * 0.85).clamp(0.0, 1.0);
                let frailty = (sigmoid(55.0, 12.0) * 0.90).clamp(0.0, 1.0);
                let myeloid = (sigmoid(50.0, 18.0) * 0.70).clamp(0.0, 1.0);
                let spindle = 1.0 - sigmoid(40.0, 15.0) * 0.75;

                CdataSnapshot {
                    step: year as u64 * 365,
                    age_years: t as f64,
                    mean_damage_score: damage,
                    mean_frailty: frailty,
                    mean_myeloid_bias: myeloid,
                    mean_spindle_fidelity: spindle,
                    alive_count: if t < 110.0 { 5 } else { (5.0 * (1.0 - (t - 110.0) / 10.0)).max(0.0) as usize },
                }
            })
            .collect();

        Self::from_snapshots(&snapshots)
    }
}

// ── Режим раскраски ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Damage,
    Frailty,
    MyeloidBias,
    SpindleFidelity,
}

impl ColorMode {
    fn next(self) -> Self {
        match self {
            ColorMode::Damage          => ColorMode::Frailty,
            ColorMode::Frailty         => ColorMode::MyeloidBias,
            ColorMode::MyeloidBias     => ColorMode::SpindleFidelity,
            ColorMode::SpindleFidelity => ColorMode::Damage,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            ColorMode::Damage          => "Damage Score",
            ColorMode::Frailty         => "Frailty",
            ColorMode::MyeloidBias     => "Myeloid Bias",
            ColorMode::SpindleFidelity => "Spindle Fidelity",
        }
    }
}

// ── Цветовая схема ────────────────────────────────────────────────────────────

/// value [0..1] → RGB
fn heat_color(value: f32, mode: ColorMode) -> (f32, f32, f32) {
    let v = value.clamp(0.0, 1.0);
    match mode {
        ColorMode::Damage | ColorMode::Frailty | ColorMode::MyeloidBias => {
            // зелёный → жёлтый → красный
            if v < 0.5 {
                let t = v * 2.0;
                (t, 1.0, 0.0)
            } else {
                let t = (v - 0.5) * 2.0;
                (1.0, 1.0 - t, 0.0)
            }
        }
        ColorMode::SpindleFidelity => {
            // красный (слабое) → белый → голубой (сильное)
            if v < 0.5 {
                let t = v * 2.0;
                (1.0, t, t)
            } else {
                let t = (v - 0.5) * 2.0;
                (1.0 - t, 1.0, 1.0)
            }
        }
    }
}

// ── Построение ячеек ланшафта ─────────────────────────────────────────────────

const GRID_X: f32 = 20.0; // ширина по оси возраста
const GRID_Y: f32 = 8.0;  // ширина по оси тканей
const HEIGHT_SCALE: f32 = 6.0; // масштаб высоты (damage → единицы сцены)

fn landscape_position(frame_idx: usize, n_frames: usize, tissue_idx: usize) -> (f32, f32) {
    let x = (frame_idx as f32 / (n_frames - 1).max(1) as f32) * GRID_X - GRID_X / 2.0;
    let y = (tissue_idx as f32 / 7.0) * GRID_Y - GRID_Y / 2.0;
    (x, y)
}

fn get_value(frame: &LandscapeFrame, tissue_idx: usize, mode: ColorMode) -> f32 {
    match mode {
        ColorMode::Damage          => frame.damage[tissue_idx],
        ColorMode::Frailty         => frame.frailty[tissue_idx],
        ColorMode::MyeloidBias     => frame.myeloid[tissue_idx],
        ColorMode::SpindleFidelity => frame.spindle[tissue_idx],
    }
}

// ── Shared state для обновления из GUI ────────────────────────────────────────

pub struct LandscapeShared {
    pub data: AgingLandscapeData,
    pub color_mode: ColorMode,
    pub animate: bool,
    pub anim_frame: usize,
    pub dirty: bool,
}

pub type SharedLandscape = Arc<Mutex<LandscapeShared>>;

// ── Главный визуализатор ──────────────────────────────────────────────────────

pub struct AgingLandscapeVisualizer {
    shared: SharedLandscape,
    _handle: Option<thread::JoinHandle<()>>,
}

impl AgingLandscapeVisualizer {
    /// Запустить с демо-данными.
    pub fn new_demo() -> Self {
        let data = AgingLandscapeData::demo();
        Self::new(data)
    }

    /// Запустить с реальными снимками симуляции.
    pub fn from_snapshots(snapshots: Vec<CdataSnapshot>) -> Self {
        let data = AgingLandscapeData::from_snapshots(&snapshots);
        Self::new(data)
    }

    fn new(data: AgingLandscapeData) -> Self {
        let shared = Arc::new(Mutex::new(LandscapeShared {
            data,
            color_mode: ColorMode::Damage,
            animate: false,
            anim_frame: 0,
            dirty: true,
        }));
        let shared_clone = Arc::clone(&shared);
        let handle = thread::spawn(move || run_landscape_window(shared_clone));
        Self { shared, _handle: Some(handle) }
    }

    /// Обновить данные из живой симуляции.
    pub fn update_snapshots(&self, snapshots: &[CdataSnapshot]) {
        if let Ok(mut s) = self.shared.lock() {
            s.data = AgingLandscapeData::from_snapshots(snapshots);
            s.dirty = true;
        }
    }

    pub fn shared(&self) -> SharedLandscape {
        Arc::clone(&self.shared)
    }
}

// ── Рендер-цикл ───────────────────────────────────────────────────────────────

fn run_landscape_window(shared: SharedLandscape) {
    let mut window = Window::new("CDATA — Aging Landscape 3D");
    window.set_background_color(0.05, 0.05, 0.12); // тёмно-синий фон
    window.set_light(Light::Absolute(Point3::new(20.0, 30.0, 20.0)));

    // Камера: смотрит сверху-сбоку на весь ландшафт
    let eye    = Point3::new(0.0, 20.0, 18.0);
    let target = Point3::new(0.0,  0.0,  2.0);
    let mut camera = ArcBall::new(eye, target);

    let mut nodes: Vec<kiss3d::scene::SceneNode> = Vec::new();
    let mut current_mode = ColorMode::Damage;
    let mut last_frame_count = 0;
    let mut anim_timer = 0u32;
    let font = Font::default();

    loop {
        if !window.render_with_camera(&mut camera) { break; }

        // ── Ввод ──────────────────────────────────────────────────────────
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(Key::Escape, _, _) => return,
                WindowEvent::Key(Key::R, _, _) => {
                    camera = ArcBall::new(eye, target);
                }
                WindowEvent::Key(Key::C, _, _) => {
                    if let Ok(mut s) = shared.lock() {
                        s.color_mode = s.color_mode.next();
                        s.dirty = true;
                    }
                }
                WindowEvent::Key(Key::A, _, _) => {
                    if let Ok(mut s) = shared.lock() {
                        s.animate = !s.animate;
                        if s.animate { s.anim_frame = 0; }
                    }
                }
                _ => {}
            }
        }

        // ── Анимация ──────────────────────────────────────────────────────
        anim_timer += 1;
        if anim_timer % 4 == 0 {
            if let Ok(mut s) = shared.lock() {
                if s.animate {
                    let n = s.data.frames.len();
                    if n > 0 {
                        s.anim_frame = (s.anim_frame + 1) % n;
                        s.dirty = true;
                    }
                }
            }
        }

        // ── Перестроить сцену при изменении данных ────────────────────────
        let (needs_rebuild, mode, anim_frame, frame_count) = {
            if let Ok(s) = shared.lock() {
                (s.dirty, s.color_mode, s.anim_frame, s.data.frames.len())
            } else {
                continue;
            }
        };

        if needs_rebuild || frame_count != last_frame_count || mode != current_mode {
            // Удалить старые ноды
            for node in nodes.iter_mut() {
                window.remove_node(node);
            }
            nodes.clear();

            if let Ok(mut s) = shared.lock() {
                build_landscape(&mut window, &mut nodes, &s.data, s.color_mode, s.anim_frame);
                s.dirty = false;
                current_mode = s.color_mode;
                last_frame_count = frame_count;
            }

            // Оси координат
            draw_axes(&mut window);
        }

        // ── HUD (текст) ───────────────────────────────────────────────────
        let mode_label = current_mode.label();
        window.draw_text(
            &format!("Color: {}  |  [C] change  [A] animate  [R] reset  [ESC] close", mode_label),
            &Point2::new(10.0, 10.0),
            60.0,
            &font,
            &Point3::new(0.9, 0.9, 0.9),
        );
    }
}

// ── Построение геометрии ──────────────────────────────────────────────────────

fn build_landscape(
    window: &mut Window,
    nodes: &mut Vec<kiss3d::scene::SceneNode>,
    data: &AgingLandscapeData,
    mode: ColorMode,
    anim_frame: usize,
) {
    let n_frames = data.frames.len();
    if n_frames == 0 { return; }

    let n_tissues = 8;
    let cell_w = GRID_X / n_frames.max(1) as f32 * 0.88;
    let cell_h = GRID_Y / n_tissues as f32 * 0.88;

    // Определяем диапазон кадров для отображения
    // В режиме анимации — только до anim_frame; в обычном — все
    let max_frame = if anim_frame > 0 { anim_frame + 1 } else { n_frames };

    for fi in 0..max_frame.min(n_frames) {
        let frame = &data.frames[fi];

        for ti in 0..n_tissues {
            let value = get_value(frame, ti, mode);
            if value < 0.001 { continue; } // не рендерить нулевые ячейки

            let (x, y_pos) = landscape_position(fi, n_frames, ti);
            let z = value * HEIGHT_SCALE;

            // Куб: ширина × высота (ось y в kiss3d = вверх) × глубина
            let mut node = window.add_cube(cell_w, z.max(0.05), cell_h);
            // Смещение: центр куба на половине высоты
            node.set_local_translation(Translation3::new(x, z / 2.0, y_pos));

            let (r, g, b) = heat_color(value, mode);

            // Подсветить последний (текущий) кадр анимации
            if fi == max_frame - 1 && anim_frame > 0 {
                let boost = 1.3f32;
                node.set_color((r * boost).min(1.0), (g * boost).min(1.0), (b * boost).min(1.0));
            } else {
                // Старые кадры чуть темнее
                let fade = 0.6 + 0.4 * (fi as f32 / n_frames as f32);
                node.set_color(r * fade, g * fade, b * fade);
            }

            nodes.push(node);
        }
    }

    // Базовая плоскость (сетка)
    let mut base = window.add_cube(GRID_X + 1.0, 0.05, GRID_Y + 1.0);
    base.set_local_translation(Translation3::new(0.0, -0.025, 0.0));
    base.set_color(0.15, 0.15, 0.25);
    nodes.push(base);
}

fn draw_axes(window: &mut Window) {
    // X ось (возраст) — белая
    window.draw_line(
        &Point3::new(-GRID_X / 2.0, 0.0, -GRID_Y / 2.0 - 0.5),
        &Point3::new( GRID_X / 2.0, 0.0, -GRID_Y / 2.0 - 0.5),
        &Point3::new(0.8, 0.8, 0.8),
    );
    // Z ось (высота) — красная
    window.draw_line(
        &Point3::new(-GRID_X / 2.0 - 0.5, 0.0, -GRID_Y / 2.0),
        &Point3::new(-GRID_X / 2.0 - 0.5, HEIGHT_SCALE, -GRID_Y / 2.0),
        &Point3::new(0.9, 0.2, 0.2),
    );
    // Y ось (ткани) — зелёная
    window.draw_line(
        &Point3::new(-GRID_X / 2.0 - 0.5, 0.0, -GRID_Y / 2.0),
        &Point3::new(-GRID_X / 2.0 - 0.5, 0.0,  GRID_Y / 2.0),
        &Point3::new(0.2, 0.9, 0.2),
    );
}
