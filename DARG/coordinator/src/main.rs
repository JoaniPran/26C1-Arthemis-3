mod db;
mod handler;
mod monitor;
mod parser;
mod state;
mod supervisor;

use common::Message;
use db::{Database, TaskStatus};
use handler::WorkerHandler;
use parser::start_workflow_watcher;
use state::CoordinatorState;
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

    let db_instance = Database::new("arthemis.db").expect("Fallo al crear la base de datos");
    db_instance
        .reset_all_running_tasks()
        .expect("Error al limpiar tareas");

    let database = Arc::new(Mutex::new(db_instance));
    let state = CoordinatorState::new();

    let folder = "./workflows".to_string();
    std::fs::create_dir_all(&folder).expect("No se pudo crear la carpeta");

    start_workflow_watcher(database.clone(), folder);

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
