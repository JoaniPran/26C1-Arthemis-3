use common::Message;
use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

pub fn start_heartbeat_loop(mut stream: TcpStream) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));

            let msg = Message::Heartbeat;

            match serde_json::to_string(&msg) {
                Ok(mut json_msg) => {
                    json_msg.push('\n');

                    if stream.write_all(json_msg.as_bytes()).is_err() {
                        println!("Latido detenido: Conexión con el Coordinador perdida.");
                        break;
                    }
                }
                Err(e) => {
                    println!("Error critico al generar el latido: {}", e);
                    break;
                }
            }
        }
    });
}
