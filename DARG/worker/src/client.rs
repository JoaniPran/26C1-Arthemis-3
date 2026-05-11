use crate::executor::TaskExecutor;
use crate::heartbeat;
use common::Message;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result as IoResult, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

pub fn run(worker_id: String, addr: &str) {
    loop {
        match TcpStream::connect(addr) {
            Ok(stream) => {
                println!("Conectado al coordinador como '{}'", worker_id);

                if let Err(e) = handle_session(&worker_id, stream) {
                    println!("Conexión terminada o perdida: {}. Reintentando...", e);
                }
            }
            Err(e) => {
                println!(
                    "No se pudo conectar al Coordinador: {}. Reintentando en 5s...",
                    e
                )
            }
        }
        thread::sleep(Duration::from_secs(5));
    }
}

fn handle_session(worker_id: &str, mut stream: TcpStream) -> IoResult<()> {
    let reg_msg = Message::RegisterWorker {
        id: worker_id.to_string(),
    };
    send_message(&mut stream, &reg_msg)?;

    let heartbeat_stream = stream.try_clone()?;
    heartbeat::start_heartbeat_loop(heartbeat_stream);

    let read_stream = stream.try_clone()?;
    let reader = BufReader::new(read_stream);

    for line in reader.lines() {
        let text = line?;
        process_message(&text, &mut stream)?;
    }

    Ok(())
}

fn process_message(text: &str, stream: &mut TcpStream) -> IoResult<()> {
    match serde_json::from_str::<Message>(text) {
        Ok(Message::AssignTask { task_name, command }) => {
            println!("Tarea recibida: {} -> {}", task_name, command);

            let code = TaskExecutor::execute(task_name.clone(), &command, stream);
            let status = if code == 0 { "Success" } else { "Failed" };

            let resp_msg = Message::TaskStatus {
                task_name,
                status: status.to_string(),
            };
            send_message(stream, &resp_msg)?;
        }
        Ok(_) => {}
        Err(e) => {
            println!("JSON malformado recibido: {}. Texto: {}", e, text);
        }
    }
    Ok(())
}

fn send_message(stream: &mut TcpStream, msg: &Message) -> IoResult<()> {
    let mut json = serde_json::to_string(msg).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    json.push('\n');
    stream.write_all(json.as_bytes())
}
