mod app;

fn load_cjk_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Try to load a CJK font from the system
    let cjk_font_data = {
        // Windows: Microsoft YaHei or SimHei
        let win_paths = [
            "C:/Windows/Fonts/msyh.ttc",
            "C:/Windows/Fonts/simhei.ttf",
            "C:/Windows/Fonts/msyhbd.ttc",
        ];
        // Linux: Noto Sans CJK
        let linux_paths = [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        ];

        let paths: Vec<&str> = if cfg!(windows) {
            win_paths.to_vec()
        } else {
            linux_paths.to_vec()
        };

        paths.iter().find_map(|p| std::fs::read(p).ok())
    };

    if let Some(data) = cjk_font_data {
        fonts.font_data.insert(
            "cjk".to_owned(),
            egui::FontData::from_owned(data),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "cjk".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "cjk".to_owned());
    }

    ctx.set_fonts(fonts);
}

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
            load_cjk_fonts(&cc.egui_ctx);
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(app::WrappeApp::default()))
        }),
    )
}
