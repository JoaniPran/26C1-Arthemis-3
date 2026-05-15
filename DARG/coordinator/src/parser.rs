use crate::db::Database;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

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

pub fn load_from_yaml(db: &Database, file_path: &str) -> Result<i32, Box<dyn Error>> {

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

    Ok(wf_id)
}
