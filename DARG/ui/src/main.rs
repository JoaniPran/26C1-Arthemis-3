mod app;
mod fonts;
mod ui;
mod utils;

use app::ArthemisApp;
use std::sync::mpsc;
use std::thread;

fn main() -> eframe::Result<()> {
    let coordinator_config = parse_coordinator_config();
    let (tx, rx) = mpsc::channel::<String>();

    if let Some(port) = coordinator_config {
        thread::spawn(move || {
            println!("Iniciando Coordinador Interno Arthemis en {}...", port);
            coordinator::server::start_server(&port, Some(tx));
        });
    }

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

fn parse_coordinator_config() -> Option<String> {
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-coordinator" | "--no-server" => return None,
            "--coordinator-port" | "--coordinator-addr" => {
                if let Some(value) = args.next() {
                    return Some(value);
                }
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--coordinator-port=") {
                    return Some(value.to_string());
                }

                if let Some(value) = arg.strip_prefix("--coordinator-addr=") {
                    return Some(value.to_string());
                }
            }
        }
    }

    Some(std::env::var("ARTHEMIS_COORDINATOR_ADDR").unwrap_or_else(|_| "8080".to_string()))
}
