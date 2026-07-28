mod app;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 750.0])
            .with_min_inner_size([650.0, 550.0])
            .with_title("wrappe GUI"),
        ..Default::default()
    };

    eframe::run_native(
        "wrappe GUI",
        options,
        Box::new(|cc| {
            // Try to add CJK font support
            let mut fonts = egui::FontDefinitions::default();
            // Windows fonts: try multiple to cover Chinese + Japanese + Korean
            let font_paths = if cfg!(windows) {
                vec![
                    "C:/Windows/Fonts/msyh.ttc",       // Microsoft YaHei (CN)
                    "C:/Windows/Fonts/malgun.ttf",      // Malgun Gothic (KR)
                    "C:/Windows/Fonts/msgothic.ttc",    // MS Gothic (JP)
                    "C:/Windows/Fonts/simhei.ttf",      // SimHei (CN fallback)
                    "C:/Windows/Fonts/gulim.ttc",       // Gulim (KR fallback)
                ]
            } else {
                vec![
                    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
                ]
            };
            for path in font_paths {
                if let Ok(data) = std::fs::read(path) {
                    let name = format!("font_{}", fonts.font_data.len());
                    fonts.font_data.insert(name.clone(), egui::FontData::from_owned(data));
                    fonts.families.entry(egui::FontFamily::Proportional).or_default().push(name.clone());
                    fonts.families.entry(egui::FontFamily::Monospace).or_default().push(name);
                }
            }
            cc.egui_ctx.set_fonts(fonts);
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(app::WrappeApp::default()))
        }),
    )
}
