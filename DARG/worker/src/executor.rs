use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use common::Message;

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn execute(task_name: String, command_str: &str, stream: &mut TcpStream) -> i32 {
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() { return -1; }

        let mut child = Command::new(parts[0])
            .args(&parts[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Fallo al iniciar el comando");

        let mut stdout_stream = stream.try_clone().expect("Error al clonar stream");
        let mut stderr_stream = stream.try_clone().expect("Error al clonar stream");
        let t_name = task_name.clone();
        let t_name_err = task_name.clone();

        let child_stdout = child.stdout.take().expect("No se pudo capturar stdout");
        let child_stderr = child.stderr.take().expect("No se pudo capturar stderr");

        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(child_stdout);
            for line in reader.lines().map_while(Result::ok) {
                let msg = Message::LogFragment { task_name: t_name.clone(), content: line };
                let mut msg_str = serde_json::to_string(&msg).unwrap();
                msg_str.push('\n');

                if stdout_stream.write_all(msg_str.as_bytes()).is_err() {
                    break; 
                }
            }
        });

        let stderr_handle = thread::spawn(move || {
            let reader = BufReader::new(child_stderr);
            for line in reader.lines().map_while(Result::ok) {
                let msg = Message::LogFragment { task_name: t_name_err.clone(), content: format!("[ERR] {}", line) };
                let mut msg_str = serde_json::to_string(&msg).unwrap();
                msg_str.push('\n');
                
                if stderr_stream.write_all(msg_str.as_bytes()).is_err() {
                    break; 
                }
            }
        });

        stdout_handle.join().ok();
        stderr_handle.join().ok();

        child.wait().map(|s| s.code().unwrap_or(0)).unwrap_or(1)
    }
}

#[cfg(test)]
fn run_external_command(command_str: &str) -> i32 {
    use std::process::Command;

    println!("Ejecutando (Modo Test): {}", command_str);

    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() { return -1; }

    let process = Command::new(parts[0])
        .args(&parts[1..])
        .spawn();

    match process {
        Ok(mut child) => {
            match child.wait() {
                Ok(status) => {
                    let code = status.code().unwrap_or(1);
                    println!("Proceso terminado con código: {}", code);
                    code
                }
                Err(_) => 1,
            }
        }
        Err(e) => {
            eprintln!("Error al iniciar el comando: {}", e);
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_simple_command() {
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