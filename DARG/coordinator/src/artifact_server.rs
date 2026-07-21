use std::fs::{self, File};
use std::io::copy;
use std::path::Path;
use tiny_http::{Header, Method, Response, ResponseBox, Server};

pub fn start_artifact_server(port: &str) {
    let addr = format!("0.0.0.0:{}", port);
    let artifacts_dir = "./artifacts";

    fs::create_dir_all(artifacts_dir).expect("Fallo al crear la carpeta de artefactos");

    println!("Servidor HTTP de Artefactos escuchando en {}...", addr);

    let server = Server::http(&addr).expect("No se pudo iniciar el servidor de artefactos");

    let content_type_header = Header::from_bytes(&b"Content-Type"[..], &b"application/zip"[..])
        .expect("Header Content-Type inválido");

    for mut request in server.incoming_requests() {
        let url = request.url().to_string();

        if request.method() == &Method::Get && url.starts_with("/download/") {
            let filename = url.replace("/download/", "");
            let filepath = format!("{}/{}", artifacts_dir, filename);

            let response: ResponseBox = match File::open(Path::new(&filepath)) {
                Ok(file) => Response::from_file(file)
                    .with_header(content_type_header.clone())
                    .boxed(),
                Err(_) => Response::from_string("Archivo no encontrado")
                    .with_status_code(404)
                    .boxed(),
            };

            let _ = request.respond(response);
            continue;
        }

        if request.method() == &Method::Post && url.starts_with("/upload/") {
            let filename = url.replace("/upload/", "");
            let filepath = format!("{}/{}", artifacts_dir, filename);

            let mut file = match File::create(&filepath) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error al crear archivo {}: {}", filepath, e);
                    let _ = request.respond(
                        Response::from_string("Error interno del servidor").with_status_code(500),
                    );
                    continue;
                }
            };

            let response = match copy(request.as_reader(), &mut file) {
                Ok(_) => {
                    println!("Artefacto guardado exitosamente: {}", filename);
                    Response::from_string("Subida exitosa").with_status_code(200)
                }
                Err(e) => {
                    eprintln!("Error al escribir archivo {}: {}", filepath, e);
                    Response::from_string("Error al guardar en disco").with_status_code(500)
                }
            };

            let _ = request.respond(response);
            continue;
        }

        let _ = request.respond(Response::from_string("Ruta HTTP no válida").with_status_code(404));
    }
}
