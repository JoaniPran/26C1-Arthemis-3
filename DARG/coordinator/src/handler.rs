use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use common::Message;
use crate::state::CoordinatorState;

pub struct WorkerHandler {
    stream: TcpStream,
    state: CoordinatorState,
    worker_id: Option<String>,
}

impl WorkerHandler {
    pub fn new(stream: TcpStream, state: CoordinatorState) -> Self {
        Self {
            stream,
            state,
            worker_id: None,
        }
    }

    pub fn handle_connection(stream: TcpStream, state: CoordinatorState) {
        let handler = WorkerHandler::new(stream, state);
        handler.run();
    }

    fn run(mut self) {
        let read_stream = self.stream.try_clone().expect("No se pudo clonar el stream para lectura");
        let reader = BufReader::new(read_stream);
        
        for line in reader.lines() {
            match line {
                Ok(text) => {
                    match serde_json::from_str::<Message>(&text) {
                        Ok(msg) => self.process_message(msg),
                        Err(e) => println!("Error crítico parseando JSON: {}. Texto recibido: {}", e, text),
                    }
                }
                Err(_) => break,
            }
        }

        self.cleanup();
    }

    fn process_message(&mut self, msg: Message) {
        match msg {
            Message::RegisterWorker { id } => self.handle_register(id),
            Message::LogFragment { task_name, content } => self.handle_log(task_name, content),
            Message::Heartbeat => self.handle_heartbeat(),
            Message::TaskStatus { task_name, status } => self.handle_status(task_name, status),
            _ => println!("Mensaje recibido no manejado."),
        }
    }

    fn handle_register(&mut self, id: String) {
        self.worker_id = Some(id.clone());

        self.state.add_worker(id.clone(), self.stream.try_clone().expect("Error al clonar stream"));
        println!("Worker registrado: {}.", id);

        let assign_msg = Message::AssignTask {
            task_name: "Prueba_Streaming".to_string(),
            command: "ping 127.0.0.1 -c 6".to_string()
        };
        let mut json_msg = serde_json::to_string(&assign_msg).unwrap();
        json_msg.push('\n');

        let _ = self.stream.write_all(json_msg.as_bytes());
    }

    fn handle_log(&mut self, task_name: String, content: String) {
        println!("[{}] LOG: {}", task_name, content);
    }

    fn handle_heartbeat(&self) {
        if let Some(id) = &self.worker_id {
            self.state.update_heartbeat(id);
        }
    }

    fn handle_status(&self, task_name: String, status: String) {
        println!("Tarea '{}' finalizada con estado: {}", task_name, status);
    }

    fn cleanup(&self) {
        if let Some(id) = &self.worker_id {
            self.state.remove_worker(id);
            println!("Worker {} desconectado.", id);
        }
    }
}