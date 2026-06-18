use crate::db::Database;
use crate::state::CoordinatorState;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn start_watchdog(
    state: CoordinatorState,
    database: Arc<Mutex<Database>>,
    ui_tx: Option<Sender<String>>,
) {
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
                    // let _ = db.clear_task_logs(task_id);

                    if let Err(e) = db.set_task_pending(task_id) {
                        println!(
                            "SUPERVISOR: Error crítico al actualizar base de datos: {}",
                            e
                        );
                    } else {
                        if let Some(tx) = &ui_tx {
                            let _ = tx.send(format!("STATUS:{}:PENDING", task_id));
                        }
                    }
                }
            }
        }
    });
}
