mod app;
mod fonts;
mod ui;
mod utils;

use app::ArthemisApp;
use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();
    let coordinator_ip = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let coordinator_addr = format!("{}:8082", coordinator_ip);

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        println!(
            "Conectando al Coordinador Arthemis ({}) ...",
            coordinator_addr
        );

        match TcpStream::connect(&coordinator_addr) {
            Ok(stream) => {
                let reader = BufReader::new(stream);
                for line in reader.lines() {
                    if let Ok(msg) = line {
                        if tx.send(msg).is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("No se pudo conectar a {}: {}", coordinator_addr, e);
            }
        }
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
