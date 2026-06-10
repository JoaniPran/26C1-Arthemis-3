use crate::SharedDatabase;
use crate::db::Database;
use crate::monitor::LogEvent;
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize, Clone)]
pub struct WorkflowYaml {
    pub name: String,
    pub tasks: Vec<TaskYaml>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskYaml {
    pub name: String,
    pub command: String,
    pub depends_on: Option<Vec<String>>,
    pub produces: Option<String>,
    pub consumes: Option<Vec<String>>,
}

pub fn start_workflow_watcher(
    database: SharedDatabase,
    folder_path: String,
    tx: Sender<LogEvent>,
    ui_tx: Option<Sender<String>>,
) {
    thread::spawn(move || {
        let mut processed_files: HashMap<String, SystemTime> = HashMap::new();

        let db_lock = database.lock().unwrap();
        if let Ok(known_workflows) = db_lock.get_all_workflows() {
            for (file_name, _) in known_workflows {
                let path_str = format!("{}/{}", folder_path, file_name);
                let path = std::path::Path::new(&path_str);

                if let Ok(metadata) = std::fs::metadata(path) {
                    let modified_time = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    processed_files.insert(path_str, modified_time);
                }
            }
        }
        drop(db_lock);

        loop {
            thread::sleep(Duration::from_secs(2));

            if let Ok(entries) = std::fs::read_dir(&folder_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .is_some_and(|ext| ext == "yaml" || ext == "yml")
                    {
                        let path_str = path.to_str().unwrap().to_string();

                        if let Ok(metadata) = std::fs::metadata(&path) {
                            let modified_time =
                                metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                            let last_processed_time = processed_files
                                .get(&path_str)
                                .copied()
                                .unwrap_or(SystemTime::UNIX_EPOCH);

                            if modified_time > last_processed_time {
                                let db = database.lock().unwrap();

                                match load_from_yaml(&db, &path_str) {
                                    Ok((workflow_id, file_name, display_name)) => {
                                        println!(
                                            "Watcher: Cargado '{}' desde '{}' (ID: {})",
                                            display_name, file_name, workflow_id
                                        );

                                        if let Some(ui_tx_s) = &ui_tx {
                                            let _ = ui_tx_s.send(format!(
                                                "LOADED:{}:{}",
                                                file_name, display_name
                                            ));
                                        }

                                        processed_files.insert(path_str.clone(), modified_time);
                                        let _ = tx.send(LogEvent::StartWorkflow(display_name));
                                    }
                                    Err(e) => {
                                        eprintln!("Watcher Error al procesar {}: {}", path_str, e);

                                        if let Some(ui_tx_s) = &ui_tx
                                            && let Some(file_name_os) = path.file_name()
                                            && let Some(file_name) = file_name_os.to_str()
                                        {
                                            let _ =
                                                ui_tx_s.send(format!("ERROR:{}:{}", file_name, e));
                                        }

                                        processed_files.insert(path_str.clone(), modified_time);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

fn load_from_yaml(db: &Database, file_path: &str) -> Result<(i32, String, String), Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;
    let workflow: WorkflowYaml = serde_yaml::from_str(&content)?;
    let file_name = Path::new(file_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    validate_workflow(&workflow)?;

    if let Ok(wf_id) = db.get_workflow_id(&file_name) {
        println!("Parser: Actualizando archivo modificado '{}'...", file_name);
        db.delete_workflows(wf_id)?;
    }

    println!(
        "Parser: Cargando workflow '{}' con {} tareas...",
        workflow.name,
        workflow.tasks.len()
    );

    let wf_id = save_new_workflow(db, &file_name, &workflow)?;

    Ok((wf_id, file_name, workflow.name))
}

fn validate_workflow(workflow: &WorkflowYaml) -> Result<(), Box<dyn Error>> {
    let grafo = create_grafo(workflow);

    if let Err(cycle_error) = toposort(&grafo, None) {
        let node_index = cycle_error.node_id();
        let node_name = grafo[node_index];

        return Err(format!(
            "El workflow '{}' fue rechazado: contiene dependencias circulares cerca de la tarea '{}'.", 
            workflow.name, node_name
        ).into());
    }

    Ok(())
}

fn save_new_workflow(
    db: &Database,
    file_name: &str,
    workflow: &WorkflowYaml,
) -> Result<i32, Box<dyn Error>> {
    let wf_id = db.insert_workflow(file_name, &workflow.name)?;
    let mut name_to_id: HashMap<String, i32> = HashMap::new();

    for task in &workflow.tasks {
        let task_id = db.insert_task(wf_id, &task.name, &task.command, task.produces.as_deref())?;
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
                    return Err(format!(
                        "Error de validación: La tarea '{}' depende de '{}', pero esa tarea no existe en el YAML.", 
                        task.name, dep_name
                    ).into());
                }
            }
        }
    }

    for task in &workflow.tasks {
        if let Some(consumes_list) = &task.consumes {
            let current_task_id = name_to_id
                .get(&task.name)
                .ok_or("Tarea no encontrada en mapa")?;

            for consumed_name in consumes_list {
                if let Some(consumed_id) = name_to_id.get(consumed_name) {
                    db.insert_consumption(*current_task_id, *consumed_id)?;
                } else {
                    return Err(format!(
                        "Error de validación: La tarea '{}' consume '{}', pero no existe.", 
                        task.name, consumed_name
                    ).into());
                }
            }
        }
    }

    Ok(wf_id)
}

fn create_grafo(workflow: &WorkflowYaml) -> DiGraph<&str, ()> {
    let mut grafo = DiGraph::<&str, ()>::new();
    let mut nodos = std::collections::HashMap::new();

    for task in &workflow.tasks {
        let nodo = grafo.add_node(task.name.as_str());
        nodos.insert(task.name.as_str(), nodo);
    }

    for task in &workflow.tasks {
        if let Some(deps) = &task.depends_on {
            for dep_name in deps {
                if let (Some(&nodo_actual), Some(&nodo_dep)) =
                    (nodos.get(task.name.as_str()), nodos.get(dep_name.as_str()))
                {
                    grafo.add_edge(nodo_dep, nodo_actual, ());
                } else {
                    eprintln!(
                        "Error: La tarea '{}' depende de '{}', pero esa tarea no existe en el YAML.",
                        task.name, dep_name
                    );
                }
            }
        }
    }

    grafo
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_task_yaml(nombre: &str, dependencias: Option<Vec<&str>>) -> TaskYaml {
        TaskYaml {
            name: nombre.to_string(),
            command: "comando_falso".to_string(),
            depends_on: dependencias.map(|vec| vec.into_iter().map(|s| s.to_string()).collect()),
            produces: None,
            consumes: None,
        }
    }

    #[test]
    fn test_valid_graph_validation() {
        let workflow = WorkflowYaml {
            name: "Grafo Valido".to_string(),
            tasks: vec![
                create_task_yaml("A", None),
                create_task_yaml("B", Some(vec!["A"])),
                create_task_yaml("C", Some(vec!["A"])),
                create_task_yaml("D", Some(vec!["B", "C"])),
            ],
        };

        assert!(validate_workflow(&workflow).is_ok());
    }

    #[test]
    fn test_circular_dependency_deadlock() {
        let workflow = WorkflowYaml {
            name: "Grafo Invalido".to_string(),
            tasks: vec![
                create_task_yaml("A", Some(vec!["B"])),
                create_task_yaml("B", Some(vec!["C"])),
                create_task_yaml("C", Some(vec!["A"])),
            ],
        };

        let result = validate_workflow(&workflow);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("dependencias circulares"));
    }

    #[test]
    fn test_phantom_dependency() {
        let workflow = WorkflowYaml {
            name: "Grafo Roto".to_string(),
            tasks: vec![create_task_yaml("A", Some(vec!["TareaQueNoExiste"]))],
        };

        let result = validate_workflow(&workflow);
        assert!(result.is_ok());
    }
}
