use std::fs;
use std::path::PathBuf;

pub fn upload_workflow_file(
    source_path: &PathBuf,
    username: &str,
    coordinator_ip: &str,
) -> Result<String, String> {
    let ext = source_path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase();

    if ext != "yaml" && ext != "yml" {
        return Err("Error: Formato no soportado. Solo se permiten archivos .yaml".to_string());
    }

    let file_name = source_path
        .file_name()
        .ok_or("Ruta de archivo inválida")?
        .to_string_lossy()
        .to_string();

    let file_bytes = fs::read(source_path)
        .map_err(|e| format!("Error al leer el archivo local: {}", e))?;

    let url = format!(
        "http://{}:8081/upload_workflow/{}/{}",
        coordinator_ip, username, file_name
    );

    let client = reqwest::blocking::Client::new();

    // Tipo explícito reqwest::blocking::Response para evitar error de inferencia
    let res: Result<reqwest::blocking::Response, reqwest::Error> = 
        client.post(&url).body(file_bytes).send();

    match res {
        Ok(response) if response.status().is_success() => Ok(file_name),
        Ok(response) => Err(format!("El servidor devolvió el código: {}", response.status())),
        Err(e) => Err(format!("Fallo al enviar el archivo al Coordinador: {}", e)),
    }
}