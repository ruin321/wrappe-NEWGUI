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
            // Try to add CJK font support without breaking default fonts
            let mut fonts = egui::FontDefinitions::default();
            let cjk_data = std::fs::read("C:/Windows/Fonts/msyh.ttc")
                .or_else(|_| std::fs::read("C:/Windows/Fonts/simhei.ttf"))
                .or_else(|_| std::fs::read("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"))
                .ok();
            if let Some(data) = cjk_data {
                fonts.font_data.insert("cjk".to_owned(), egui::FontData::from_owned(data));
                fonts.families.entry(egui::FontFamily::Proportional).or_default().push("cjk".to_owned());
                fonts.families.entry(egui::FontFamily::Monospace).or_default().push("cjk".to_owned());
                cc.egui_ctx.set_fonts(fonts);
            }
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(app::WrappeApp::default()))
        }),
    )
}
