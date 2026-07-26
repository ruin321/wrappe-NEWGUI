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
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(app::WrappeApp::default()))
        }),
    )
}
