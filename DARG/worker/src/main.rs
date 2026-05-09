use std::net::TcpStream;
use std::io::{Write, BufReader, BufRead}; // Consolidado: incluye BufRead para los logs [cite: 17, 62]
use std::process::{Command, Stdio};       // Consolidado: incluye Stdio para el piping [cite: 61, 182]
use std::thread;
use std::time::Duration;
use common::Message;
use serde_json::Deserializer;// Agregamos Stdio

fn main() {
    // Leer el ID del worker desde los argumentos de la terminal
    let args: Vec<String> = std::env::args().collect();
    let worker_id = if args.len() > 1 {
        args[1].clone()
    } else {
        "worker-default".to_string()
    };

    let addr = "127.0.0.1:8080";
    let mut stream = None;

    // Criterio de Aceptación: Manejo de error y reintentos si el Coordinador no está disponible
    while stream.is_none() {
        match TcpStream::connect(addr) {
            Ok(s) => {
                println!("🔗 Conectado al coordinador en {}", addr);
                stream = Some(s);
            }
            Err(_) => {
                println!("⚠️ Coordinador no encontrado. Reintentando en 5 segundos...");
                thread::sleep(Duration::from_secs(5));
            }
        }
    }

    let mut stream = stream.unwrap();
    let stream_write = stream.try_clone().expect("Error al clonar stream para lectura");

    // 1. Handshake inicial
    let register_msg = Message::RegisterWorker {
        id: worker_id, // Usar el ID dinámico
    };
    let j = serde_json::to_string(&register_msg).expect("Error al serializar.");
    stream.write_all(j.as_bytes()).expect("Error al enviar registro.");

    // 2. Bucle de lectura de instrucciones (Wait for AssignTask)
    let reader = BufReader::new(stream_write);
    let mut stream_iter = Deserializer::from_reader(reader).into_iter::<Message>();

    println!("⌛ Esperando instrucciones del coordinador...");

    while let Some(Ok(msg)) = stream_iter.next() {
        match msg {
            Message::AssignTask { task_name, command } => {
                let exit_code = run_command_with_logs(task_name.clone(), &command, &mut stream);
        
                let status = if exit_code == 0 { "Success".to_string() } else { "Failed".to_string() };
                let response = Message::TaskStatus { task_name, status };
                let j = serde_json::to_string(&response).unwrap();
                stream.write_all(j.as_bytes()).ok();
            }
            _ => {}//println!("Mensaje del servidor no reconocido."),
        }
    }

    println!("🔌 El coordinador cerró la conexión. Finalizando worker.");
}

/// Ejecuta un comando en el sistema operativo y devuelve el código de salida.
fn run_external_command(command_str: &str) -> i32 {
    println!("⚙️ Ejecutando: {}", command_str);
    
    // Separamos el comando de sus argumentos (simplificado para este hito)
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() { return -1; }

    let process = Command::new(parts[0])
        .args(&parts[1..])
        .spawn(); // Iniciamos el proceso

    match process {
        Ok(mut child) => {
            // Criterio de Aceptación: Esperar al hijo para evitar procesos "zombie"
            match child.wait() {
                Ok(status) => {
                    let code = status.code().unwrap_or(1);
                    println!("✅ Proceso terminado con código: {}", code);
                    code
                }
                Err(_) => 1,
            }
        }
        Err(e) => {
            eprintln!("❌ Error al iniciar el comando: {}", e);
            -1
        }
    }
}

// Modificamos la función para que reciba el stream y el nombre de la tarea
fn run_command_with_logs(task_name: String, command_str: &str, stream: &mut TcpStream) -> i32 {
    println!("⚙️ Ejecutando con streaming de logs: {}", command_str);
    
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() { return -1; }

    // Tarea 1.4: Configurar piping de stdout
    let mut child = Command::new(parts[0])
        .args(&parts[1..])
        .stdout(Stdio::piped()) // Redirigimos la salida a un pipe 
        .stderr(Stdio::piped()) // También capturamos errores
        .spawn()
        .expect("Fallo al iniciar el comando");

    // Tarea 1.4: Procesar stdout línea por línea [cite: 62]
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(content) = line {
                // Tarea 1.4: Enviar log fragmentado 
                let log_msg = Message::LogFragment {
                    task_name: task_name.clone(),
                    content,
                };
                let j = serde_json::to_string(&log_msg).expect("Error al serializar log");
                let _ = stream.write_all(j.as_bytes()); // Enviamos al coordinador
            }
        }
    }

    // Esperar a que termine para evitar zombies [cite: 164]
    match child.wait() {
        Ok(status) => status.code().unwrap_or(0),
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_simple_command() {
        // En Linux/macOS usamos 'ls', en Windows 'dir' o 'echo'
        #[cfg(not(target_os = "windows"))]
        let cmd = "ls";
        #[cfg(target_os = "windows")]
        let cmd = "cmd /C dir";

        let code = run_external_command(cmd);
        assert_eq!(code, 0, "El comando debería haber terminado con éxito (código 0)");
    }

    #[test]
    fn test_execute_failing_command() {
        let code = run_external_command("not-a-real-command-12345");
        assert_ne!(code, 0, "Un comando inexistente no debería devolver código 0");
    }
}