//! Cell DT — Simulation Configurator
//! Multilingual GUI: EN / FR / ES / RU / ZH / AR / KA

use cell_dt_gui::ConfigApp;
use eframe::{NativeOptions, egui};
use egui::FontDefinitions;

fn load_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    // Georgian (Noto Sans Georgian)
    let georgian_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fonts/NotoSansGeorgian-Regular.ttf"
    );
    if let Ok(data) = std::fs::read(georgian_path) {
        fonts.font_data.insert(
            "NotoSansGeorgian".to_owned(),
            egui::FontData::from_owned(data),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push("NotoSansGeorgian".to_owned());
    }

    // Arabic (Noto Sans Arabic)
    let arabic_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fonts/NotoSansArabic-Regular.ttf"
    );
    if let Ok(data) = std::fs::read(arabic_path) {
        fonts.font_data.insert(
            "NotoSansArabic".to_owned(),
            egui::FontData::from_owned(data),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push("NotoSansArabic".to_owned());
    }

    // CJK (Chinese) — Noto Sans CJK (system)
    let cjk_paths = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    ];
    for p in &cjk_paths {
        if let Ok(data) = std::fs::read(p) {
            fonts.font_data.insert(
                "NotoSansCJK".to_owned(),
                egui::FontData::from_owned(data),
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .push("NotoSansCJK".to_owned());
            break;
        }
    }

    fonts
}

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_min_inner_size([900.0, 650.0])
            .with_resizable(true)
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Cell DT — Simulation Configurator",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_fonts(load_fonts());
            Box::new(ConfigApp::new())
        }),
    )
}
