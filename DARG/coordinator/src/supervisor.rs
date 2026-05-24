use crate::db::{Database, TaskStatus};
use crate::state::CoordinatorState;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn start_watchdog(state: CoordinatorState, database: Arc<Mutex<Database>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));

            let dead_workers = state.remove_dead_workers(15);

            for (worker_id, task_id_out) in dead_workers {
                println!(
                    "SUPERVISOR: El worker {} no responde (timeout). Expulsando...",
                    worker_id
                );

                if let Some(task_id) = task_id_out {
                    println!(
                        "SUPERVISOR: Reasignando tarea huérfana (ID: {}) a PENDING...",
                        task_id
                    );
                    let db = database.lock().unwrap();
                    if let Err(e) = db.update_task_status(task_id, TaskStatus::Pending) {
                        println!(
                            "SUPERVISOR: Error crítico al actualizar base de datos: {}",
                            e
                        );
                    }
                }
            }
        }
    });
}
