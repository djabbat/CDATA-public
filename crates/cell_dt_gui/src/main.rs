/// CDATA v3.0 — Desktop GUI (eframe / egui)
///
/// Layout:
///   Left  (200px) — preset + interventions + age cursor
///   Center         — 3×3 plots (threshold lines, cursor VLine, baseline vs ivs)
///   Right (235px)  — values at cursor age, summary at 80, MCAI onset

use eframe::egui::{self, Color32, RichText};
use egui_plot::{HLine, Line, Plot, PlotPoints, VLine};
use cell_dt_aging_engine::{
    AgingEngine, AgeSnapshot, InterventionSet, SimulationConfig, SimulationPreset,
};

// ── Variable metadata ─────────────────────────────────────────────────────────

struct VarMeta {
    name:        &'static str,
    unit:        &'static str,
    description: &'static str,
    y_max:       f64,
    warn:        f64,
    crit:        f64,
    bad_is_high: bool,
}

const VARS: [VarMeta; 9] = [
    VarMeta { name:"Centriole Damage", unit:"index 0–1",
        description:"Core CDATA. Irreversible centriolar DNA damage.\nα=0.0082 · ν(t) · (1−Π(t))",
        y_max:1.0, warn:0.40, crit:0.70, bad_is_high:true },
    VarMeta { name:"Stem Cell Pool", unit:"fraction",
        description:"Residual regenerative capacity.\n= 1 − damage × 0.8. Below 0.3 → regen failure.",
        y_max:1.0, warn:0.50, crit:0.30, bad_is_high:false },
    VarMeta { name:"ROS Level", unit:"0–1",
        description:"Reactive oxygen species (sigmoid, mtDNA-driven).\nAmplifies centriole damage via oxidative PTMs.",
        y_max:1.0, warn:0.45, crit:0.70, bad_is_high:true },
    VarMeta { name:"SASP Level", unit:"0–1",
        description:"Senescence-Associated Secretory Phenotype.\nHormetic: low→stimulates repair; high→inhibits.",
        y_max:1.0, warn:0.35, crit:0.65, bad_is_high:true },
    VarMeta { name:"MCAI", unit:"0–1",
        description:"Model Composite Aging Index (v3.2.3).\nUnweighted mean: (D+SASP+(1−pool)+(1−telo)+VAF)/5.\nClinical threshold ≈ 0.25 (Fried 2001).",
        y_max:1.0, warn:0.25, crit:0.50, bad_is_high:true },
    VarMeta { name:"Telomere Length", unit:"fraction",
        description:"Normalized (1=full, 0=critically short).\nLoss: 0.012 × division_rate per year.\nMaster numbers 11/22 preserved.",
        y_max:1.0, warn:0.40, crit:0.20, bad_is_high:false },
    VarMeta { name:"Epigenetic Age", unit:"years",
        description:"Horvath/Hannum clock estimate.\nDrift: (chrono−epi)×0.1 + EPI_STRESS×damage + SASP.",
        y_max:130.0, warn:0.0, crit:0.0, bad_is_high:true },
    VarMeta { name:"NK Efficiency", unit:"0–1",
        description:"NK cell killing (1 − age×0.010), PMID 12803352.\n~70% decline by age 70. Clears senescent cells.",
        y_max:1.0, warn:0.40, crit:0.20, bad_is_high:false },
    VarMeta { name:"Fibrosis Level", unit:"0–1",
        description:"SASP-driven extracellular matrix replacement.\nReduces regen_factor by up to 40% (L3 link).",
        y_max:1.0, warn:0.25, crit:0.50, bad_is_high:true },
];

fn extract(s: &AgeSnapshot, i: usize) -> f64 {
    match i {
        0 => s.centriole_damage,
        1 => s.stem_cell_pool,
        2 => s.ros_level,
        3 => s.sasp_level,
        4 => s.mcai,
        5 => s.telomere_length,
        6 => s.epigenetic_age,
        7 => s.nk_efficiency,
        _ => s.fibrosis_level,
    }
}

fn val_color(val: f64, meta: &VarMeta) -> Color32 {
    if meta.y_max > 10.0 { return Color32::from_rgb(160, 190, 240); }
    if meta.bad_is_high {
        if val >= meta.crit  { Color32::from_rgb(235, 80, 60) }
        else if val >= meta.warn { Color32::from_rgb(225, 175, 35) }
        else { Color32::from_rgb(75, 195, 95) }
    } else {
        if val <= meta.crit  { Color32::from_rgb(235, 80, 60) }
        else if val <= meta.warn { Color32::from_rgb(225, 175, 35) }
        else { Color32::from_rgb(75, 195, 95) }
    }
}

