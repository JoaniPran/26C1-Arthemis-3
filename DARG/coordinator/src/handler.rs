use crate::db::{Database, TaskStatus};
use crate::monitor::LogEvent;
use crate::state::CoordinatorState;
use common::Message;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::{Arc, Mutex, mpsc::Sender};

pub struct WorkerHandler {
    stream: TcpStream,
    state: CoordinatorState,
    database: Arc<Mutex<Database>>,
    worker_id: Option<String>,
    log_tx: Sender<LogEvent>,
}

impl WorkerHandler {
    pub fn new(
        stream: TcpStream,
        state: CoordinatorState,
        database: Arc<Mutex<Database>>,
        log_tx: Sender<LogEvent>,
    ) -> Self {
        Self {
            stream,
            state,
            database,
            worker_id: None,
            log_tx,
        }
    }

    pub fn handle_connection(
        stream: TcpStream,
        state: CoordinatorState,
        database: Arc<Mutex<Database>>,
        log_tx: Sender<LogEvent>,
    ) {
        let mut handler = WorkerHandler::new(stream, state, database, log_tx);
        handler.run();
    }

    fn run(&mut self) {
        let read_stream = match self.stream.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };

        let reader = BufReader::new(read_stream);

        for line in reader.lines() {
            match line {
                Ok(text) => self.process_message(&text),
                Err(_) => break,
            }
        }

        self.cleanup();
    }

    fn process_message(&mut self, text: &str) {
        match serde_json::from_str::<Message>(text) {
            Ok(Message::RegisterWorker { id }) => self.handle_register(id),
            Ok(Message::Heartbeat) => self.handle_heartbeat(),
            Ok(Message::TaskStatus { task_id, status }) => self.handle_status(task_id, status),
            Ok(Message::LogFragment { task_id, content }) => self.handle_log(task_id, content),
            Ok(Message::AssignTask { .. }) => {}
            Err(e) => println!("Recibido mensaje malformado: {}. Contenido: {}", e, text),
        }
    }

    fn handle_register(&mut self, id: String) {
        self.worker_id = Some(id.clone());

        let task_id_opt = self.state.remove_worker(&id);

        if let Some(task_id) = task_id_opt {
            println!(
                "Worker {} reconectó antes del timeout. Reasignando tarea previa (ID: {}) a PENDING...",
                id, task_id
            );
            let db = self.database.lock().unwrap();
            let _ = db.update_task_status(task_id, TaskStatus::Pending);
        }

        self.state.add_worker(
            id.clone(),
            self.stream
                .try_clone()
                .expect("Error al clonar stream para registro"),
        );
        println!("Worker registrado: {}.", id);
    }

    fn handle_heartbeat(&self) {
        if let Some(id) = &self.worker_id {
            self.state.update_heartbeat(id);
        }
    }

    fn handle_status(&self, task_id: i32, status: String) {
        let final_status = if status == "Success" {
            TaskStatus::Success
        } else {
            TaskStatus::Failed
        };

        let db = self.database.lock().unwrap();
        let _ = db.update_task_status(task_id, final_status);

        drop(db);

        if let Some(id) = &self.worker_id {
            self.state.set_worker_free(id);
            println!(
                "Worker {} completó la Tarea ID: {} y ahora está libre.",
                id, task_id
            );
        }
    }

    fn handle_log(&mut self, task_id: i32, content: String) {
        let db = self.database.lock().unwrap();
        if let Err(e) = db.insert_log(task_id, &content) {
            println!("Error guardando log en BD (Tarea {}): {}", task_id, e);
        }

        if let Ok((task_name, workflow_name)) = db.get_task_info(task_id) {
            let _ = self.log_tx.send(LogEvent::LogLine {
                workflow_name,
                task_name,
                content,
            });
        }
    }

    fn cleanup(&self) {
        if let Some(id) = &self.worker_id {
            let task_id_opt = self.state.remove_worker(id);
            println!("Worker {} desconectado.", id);

            if let Some(task_id) = task_id_opt {
                println!("Reasignando tarea huérfana (ID: {}) a PENDING...", task_id);

                let db = self.database.lock().unwrap();
                let _ = db.update_task_status(task_id, TaskStatus::Pending);
            }
        }
    }
}
