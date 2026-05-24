use std::fs;
use std::path::{Path, PathBuf};

pub fn copy_workflow_file(source_path: &PathBuf) -> Result<String, String> {
    let ext = source_path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase();

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
