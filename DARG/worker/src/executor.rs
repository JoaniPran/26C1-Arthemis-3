use common::Message;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn execute(task_id: i32, command_str: &str, stream: &mut TcpStream) -> i32 {
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
                let _ = Self::send_log_fragment(stream, task_id, &err_msg);
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

        let child_stdout = child.stdout.take().expect("Stdout piped pero no capturado");
        let child_stderr = child.stderr.take().expect("Stderr piped pero no capturado");

        let stdout_handle = thread::spawn(move || {
            Self::stream_output_to_network(child_stdout, stream_stdout, task_id);
        });

        let stderr_handle = thread::spawn(move || {
            Self::stream_output_to_network(child_stderr, stream_stderr, task_id);
        });

        stdout_handle.join().ok();
        stderr_handle.join().ok();

        child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1)
    }

    fn stream_output_to_network<R: Read>(reader: R, mut net_stream: TcpStream, task_id: i32) {
        let buf_reader = BufReader::new(reader);

        for line in buf_reader.lines().map_while(Result::ok) {
            if Self::send_log_fragment(&mut net_stream, task_id, &line).is_err() {
                break;
            }
        }
    }

    fn send_log_fragment(
        stream: &mut TcpStream,
        task_id: i32,
        content: &str,
    ) -> std::io::Result<()> {
        let msg = Message::LogFragment {
            task_id,
            content: content.to_string(),
        };

        let mut msg_str = serde_json::to_string(&msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        msg_str.push('\n');
        stream.write_all(msg_str.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;

    fn create_dummy_connection() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let client = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        let (server, _) = listener.accept().unwrap();

        (client, server)
    }

    #[test]
    fn test_execute_success_and_logs() {
        let (mut client, server) = create_dummy_connection();

        #[cfg(not(target_os = "windows"))]
        let cmd = "echo Hola Mundo";
        #[cfg(target_os = "windows")]
        let cmd = "cmd /C echo Hola Mundo";

        let exit_code = TaskExecutor::execute(10, cmd, &mut client);

        assert_eq!(exit_code, 0);

        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        assert!(line.contains("LogFragment"));
        assert!(line.contains("10"));
        assert!(line.contains("Hola Mundo"));
    }

    #[test]
    fn test_execute_invalid_command() {
        let (mut client, server) = create_dummy_connection();

        let cmd = "comando_inventado_12345";

        let exit_code = TaskExecutor::execute(99, cmd, &mut client);

        assert_eq!(exit_code, 127);

        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        assert!(line.contains("LogFragment"));
        assert!(line.contains("99"));
        assert!(line.contains("[CRÍTICO]"));
        assert!(line.contains("comando_inventado_12345"));
    }
}
