use rouille::Response;
use std::fs::{self, File};

pub fn start_artifact_server(port: &str) {
    let addr = format!("0.0.0.0:{}", port);
    let artifacts_dir = "./artifacts";

    fs::create_dir_all(artifacts_dir).expect("Fallo al crear la carpeta de artefactos");

    println!("Servidor HTTP de Artefactos escuchando en {}...", addr);

    rouille::start_server(addr, move |request| {
        let url = request.url();

        if request.method() == "GET" && url.starts_with("/download/") {
            let filename = url.replace("/download/", "");
            let filepath = format!("{}/{}", artifacts_dir, filename);

            return match File::open(&filepath) {
                Ok(file) => Response::from_file("application/zip", file),
                Err(_) => Response::text("Archivo no encontrado").with_status_code(404),
            };
        }

        if request.method() == "POST" && url.starts_with("/upload/") {
            let filename = url.replace("/upload/", "");
            let filepath = format!("{}/{}", artifacts_dir, filename);

            if let Some(parent) = std::path::Path::new(&filepath).parent() {
                let _ = fs::create_dir_all(parent);
            }

            let mut file = match File::create(&filepath) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error al crear archivo {}: {}", filepath, e);
                    return Response::text("Error interno del servidor").with_status_code(500);
                }
            };

            let mut body = request
                .data()
                .expect("Fallo al leer el cuerpo del request HTTP");
            
            return match std::io::copy(&mut body, &mut file) {
                Ok(_) => {
                    println!("Artefacto guardado exitosamente: {}", filename);
                    Response::text("Subida exitosa").with_status_code(200)
                }
                Err(e) => {
                    eprintln!("Error al escribir archivo {}: {}", filepath, e);
                    Response::text("Error al guardar en disco").with_status_code(500)
                }
            };
        }

        Response::text("Ruta HTTP no válida").with_status_code(404)
    });
}
