use crate::SharedDatabase;
use crate::db::Database;
use crate::monitor::LogEvent;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct WorkflowYaml {
    pub name: String,
    pub tasks: Vec<TaskYaml>,
}

#[derive(Debug, Deserialize)]
pub struct TaskYaml {
    pub name: String,
    pub command: String,
    pub depends_on: Option<Vec<String>>,
}

pub fn start_workflow_watcher(database: SharedDatabase, folder_path: String, tx: Sender<LogEvent>) {
    thread::spawn(move || {
        let mut processed_files = std::collections::HashSet::new();

        loop {
            thread::sleep(Duration::from_secs(5));

            if let Ok(entries) = std::fs::read_dir(&folder_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "yaml") {
                        let path_str = path.to_str().unwrap().to_string();

                        if !processed_files.contains(&path_str) {
                            let db = database.lock().unwrap();

                            match load_from_yaml(&db, &path_str) {
                                Ok((workflow_id, workflow_name)) => {
                                    println!(
                                        "Watcher: Cargado '{}' (ID: {})",
                                        workflow_name, workflow_id
                                    );
                                    processed_files.insert(path_str);

                                    let _ = tx.send(LogEvent::StartWorkflow(workflow_name));
                                }
                                Err(e) => eprintln!("Watcher Error: {}", e),
                            }
                        }
                    }
                }
            }
        }
    });
}

fn load_from_yaml(db: &Database, file_path: &str) -> Result<(i32, String), Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;
    let workflow: WorkflowYaml = serde_yaml::from_str(&content)?;

    if let Ok(wf_id) = db.get_workflow_id(&workflow.name) {
        println!(
            "Parser: El workflow '{}' ya existe (ID: {}). Reiniciando tareas a PENDING y limpiando historial...",
            workflow.name, wf_id
        );

        db.reset_workflow(wf_id)?;

        return Ok((wf_id, workflow.name));
    }

    println!(
        "Parser: Cargando workflow '{}' con {} tareas...",
        workflow.name,
        workflow.tasks.len()
    );

    let wf_id = db.insert_workflow(&workflow.name)?;
    let mut name_to_id: HashMap<String, i32> = HashMap::new();

    for task in &workflow.tasks {
        let task_id = db.insert_task(wf_id, &task.name, &task.command)?;
        name_to_id.insert(task.name.clone(), task_id);
    }

    for task in &workflow.tasks {
        if let Some(deps) = &task.depends_on {
            let current_task_id = name_to_id
                .get(&task.name)
                .ok_or("Tarea no encontrada en mapa")?;

            for dep_name in deps {
                if let Some(dep_id) = name_to_id.get(dep_name) {
                    db.insert_dependency(*current_task_id, *dep_id)?;
                } else {
                    return Err(format!("Error de validación: La tarea '{}' depende de '{}', pero esa tarea no existe en el YAML.", task.name, dep_name).into());
                }
            }
        }
    }

    let name_cloned = workflow.name.clone();
    Ok((wf_id, name_cloned))
}
