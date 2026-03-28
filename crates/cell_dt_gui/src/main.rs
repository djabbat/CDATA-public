/// CDATA v3.0 — Desktop GUI (eframe / egui)
///
/// Panels:
///   Left   — preset selector + intervention checkboxes + strength slider
///   Center — 3×3 grid of plots (Damage, StemPool, ROS, SASP, Frailty,
///             Telomere, EpiAge, NK, Fibrosis) over 0–100 years
///
/// Two simulation runs are shown in each plot:
///   Gray  — Baseline (no interventions, selected preset)
///   Green — With active interventions

use eframe::egui;
use egui::Color32;
use egui_plot::{Line, Plot, PlotPoints};
use cell_dt_aging_engine::{
    AgingEngine, AgeSnapshot, InterventionSet, SimulationConfig, SimulationPreset,
};

// ── App state ─────────────────────────────────────────────────────────────────

struct CdataApp {
    preset:    SimulationPreset,
    ivs:       InterventionSet,
    baseline:  Vec<AgeSnapshot>,
    with_ivs:  Vec<AgeSnapshot>,
    dirty:     bool,
}

impl CdataApp {
    fn new() -> Self {
        let mut app = Self {
            preset:   SimulationPreset::Normal,
            ivs:      InterventionSet::default(),
            baseline: Vec::new(),
            with_ivs: Vec::new(),
            dirty:    true,
        };
        app.recompute();
        app
    }

    fn recompute(&mut self) {
        let base_cfg = SimulationConfig {
            preset: self.preset.clone(),
            interventions: InterventionSet::default(),
            ..Default::default()
        };
        self.baseline = AgingEngine::new(base_cfg).unwrap().run(1);

        let ivs_cfg = SimulationConfig {
            preset: self.preset.clone(),
            interventions: self.ivs.clone(),
            ..Default::default()
        };
        self.with_ivs = AgingEngine::new(ivs_cfg).unwrap().run(1);

        self.dirty = false;
    }
}

// ── Plot helpers ──────────────────────────────────────────────────────────────

fn make_points(snaps: &[AgeSnapshot], f: impl Fn(&AgeSnapshot) -> f64) -> PlotPoints {
    snaps.iter().map(|s| [s.age_years, f(s)]).collect()
}

fn subplot(
    ui: &mut egui::Ui,
    label: &str,
    baseline: &[AgeSnapshot],
    with_ivs: &[AgeSnapshot],
    extractor: impl Fn(&AgeSnapshot) -> f64,
    y_max: f64,
) {
    let show_ivs = with_ivs.iter().zip(baseline.iter())
        .any(|(a, b)| (extractor(a) - extractor(b)).abs() > 1e-6);

    Plot::new(label)
        .height(140.0)
        .include_y(0.0)
        .include_y(y_max)
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(make_points(baseline, &extractor))
                    .name("Baseline")
                    .color(Color32::from_rgb(120, 150, 200))
                    .width(2.0),
            );
            if show_ivs {
                plot_ui.line(
                    Line::new(make_points(with_ivs, &extractor))
                        .name("Interventions")
                        .color(Color32::from_rgb(80, 200, 100))
                        .width(2.0),
                );
            }
        });
    ui.label(egui::RichText::new(label).small().color(Color32::GRAY));
}

// ── eframe::App ───────────────────────────────────────────────────────────────

impl eframe::App for CdataApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dirty { self.recompute(); }

        // ── Left panel: controls ──────────────────────────────────────────────
        egui::SidePanel::left("controls")
            .min_width(200.0)
            .max_width(220.0)
            .show(ctx, |ui| {
                ui.heading("CDATA v3.0");
                ui.separator();

                ui.label(egui::RichText::new("Preset").strong());
                for preset in [
                    SimulationPreset::Normal,
                    SimulationPreset::Progeria,
                    SimulationPreset::Longevity,
                    SimulationPreset::Isc,
                    SimulationPreset::Muscle,
                    SimulationPreset::Neural,
                ] {
                    let selected = self.preset == preset;
                    let btn = egui::Button::new(preset.label())
                        .fill(if selected {
                            Color32::from_rgb(60, 100, 160)
                        } else {
                            Color32::from_rgb(45, 45, 45)
                        });
                    if ui.add(btn).clicked() && !selected {
                        self.preset = preset;
                        self.dirty  = true;
                    }
                }

                ui.separator();
                ui.label(egui::RichText::new("Interventions").strong());

                macro_rules! chk {
                    ($field:ident, $label:literal) => {
                        if ui.checkbox(&mut self.ivs.$field, $label).changed() {
                            self.dirty = true;
                        }
                    };
                }
                chk!(caloric_restriction,      "Caloric Restriction");
                chk!(senolytics,               "Senolytics");
                chk!(antioxidants,             "Antioxidants");
                chk!(mtor_inhibition,          "mTOR Inhibition");
                chk!(telomerase,               "Telomerase");
                chk!(nk_boost,                 "NK Boost");
                chk!(stem_cell_therapy,        "Stem Cell Therapy");
                chk!(epigenetic_reprogramming, "Epigenetic Reprog.");

                ui.separator();
                ui.label(egui::RichText::new("Strength").strong());
                let slider = egui::Slider::new(&mut self.ivs.strength, 0.0..=1.0)
                    .text("effect")
                    .clamp_to_range(true);
                if ui.add(slider).changed() { self.dirty = true; }

                ui.separator();
                ui.add_space(4.0);
                let last = self.baseline.last();
                if let Some(s) = last {
                    ui.label(egui::RichText::new("At age 100:").small().strong());
                    ui.label(format!("Damage   {:.3}", s.centriole_damage));
                    ui.label(format!("Frailty  {:.3}", s.frailty_index));
                    ui.label(format!("Telomere {:.3}", s.telomere_length));
                    ui.label(format!("EpiAge   {:.1}", s.epigenetic_age));
                }
                ui.add_space(8.0);
                ui.label(egui::RichText::new("PMID: 36583780").small().weak());
                ui.label(egui::RichText::new("Tkemaladze J., 2023").small().weak());
            });

        // ── Central panel: 3×3 plots ──────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            let b = &self.baseline;
            let w = &self.with_ivs;

            egui::Grid::new("plots")
                .num_columns(3)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    subplot(ui, "Centriole Damage",  b, w, |s| s.centriole_damage, 1.0);
                    subplot(ui, "Stem Cell Pool",    b, w, |s| s.stem_cell_pool,   1.0);
                    subplot(ui, "ROS Level",         b, w, |s| s.ros_level,        1.0);
                    ui.end_row();

                    subplot(ui, "SASP Level",        b, w, |s| s.sasp_level,       1.0);
                    subplot(ui, "Frailty Index",     b, w, |s| s.frailty_index,    1.0);
                    subplot(ui, "Telomere Length",   b, w, |s| s.telomere_length,  1.0);
                    ui.end_row();

                    subplot(ui, "Epigenetic Age",    b, w, |s| s.epigenetic_age,  130.0);
                    subplot(ui, "NK Efficiency",     b, w, |s| s.nk_efficiency,    1.0);
                    subplot(ui, "Fibrosis Level",    b, w, |s| s.fibrosis_level,   1.0);
                    ui.end_row();
                });
        });
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CDATA v3.0 — Cell Digital Twin")
            .with_inner_size([1100.0, 750.0]),
        ..Default::default()
    };
    eframe::run_native(
        "CDATA v3.0",
        options,
        Box::new(|_cc| Box::new(CdataApp::new())),
    )
}
