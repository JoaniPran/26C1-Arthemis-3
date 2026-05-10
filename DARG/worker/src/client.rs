use std::net::TcpStream;
use std::io::{BufRead, BufReader, Write};
use std::thread;
use common::Message;
use crate::executor::TaskExecutor;
use crate::heartbeat;   

pub fn run(worker_id: String, addr: &str) {
    loop {
        match TcpStream::connect(addr) {
            Ok(mut stream) => {
                println!("Conectado como {}", worker_id);

                let mut reg_msg = serde_json::to_string(&Message::RegisterWorker { id: worker_id.clone() }).unwrap();
                reg_msg.push('\n');

                let _ = stream.write_all(reg_msg.as_bytes());

                heartbeat::start_heartbeat_loop(stream.try_clone().unwrap());

                let reader = BufReader::new(stream.try_clone().unwrap());
                
                for line in reader.lines() {
                    if let Ok(text) = line {
                        if let Ok(msg) = serde_json::from_str::<Message>(&text) {
                            if let Message::AssignTask { task_name, command } = msg {
                                let code = TaskExecutor::execute(task_name.clone(), &command, &mut stream);
                                let status = if code == 0 { "Success" } else { "Failed" };

                                let mut resp = serde_json::to_string(&Message::TaskStatus { 
                                    task_name, 
                                    status: status.to_string()
                                }).unwrap();
                                resp.push('\n');
                                let _ = stream.write_all(resp.as_bytes());
                            }
                        }
                    } else {
                        break;
                    }
                }
                println!("Conexión perdida. Reintentando en 5 segundos...");
            }
            Err(_) => thread::sleep(std::time::Duration::from_secs(5)),
        }
    }
}