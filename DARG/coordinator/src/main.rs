mod db;
mod handler;
mod state;
mod supervisor;

use common::Message;
use db::Database;
use handler::WorkerHandler;
use state::CoordinatorState;
use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
    println!("Base de datos lista y recuperada.");

    // ---------------------------------------------------------
    // ZONA DE SIMULACIÓN (Hasta que el compañero termine el parser)
    // ---------------------------------------------------------
    println!("Procesando flujo de trabajo...");
    match db_instance.insert_workflow("Pipeline_Ejemplo") {
        Ok(wf_id) => {
            let t1 = db_instance
                .insert_task(wf_id, "Compilar", "ping 127.0.0.1 -c 15")
                .unwrap();
            let t2 = db_instance
                .insert_task(wf_id, "Testear", "cargo test")
                .unwrap();
            let t3 = db_instance
                .insert_task(wf_id, "Deploy", "echo 'Success'")
                .unwrap();

            db_instance.insert_dependency(t2, t1).unwrap();
            db_instance.insert_dependency(t3, t2).unwrap();
            println!("Flujo inyectado con ID: {}", wf_id);
        }
        Err(_) => {
            if let Ok(wf_id) = db_instance.get_workflow_id("Pipeline_Ejemplo") {
                db_instance.reset_workflow(wf_id).unwrap();
                println!("Flujo 'Pipeline_Ejemplo' detectado. Reiniciando estados a PENDING.");
            }
        }
    }

    let database = Arc::new(Mutex::new(db_instance));

    let listener = TcpListener::bind(&addr).expect("No se pudo bindear el puerto.");
    let state = CoordinatorState::new();

    supervisor::start_watchdog(state.clone(), database.clone());

    let db_dispatcher = database.clone();
    let state_dispatcher = state.clone();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(2));

            let db = db_dispatcher.lock().unwrap();
            if let Ok(ready_tasks) = db.get_ready_tasks() {
                for task in ready_tasks {
                    if let Some((worker_id, mut stream)) = state_dispatcher.assign_worker(task.id) {
                        println!("Asignando tarea '{}' al worker {}", task.name, worker_id);

                        let _ = db.update_task_status(task.id, "RUNNING");

                        let assign_msg = Message::AssignTask {
                            task_name: task.name.clone(),
                            command: task.command.clone(),
                        };
                        let mut json_msg = serde_json::to_string(&assign_msg).unwrap();
                        json_msg.push('\n');
                        let _ = stream.write_all(json_msg.as_bytes());
                    } else {
                        break;
                    }
                }
            }
        }
    });

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
