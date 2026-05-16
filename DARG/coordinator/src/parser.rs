use crate::db::Database;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

use std::thread;
use std::time::Duration;

use crate::SharedDatabase;
use crate::monitor;

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

pub fn load_from_yaml(db: &Database, file_path: &str) -> Result<(i32, String), Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;
    let workflow: WorkflowYaml = serde_yaml::from_str(&content)?;

    println!(
        "Parser: Cargando workflow '{}' con {} tareas...",
        workflow.name,
        workflow.tasks.len()
    );

    let wf_id = db.insert_workflow(&workflow.name)?;
    // me guardo en un hashmap el nombre de la tarea y el id, para poder agregarle las dependencias en la bdd.
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

pub fn start_workflow_watcher(database: SharedDatabase, folder_path: String) {
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

                                    monitor::start_workflow_log_monitor(
                                        database.clone(),
                                        workflow_id,
                                        workflow_name,
                                    );
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
