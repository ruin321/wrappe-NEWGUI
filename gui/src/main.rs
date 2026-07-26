mod app;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([850.0, 720.0])
            .with_min_inner_size([600.0, 500.0])
            .with_title("wrappe GUI - Pack executables into self-contained binaries"),
        ..Default::default()
    };

    eframe::run_native(
        "wrappe GUI",
        options,
        Box::new(|cc| {
            // Start with dark mode
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(app::WrappeApp::default()))
        }),
    )
}
