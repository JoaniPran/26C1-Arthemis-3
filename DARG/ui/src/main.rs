mod app;
mod fonts;
mod ui;

use app::ArthemisApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_inner_size([1920.0, 1080.0])
            .with_title("Arthemis Orchestrator v1.0.0"),
        ..Default::default()
    };

    eframe::run_native(
        "Arthemis UI",
        options,
        Box::new(|cc| Box::new(ArthemisApp::new(cc))),
    )
}
