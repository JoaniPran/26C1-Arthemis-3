use std::fs;
use std::path::{Path, PathBuf};

pub fn get_existing_workflows() -> Vec<String> {
    let mut workflows = Vec::new();
    let dest_dir = Path::new("workflows");

    let _ = fs::create_dir_all(dest_dir);

    if let Ok(entries) = fs::read_dir(dest_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().unwrap_or_default().to_str().unwrap_or_default().to_lowercase();
            
            if ext == "yaml" || ext == "yml" {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    workflows.push(name.to_string());
                }
            }
        }
    }
    workflows
}

pub fn copy_workflow_file(source_path: &PathBuf) -> Result<String, String> {
    let ext = source_path.extension().unwrap_or_default().to_str().unwrap_or_default().to_lowercase();
    
    if ext != "yaml" && ext != "yml" {
        return Err("Error: Formato no soportado. Solo se permiten archivos .yaml".to_string());
    }

    let dest_dir = Path::new("workflows");
    if let Err(e) = fs::create_dir_all(dest_dir) {
        return Err(format!("Error al crear carpeta destino: {}", e));
    }

    let file_name = source_path.file_name().ok_or("Ruta de archivo inválida")?;
    let destination = dest_dir.join(file_name);

    match fs::copy(source_path, destination) {
        Ok(_) => Ok(file_name.to_string_lossy().to_string()),
        Err(e) => Err(format!("Error interno al guardar el archivo: {}", e)),
    }
}