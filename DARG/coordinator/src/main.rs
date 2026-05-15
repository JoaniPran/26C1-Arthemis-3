mod db;
mod handler;
mod parser;
mod state;
mod supervisor;

use common::Message;
use db::{Database, TaskStatus};
use handler::WorkerHandler;
use state::CoordinatorState;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

type SharedDatabase = Arc<Mutex<Database>>;

fn main() {
    let port = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "8080".to_string());
    start_server(&port);
}

fn start_server(port: &str) {
    let addr = format!("127.0.0.1:{}", port);

    println!("Inicializando base de datos SQLite...");
    let db_instance = Database::new("arthemis.db").expect("Fallo al crear la base de datos");
    db_instance
        .reset_all_running_tasks()
        .expect("Error al limpiar tareas huérfanas");

    let workflow_id = match parser::load_from_yaml(&db_instance, "workflow_ejemplo_1.yaml") {
        Ok(id) => {
            println!("YAML cargado exitosamente. ID: {}", id);
            id
        }
        Err(e) => {
            println!(
                "Error cargando YAML: {}. Usando ID por defecto o saliendo.",
                e
            );
           1 //manejar el error de forma más agresiva
        }
    };

    let database = Arc::new(Mutex::new(db_instance));
    let state = CoordinatorState::new();

    start_workflow_log_monitor(
        database.clone(),
        workflow_id,
        "Pipeline_Ejemplo".to_string(),
    );

    supervisor::start_watchdog(state.clone(), database.clone());
    start_dispatcher(state.clone(), database.clone());

    let listener = TcpListener::bind(&addr).expect("No se pudo bindear el puerto.");
    println!("Coordinador Arthemis 3 escuchando en {}...", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let state_clone = state.clone();
                let db_clone = database.clone();

                thread::spawn(move || {
                    WorkerHandler::handle_connection(s, state_clone, db_clone);
                });
            }
            Err(e) => println!("Error de conexión: {}.", e),
        }
    }
}

fn start_dispatcher(state: CoordinatorState, database: SharedDatabase) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(2));

            let db = database.lock().unwrap();

            match db.get_ready_tasks() {
                Ok(ready_tasks) => {
                    for task in ready_tasks {
                        if let Some((worker_id, mut stream)) = state.assign_worker(task.id) {
                            println!("Asignando tarea '{}' al worker {}", task.name, worker_id);

                            let _ = db.update_task_status(task.id, TaskStatus::Running);

                            let assign_msg = Message::AssignTask {
                                task_id: task.id,
                                task_name: task.name.clone(),
                                command: task.command.clone(),
                            };

                            if let Ok(mut json_msg) = serde_json::to_string(&assign_msg) {
                                json_msg.push('\n');

                                if stream.write_all(json_msg.as_bytes()).is_err() {
                                    println!(
                                        "Fallo al enviar la tarea '{}' al worker {}. Revertiendo...",
                                        task.name, worker_id
                                    );
                                    let _ = db.update_task_status(task.id, TaskStatus::Pending);
                                    state.set_worker_free(&worker_id);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("Error interno al consultar tareas listas: {}", e);
                }
            }
        }
    });
}

fn start_workflow_log_monitor(database: SharedDatabase, workflow_id: i32, workflow_name: String) {
    thread::spawn(move || {
        let filename = format!("{}_logs.txt", workflow_name);
        let mut last_id = 0;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&filename)
            .expect("No se pudo crear el archivo de logs");

        let _ = writeln!(
            file,
            "=== INICIANDO LOGS EN TIEMPO REAL: {} ===",
            workflow_name
        );

        loop {
            thread::sleep(Duration::from_millis(500));

            let db = database.lock().unwrap();

            if let Ok(new_logs) = db.get_new_workflow_logs(workflow_id, last_id) {
                if !new_logs.is_empty() {
                    for (log_id, task_name, content) in new_logs {
                        let _ = writeln!(file, "[{}] {}", task_name, content);

                        last_id = log_id;
                    }
                    let _ = file.flush();
                }
            }
        }
    });
}
