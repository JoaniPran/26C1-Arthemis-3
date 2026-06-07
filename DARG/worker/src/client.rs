use crate::executor::TaskExecutor;
use crate::heartbeat;
use common::Message;
use flate2::read::GzDecoder;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result as IoResult, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use tar::Archive;

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
        Ok(Message::AssignTask {
            task_id,
            task_name,
            command,
            input_artifact,
            artifact_path,
        }) => {
            println!("Tarea recibida: {} -> {}", task_name, command);

            // 1. Si vino un artefacto de una tarea previa, lo descomprimimos antes de correr el comando
            if let Some(bytes) = input_artifact {
                println!(
                    "Recibido artefacto de entrada para la tarea {}. Descomprimiendo...",
                    task_id
                );
                if let Err(e) = decompress_artifact_to_disk(&bytes) {
                    println!(
                        "[Error] No se pudo preparar el entorno con el artefacto: {}",
                        e
                    );
                    // Avisamos al coordinador que falló la preparación
                    let resp_msg = Message::TaskStatus {
                        task_id,
                        status: "Failed_Environment_Setup".to_string(),
                        output_artifact: None,
                    };
                    return send_message(stream, &resp_msg);
                }
            }

            // 2. Ejecutamos la tarea pasándole la ruta del artefacto que tiene que buscar al terminar
            let (code, output_bytes) = TaskExecutor::execute(
                task_id,
                &command,
                artifact_path.as_deref(), // Convertimos Option<String> a Option<&str>
                stream,
            );

            let status = if code == 0 { "Success" } else { "Failed" };

            // 3. Enviamos el estatus junto con los bytes del artefacto si es que la tarea generó uno
            let resp_msg = Message::TaskStatus {
                task_id,
                status: status.to_string(),
                output_artifact: output_bytes, // <-- Metemos los bytes resultantes acá
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

pub fn descubrir_carpeta_del_comando(comando: &str) -> String {
    if comando.contains("git clone") {
        if let Some(ultima_palabra) = comando.split_whitespace().last() {
            if !ultima_palabra.contains("http") && !ultima_palabra.contains("git@") {
                return ultima_palabra.to_string();
            }
        }
    }

    if let Some(pos) = comando.find("mkdir -p ") {
        if let Some(ruta) = comando[pos + 9..].split_whitespace().next() {
            return ruta.to_string();
        }
    } else if let Some(pos) = comando.find("mkdir ") {
        if let Some(ruta) = comando[pos + 6..].split_whitespace().next() {
            return ruta.to_string();
        }
    }

    "output".to_string()
}

fn decompress_artifact_to_disk(bytes: &[u8]) -> IoResult<()> {
    let tar_gz_decoder = GzDecoder::new(bytes);
    let mut archive = Archive::new(tar_gz_decoder);

    archive.unpack(".").map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Falló al descomprimir el artefacto de entrada: {}", e),
        )
    })?;

    println!("¡Artefacto de entrada descomprimido con éxito en el directorio local!");
    Ok(())
}
