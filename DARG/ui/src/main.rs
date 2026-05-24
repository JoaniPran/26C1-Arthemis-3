mod app;
mod fonts;
mod ui;
mod utils;

use app::ArthemisApp;
use std::thread;
use std::sync::mpsc;

fn main() -> eframe::Result<()> {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        println!("Iniciando Coordinador Interno Arthemis...");
        coordinator::server::start_server("8080", Some(tx));
    });

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
        Box::new(|cc| {
            let mut app = ArthemisApp::new(cc);
            app.backend_rx = Some(rx); 
            Box::new(app)
        }),
    )
}
