use std::net::TcpStream;
use std::io::Write;
use common::Message;


fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("No se pudo conectar al coordinador.");

    let register_msg = Message::RegisterWorker {
        id: "worker-1".to_string()
    };

    let j = serde_json::to_string(&register_msg).expect("Error al serializar.");
    stream.write_all(j.as_bytes()).expect("Error al enviar.");

    println!("Registro enviado al coordinador.");
}
