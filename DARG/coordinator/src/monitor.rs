use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::mpsc::Receiver;
use std::thread;

pub enum LogEvent {
    StartWorkflow(String),
    LogLine {
        workflow_name: String,
        task_name: String,
        content: String,
    },
}

pub fn start_global_log_monitor(rx: Receiver<LogEvent>) {
    thread::spawn(move || {
        let mut open_files: HashMap<String, File> = HashMap::new();

        while let Ok(event) = rx.recv() {
            match event {
                LogEvent::StartWorkflow(name) => {
                    let filename = format!("{}_logs.txt", name);
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&filename)
                    {
                        let _ = writeln!(file, "=== INICIANDO LOGS EN TIEMPO REAL: {} ===", name);
                        let _ = file.flush();
                        open_files.insert(name, file);
                    }
                }
                LogEvent::LogLine {
                    workflow_name,
                    task_name,
                    content,
                } => {
                    let file_entry = open_files.entry(workflow_name.clone()).or_insert_with(|| {
                        let filename = format!("{}_logs.txt", workflow_name);
                        OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&filename)
                            .expect("No se pudo abrir el archivo de logs")
                    });

                    let _ = writeln!(file_entry, "[{}] {}", task_name, content);
                    let _ = file_entry.flush();
                }
            }
        }
    });
}
