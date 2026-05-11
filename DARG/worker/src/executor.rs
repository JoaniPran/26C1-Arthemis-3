use common::Message;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn execute(task_name: String, command_str: &str, stream: &mut TcpStream) -> i32 {
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() {
            return -1;
        }

        let mut child = match Command::new(parts[0])
            .args(&parts[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child_process) => child_process,
            Err(e) => {
                let err_msg = format!(
                    "[CRÍTICO] Error del SO al iniciar comando '{}': {}",
                    parts[0], e
                );
                let _ = Self::send_log_fragment(stream, &task_name, &err_msg);
                return 127;
            }
        };

        let stream_stdout = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                println!("Error crítico al clonar socket para stdout: {}", e);
                return -1;
            }
        };
        let stream_stderr = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                println!("Error crítico al clonar socket para stderr: {}", e);
                return -1;
            }
        };

        let t_name_out = task_name.clone();
        let t_name_err = task_name;

        let child_stdout = child.stdout.take().expect("Stdout piped pero no capturado");
        let child_stderr = child.stderr.take().expect("Stderr piped pero no capturado");

        let stdout_handle = thread::spawn(move || {
            Self::stream_output_to_network(child_stdout, stream_stdout, t_name_out);
        });

        let stderr_handle = thread::spawn(move || {
            Self::stream_output_to_network(child_stderr, stream_stderr, t_name_err);
        });

        stdout_handle.join().ok();
        stderr_handle.join().ok();

        child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1)
    }

    fn stream_output_to_network<R: Read>(reader: R, mut net_stream: TcpStream, task_name: String) {
        let buf_reader = BufReader::new(reader);

        for line in buf_reader.lines().map_while(Result::ok) {
            if Self::send_log_fragment(&mut net_stream, &task_name, &line).is_err() {
                break;
            }
        }
    }

    fn send_log_fragment(
        stream: &mut TcpStream,
        task_name: &str,
        content: &str,
    ) -> std::io::Result<()> {
        let msg = Message::LogFragment {
            task_name: task_name.to_string(),
            content: content.to_string(),
        };

        let mut msg_str = serde_json::to_string(&msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        msg_str.push('\n');
        stream.write_all(msg_str.as_bytes())
    }
}

#[cfg(test)]
fn run_external_command(command_str: &str) -> i32 {
    use std::process::Command;

    println!("Ejecutando (Modo Test): {}", command_str);

    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        return -1;
    }

    let process = Command::new(parts[0]).args(&parts[1..]).spawn();

    match process {
        Ok(mut child) => match child.wait() {
            Ok(status) => {
                let code = status.code().unwrap_or(1);
                println!("Proceso terminado con código: {}", code);
                code
            }
            Err(_) => 1,
        },
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
        assert_eq!(
            code, 0,
            "El comando debería haber terminado con éxito (código 0)"
        );
    }

    #[test]
    fn test_execute_failing_command() {
        let code = run_external_command("not-a-real-command-12345");
        assert_ne!(
            code, 0,
            "Un comando inexistente no debería devolver código 0"
        );
    }
}
