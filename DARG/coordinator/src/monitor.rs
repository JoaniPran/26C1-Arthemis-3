use crate::SharedDatabase;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Duration;

pub fn start_workflow_log_monitor(
    database: SharedDatabase,
    workflow_id: i32,
    workflow_name: String,
) {
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
