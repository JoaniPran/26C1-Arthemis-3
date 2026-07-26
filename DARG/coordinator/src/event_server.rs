use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

pub type EventSender = mpsc::Sender<String>;

pub fn start_event_server(port: &str) -> EventSender {
    let (tx, rx) = mpsc::channel::<String>();
    let clients = Arc::new(Mutex::new(Vec::new()));

    let clients_clone = clients.clone();
    let addr = format!("0.0.0.0:{}", port);

    thread::spawn(move || {
        let listener =
            TcpListener::bind(&addr).expect("No se pudo bindear el puerto de eventos UI");
        println!("Servidor de Eventos para la UI escuchando en {}...", addr);

        for s in listener.incoming().flatten() {
            let _ = s.set_nonblocking(true); // Evita bloqueos
            clients_clone.lock().unwrap().push(s);
            println!("UI conectada al servidor de eventos.");
        }
    });

    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            let mut clients_guard = clients.lock().unwrap();
            let mut formatted_msg = msg.clone();
            formatted_msg.push('\n');

            // Enviamos a todos los clientes activos y eliminamos los que cerraron la ventana
            clients_guard.retain_mut(|client| client.write_all(formatted_msg.as_bytes()).is_ok());
        }
    });

    tx
}