fn pct(base: f64, new: f64) -> f64 {
    if base.abs() < 1e-9 { 0.0 } else { (new - base) / base * 100.0 }
}

fn nearest<'a>(snaps: &'a [AgeSnapshot], age: f64) -> Option<&'a AgeSnapshot> {
    snaps.iter().min_by(|a, b| {
        (a.age_years - age).abs().partial_cmp(&(b.age_years - age).abs()).unwrap()
    })
}

// ── App ───────────────────────────────────────────────────────────────────────

struct CdataApp {
    preset:     SimulationPreset,
    ivs:        InterventionSet,
    baseline:   Vec<AgeSnapshot>,
    with_ivs:   Vec<AgeSnapshot>,
    cursor_age: f64,
    dirty:      bool,
}

impl CdataApp {
    fn new() -> Self {
        let mut app = Self {
            preset: SimulationPreset::Normal,
            ivs:    InterventionSet::default(),
            baseline: Vec::new(),
            with_ivs: Vec::new(),
            cursor_age: 60.0,
            dirty: true,
        };
        app.recompute();
        app
    }

    fn recompute(&mut self) {
        let b_cfg = SimulationConfig {
            preset: self.preset.clone(),
            interventions: InterventionSet::default(),
            ..Default::default()
        };
        self.baseline = AgingEngine::new(b_cfg).unwrap().run(1);

        let i_cfg = SimulationConfig {
            preset: self.preset.clone(),
            interventions: self.ivs.clone(),
            ..Default::default()
        };
        self.with_ivs = AgingEngine::new(i_cfg).unwrap().run(1);
        self.dirty = false;
    }
}

