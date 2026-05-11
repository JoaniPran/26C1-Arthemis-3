use crate::db::Database;
use crate::state::CoordinatorState;
use common::Message;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct WorkerHandler {
    stream: TcpStream,
    state: CoordinatorState,
    database: Arc<Mutex<Database>>,
    worker_id: Option<String>,
}

impl WorkerHandler {
    pub fn new(stream: TcpStream, state: CoordinatorState, database: Arc<Mutex<Database>>) -> Self {
        Self {
            stream,
            state,
            database,
            worker_id: None,
        }
    }

    pub fn handle_connection(
        stream: TcpStream,
        state: CoordinatorState,
        database: Arc<Mutex<Database>>,
    ) {
        let handler = WorkerHandler::new(stream, state, database);
        handler.run();
    }

    fn run(mut self) {
        let read_stream = self
            .stream
            .try_clone()
            .expect("No se pudo clonar el stream para lectura");
        let reader = BufReader::new(read_stream);

        for line in reader.lines() {
            match line {
                Ok(text) => match serde_json::from_str::<Message>(&text) {
                    Ok(msg) => self.process_message(msg),
                    Err(e) => println!(
                        "Error crítico parseando JSON: {}. Texto recibido: {}",
                        e, text
                    ),
                },
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

        let task_id_opt = self.state.remove_worker(&id);

        if let Some(task_id) = task_id_opt {
            println!(
                "MANEJADOR: Worker {} reconectó antes del timeout. Reasignando tarea previa (ID: {}) a PENDING...",
                id, task_id
            );
            let db = self.database.lock().unwrap();
            let _ = db.update_task_status(task_id, "PENDING");
        }

        self.state.add_worker(
            id.clone(),
            self.stream.try_clone().expect("Error al clonar stream"),
        );
        println!("Worker registrado: {}.", id);
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

        let final_status = if status == "Success" {
            "SUCCESS"
        } else {
            "FAILED"
        };
        let db = self.database.lock().unwrap();
        let _ = db.update_task_status_by_name(&task_name, final_status);
        drop(db);

        if let Some(id) = &self.worker_id {
            self.state.set_worker_free(id);
            println!("Worker {} ahora está libre.", id);
        }
    }

    fn cleanup(&self) {
        if let Some(id) = &self.worker_id {
            let task_id_opt = self.state.remove_worker(id);
            println!("Worker {} desconectado.", id);

            if let Some(task_id) = task_id_opt {
                println!(
                    "MANEJADOR: Reasignando tarea huérfana (ID: {}) a PENDING...",
                    task_id
                );
                let db = self.database.lock().unwrap();
                let _ = db.update_task_status(task_id, "PENDING");
            }
        }
    }
}
