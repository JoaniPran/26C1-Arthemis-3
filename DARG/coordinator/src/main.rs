mod state;
mod handler;
mod supervisor;

use std::net::TcpListener;
use std::thread;
use state::CoordinatorState;
use handler::WorkerHandler;

fn main() {
    let port = std::env::args().nth(1).unwrap_or_else(|| "8080".to_string());
    start_server(&port);
}

fn start_server(port: &str) {
    let addr = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(&addr).expect("No se pudo bindear el puerto.");
    let state = CoordinatorState::new();

    supervisor::start_watchdog(state.clone());

    println!("Coordinador Arthemis 3 escuchando en {}...", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let state_clone = state.clone();

                thread::spawn(move || {
                    WorkerHandler::handle_connection(s, state_clone);
                });
            }
            Err(e) => println!("Error de conexión: {}.", e),
        }
    }
}