impl eframe::App for CdataApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dirty { self.recompute(); }

        let ivs_active = self.ivs.any_active();
        let cursor     = self.cursor_age;

        // Clone slices once — avoids borrow conflicts with &mut self inside closures
        let baseline: Vec<AgeSnapshot>  = self.baseline.clone();
        let with_ivs: Vec<AgeSnapshot>  = if ivs_active { self.with_ivs.clone() } else { vec![] };

        // ── LEFT panel ────────────────────────────────────────────────────────
        egui::SidePanel::left("controls")
            .min_width(195.0).max_width(195.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.heading(RichText::new("CDATA v3.0").strong());
                    ui.label(RichText::new("Cell Digital Twin Simulator").small().weak());
                    ui.separator();

                    ui.label(RichText::new("▸ Preset").strong());
                    for preset in [
                        SimulationPreset::Normal,   SimulationPreset::Progeria,
                        SimulationPreset::Longevity, SimulationPreset::Isc,
                        SimulationPreset::Muscle,    SimulationPreset::Neural,
                    ] {
                        let sel  = self.preset == preset;
                        let fill = if sel { Color32::from_rgb(50, 90, 155) }
                                   else   { Color32::from_rgb(38, 38, 48) };
                        let lbl  = RichText::new(preset.label())
                            .color(if sel { Color32::WHITE } else { Color32::LIGHT_GRAY });
                        if ui.add(egui::Button::new(lbl).fill(fill)
                            .min_size([185.0, 22.0].into())).clicked() && !sel
                        {
                            self.preset = preset;
                            self.dirty  = true;
                        }
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.label(RichText::new("▸ Interventions").strong());
                    ui.label(RichText::new("Green = with interventions").small().weak());
                    ui.add_space(2.0);

                    macro_rules! chk {
                        ($field:ident, $label:literal, $tip:literal) => {{
                            if ui.checkbox(&mut self.ivs.$field, $label)
                                .on_hover_text($tip).changed() { self.dirty = true; }
                        }};
                    }
                    chk!(caloric_restriction,      "Caloric Restriction",    "−15% damage rate  (PMID 17460228)");
                    chk!(senolytics,               "Senolytics",             "Navitoclax/D+Q: extra NK clear ×0.3");
                    chk!(antioxidants,             "Antioxidants",           "NAC/MitoQ: −20% ROS post-step");
                    chk!(mtor_inhibition,          "mTOR Inhibition",        "Rapamycin: +20% protection factor");
                    chk!(telomerase,               "Telomerase",             "hTERT: −50% telomere loss/division");
                    chk!(nk_boost,                 "NK Cell Boost",          "IL-15: +30% NK efficiency");
                    chk!(stem_cell_therapy,        "Stem Cell Therapy",      "HSC transplant: pool ≥ 0.2");
                    chk!(epigenetic_reprogramming, "Epigenetic Reprog.",     "OSK: −30%/yr epigenetic overshoot");

                    ui.add_space(4.0);
                    ui.label(RichText::new("Strength").small());
                    if ui.add(egui::Slider::new(&mut self.ivs.strength, 0.0..=1.0)
                        .clamp_to_range(true)).changed() { self.dirty = true; }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.label(RichText::new("▸ Age cursor (yellow line)").strong());
                    ui.add(egui::Slider::new(&mut self.cursor_age, 0.0..=100.0)
                        .text("yr").clamp_to_range(true));

                    ui.add_space(8.0);
                    ui.separator();
                    ui.label(RichText::new("PMID 36583780").small().weak());
                    ui.label(RichText::new("Tkemaladze J., 2023").small().weak());
                });
            });

        // ── RIGHT panel ───────────────────────────────────────────────────────
        egui::SidePanel::right("values")
            .min_width(235.0).max_width(235.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("Values at age {:.0}", cursor))
                        .strong().size(13.0));
                    ui.separator();

                    let b_snap = nearest(&baseline, cursor);
                    let i_snap = if ivs_active { nearest(&with_ivs, cursor) } else { None };

                    if let Some(b) = b_snap {
                        for (idx, meta) in VARS.iter().enumerate() {
                            let bv = extract(b, idx);
                            let iv = i_snap.map(|s| extract(s, idx));
                            let col = val_color(bv, meta);
                            let is_epi = meta.y_max > 10.0;

                            ui.add_space(2.0);
                            ui.label(RichText::new(meta.name).small().strong()
                                .color(Color32::from_rgb(210, 215, 230)));

                            ui.horizontal(|ui| {
                                let base_txt = if is_epi {
                                    format!("Base {:.1} yr", bv)
                                } else {
                                    format!("Base {:.3}", bv)
                                };
                                ui.label(RichText::new(base_txt).small().color(col));

                                if let Some(iv_val) = iv {
                                    let delta = iv_val - bv;
                                    let sign  = if delta >= 0.0 { "+" } else { "" };
                                    let p     = pct(bv, iv_val);
                                    let ic    = val_color(iv_val, meta);
                                    let ivs_txt = if is_epi {
                                        format!("  Ivs {:.1} ({}{:.1}%)", iv_val, sign, p)
                                    } else {
                                        format!("  Ivs {:.3} ({}{:.0}%)", iv_val, sign, p)
                                    };
                                    ui.label(RichText::new(ivs_txt).small().color(ic));
                                }
                            });

                            // Biological description
                            for line in meta.description.lines() {
                                ui.label(RichText::new(line).small().weak()
                                    .color(Color32::from_rgb(130, 140, 150)));
                            }
                        }
                    }

                    // ── Summary at 80 ─────────────────────────────────────
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label(RichText::new("▸ Summary at age 80").strong());

                    if let (Some(b80), _) = (nearest(&baseline, 80.0), ()) {
                        let i80 = if ivs_active { nearest(&with_ivs, 80.0) } else { None };

                        for (field, name) in [
                            (0usize, "Damage  "), (4, "MCAI    "),
                            (5, "Telomere"), (7, "NK Eff. "),
                        ] {
                            let bv = extract(b80, field);
                            let iv = i80.map(|s| extract(s, field)).unwrap_or(bv);
                            let col = val_color(iv, &VARS[field]);
                            ui.label(RichText::new(
                                format!("{}: {:.3} → {:.3}  ({:+.0}%)",
                                    name, bv, iv, pct(bv, iv))
                            ).small().color(col));
                        }
                    }

                    // ── Frailty onset ─────────────────────────────────────
                    ui.add_space(6.0);
                    ui.separator();
                    ui.label(RichText::new("▸ Frailty onset (≥ 0.25)").strong());

                    let fa_b = baseline.iter()
                        .find(|s| s.mcai >= 0.25)
                        .map(|s| s.age_years).unwrap_or(100.0);
                    let fa_i = if ivs_active {
                        with_ivs.iter()
                            .find(|s| s.mcai >= 0.25)
                            .map(|s| s.age_years).unwrap_or(100.0)
                    } else { fa_b };

                    if ivs_active {
                        let gain = fa_i - fa_b;
                        let col  = if gain > 0.0 { Color32::from_rgb(75, 200, 95) }
                                   else if gain < 0.0 { Color32::from_rgb(230, 80, 60) }
                                   else { Color32::GRAY };
                        ui.label(RichText::new(format!(
                            "Base: age {:.0}   Ivs: age {:.0}", fa_b, fa_i)).small());
                        ui.label(RichText::new(format!(
                            "Lifespan gain: {:+.0} years", gain)).small().color(col));
                    } else {
                        ui.label(RichText::new(format!("Age {:.0}", fa_b)).small());
                    }
                });
            });

        // ── CENTER: 3×3 plots ─────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("plots")
                .num_columns(3)
                .spacing([6.0, 2.0])
                .show(ui, |ui| {
                    for row in 0..3usize {
                        for col in 0..3usize {
                            let idx  = row * 3 + col;
                            let meta = &VARS[idx];

                            let b_pts: PlotPoints =
                                baseline.iter().map(|s| [s.age_years, extract(s, idx)]).collect();
                            let i_pts: PlotPoints =
                                with_ivs.iter().map(|s| [s.age_years, extract(s, idx)]).collect();

                            let ivs_differs = ivs_active && with_ivs.iter().zip(baseline.iter())
                                .any(|(a, b)| (extract(a, idx) - extract(b, idx)).abs() > 1e-6);

                            ui.vertical(|ui| {
                                // Header
                                ui.label(RichText::new(
                                    format!("{} [{}]", meta.name, meta.unit))
                                    .small().strong()
                                    .color(Color32::from_rgb(195, 210, 235)));

                                Plot::new(format!("plt{}", idx))
                                    .height(158.0)
                                    .include_y(0.0)
                                    .include_y(meta.y_max)
                                    .x_axis_label("Age (yr)")
                                    .show_axes([true, true])
                                    .show(ui, |plot_ui| {
                                        // Threshold lines (not for epigenetic age)
                                        if meta.y_max <= 2.0 {
                                            let warn_col = Color32::from_rgba_premultiplied(195, 155, 0, 110);
                                            let crit_col = Color32::from_rgba_premultiplied(195, 55, 40, 130);
                                            let dash = egui_plot::LineStyle::Dashed { length: 8.0 };
                                            if meta.warn > 0.0 {
                                                plot_ui.hline(HLine::new(meta.warn)
                                                    .color(warn_col).width(1.2).style(dash));
                                                plot_ui.hline(HLine::new(meta.crit)
                                                    .color(crit_col).width(1.2).style(dash));
                                            }
                                        }
                                        // Cursor
                                        plot_ui.vline(VLine::new(cursor)
                                            .color(Color32::from_rgba_premultiplied(255, 250, 60, 180))
                                            .width(1.5)
                                            .style(egui_plot::LineStyle::Dashed { length: 6.0 }));
                                        // Baseline (blue)
                                        plot_ui.line(Line::new(b_pts)
                                            .name("Baseline")
                                            .color(Color32::from_rgb(100, 145, 215))
                                            .width(2.0));
                                        // Interventions (green)
                                        if ivs_differs {
                                            plot_ui.line(Line::new(i_pts)
                                                .name("Interventions")
                                                .color(Color32::from_rgb(65, 195, 85))
                                                .width(2.0));
                                        }
                                    });

                                // Value readout at cursor
                                if let Some(snap) = nearest(&baseline, cursor) {
                                    let val = extract(snap, idx);
                                    let col = val_color(val, meta);
                                    let txt = if meta.y_max > 10.0 {
                                        format!("@ age {:.0}: {:.1} yr", cursor, val)
                                    } else {
                                        format!("@ age {:.0}: {:.3}", cursor, val)
                                    };
                                    ui.label(RichText::new(txt).small().color(col));
                                }
                            });
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fabricate a minimal AgeSnapshot for testing extract()
    fn snap(age: f64, cd: f64, scp: f64, ros: f64, sasp: f64,
            fi: f64, tl: f64, ea: f64, nk: f64, fib: f64) -> AgeSnapshot {
        AgeSnapshot {
            age_years: age,
            centriole_damage: cd,
            stem_cell_pool: scp,
            ros_level: ros,
            sasp_level: sasp,
            mcai: fi,
            telomere_length: tl,
            differentiated_telomere_length: 1.0,
            epigenetic_age: ea,
            nk_efficiency: nk,
            fibrosis_level: fib,
            chip_vaf: 0.0,
        }
    }

    #[test]
    fn test_extract_all_indices() {
        let s = snap(30.0, 0.1, 0.9, 0.2, 0.15, 0.12, 0.8, 32.0, 0.7, 0.05);
        assert!((extract(&s, 0) - 0.1).abs() < 1e-9);   // centriole_damage
        assert!((extract(&s, 1) - 0.9).abs() < 1e-9);   // stem_cell_pool
        assert!((extract(&s, 2) - 0.2).abs() < 1e-9);   // ros_level
        assert!((extract(&s, 3) - 0.15).abs() < 1e-9);  // sasp_level
        assert!((extract(&s, 4) - 0.12).abs() < 1e-9);  // mcai
        assert!((extract(&s, 5) - 0.8).abs() < 1e-9);   // telomere_length
        assert!((extract(&s, 6) - 32.0).abs() < 1e-9);  // epigenetic_age
        assert!((extract(&s, 7) - 0.7).abs() < 1e-9);   // nk_efficiency
        assert!((extract(&s, 8) - 0.05).abs() < 1e-9);  // fibrosis_level
    }

    #[test]
    fn test_pct_increase() {
        assert!((pct(1.0, 1.5) - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_pct_decrease() {
        assert!((pct(1.0, 0.9) - (-10.0)).abs() < 1e-6);
    }

    #[test]
    fn test_pct_zero_base_returns_zero() {
        assert_eq!(pct(0.0, 5.0), 0.0);
    }

    #[test]
    fn test_nearest_returns_closest() {
        let snaps = vec![
            snap(10.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 1.0, 0.0),
            snap(20.0, 0.1, 0.9, 0.1, 0.1, 0.05, 0.9, 20.0, 0.9, 0.01),
            snap(30.0, 0.2, 0.8, 0.2, 0.2, 0.1, 0.8, 30.0, 0.8, 0.02),
        ];
        let n = nearest(&snaps, 22.0).unwrap();
        assert!((n.age_years - 20.0).abs() < 1e-9, "closest to 22 should be 20");
    }

    #[test]
    fn test_nearest_empty_returns_none() {
        let snaps: Vec<AgeSnapshot> = vec![];
        assert!(nearest(&snaps, 50.0).is_none());
    }

    #[test]
    fn test_val_color_high_bad_critical() {
        let meta = &VARS[0]; // centriole_damage: bad_is_high=true, crit=0.70
        let col = val_color(0.80, meta);
        // Should be reddish
        assert!(col.r() > 180, "critical high value should be red");
    }

    #[test]
    fn test_val_color_low_bad_critical() {
        let meta = &VARS[1]; // stem_cell_pool: bad_is_high=false, crit=0.30
        let col = val_color(0.10, meta);
        assert!(col.r() > 180, "critical low value should be red");
    }

    #[test]
    fn test_val_color_good_bad_is_high() {
        let meta = &VARS[0]; // centriole_damage: warn=0.40
        let col = val_color(0.10, meta); // well below warn
        assert!(col.g() > 150, "good low value should be green");
    }

    #[test]
    fn test_vars_count() {
        assert_eq!(VARS.len(), 9, "must have exactly 9 variable definitions");
    }

    #[test]
    fn test_vars_thresholds_ordered() {
        for v in &VARS {
            if v.bad_is_high && v.warn > 0.0 {
                assert!(v.warn <= v.crit || v.y_max > 10.0,
                    "{}: warn ({}) should be ≤ crit ({}) for bad_is_high",
                    v.name, v.warn, v.crit);
            }
        }
    }

    #[test]
    fn test_simulation_runs_without_panic() {
        // Verify that a default simulation produces snapshots
        let cfg = SimulationConfig::default();
        let snaps = AgingEngine::new(cfg).unwrap().run(1);
        assert!(!snaps.is_empty(), "simulation should produce snapshots");
        assert!(snaps.last().unwrap().age_years >= 99.0, "should run to 100 yr");
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CDATA v3.0 — Cell Digital Twin Simulator")
            .with_inner_size([1300.0, 840.0]),
        ..Default::default()
    };
    eframe::run_native(
        "CDATA v3.0",
        options,
        Box::new(|_cc| Box::new(CdataApp::new())),
    )
}
