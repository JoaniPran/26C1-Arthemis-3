use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::BufReader;
use common::Message;
use serde_json::Deserializer;

fn handle_worker(stream: TcpStream) {
    let reader = BufReader::new(&stream);
    let mut stream_iter = Deserializer::from_reader(reader).into_iter::<Message>();

    while let Some(Ok(msg)) = stream_iter.next() {
        match msg {
            Message::RegisterWorker { id } => println!("Worker conectado: {}.", id),
            Message::LogFragment { task_name, content } => println!("[{}] LOG: {}", task_name, content),
            _ => println!("Mensaje recibido no manejado.")
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("No se pudo bindear el puerto.");
    println!("Coordinador escuchando en el puerto 8080...");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(|| handle_worker(s));
            }
            Err(e) => println!("Error de conexion: {}.", e),
        }
    }
}
