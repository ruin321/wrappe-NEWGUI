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
        Box::new(|_cc| Ok(Box::new(app::WrappeApp::default()))),
    )
}
