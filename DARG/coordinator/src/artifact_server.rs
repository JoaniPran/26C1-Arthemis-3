use crate::db::Database;
use rouille::Response;
use std::fs::{self, File};

use rouille::input::json_input;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    user_id: i32,
    username: String,
}

#[derive(Deserialize)]
struct StartTaskRequest {
    task_id: i32,
}

#[derive(Deserialize)]
struct ResetWorkflowRequest {
    user_id: i32,
    workflow_name: String,
}

pub fn start_artifact_server(port: &str, database: Arc<Mutex<Database>>) {
    let addr = format!("0.0.0.0:{}", port);
    let artifacts_dir = "./artifacts";
    let workflows_dir = "./workflows";

    fs::create_dir_all(artifacts_dir).expect("Fallo al crear la carpeta de artefactos");
    fs::create_dir_all(workflows_dir).expect("Fallo al crear la carpeta de workflows");

    let certificate =
        fs::read("coordinator/certs/coordinator.pem").expect("No se pudo leer el certificado TLS");
    let private_key = fs::read("coordinator/certs/coordinator.key")
        .expect("No se pudo leer la clave privada TLS");

    println!(
        "Servidor HTTPS de Artefactos y Workflows escuchando en {}...",
        addr
    );

    let server = rouille::Server::new_ssl(
        addr,
        move |request| {
            let url = request.url();

            if request.method() == "GET" && url == "/workers_count" {
                let db = database.lock().unwrap();
                return match db.get_connected_worker_count() {
                    Ok(count) => Response::text(count.to_string()).with_status_code(200),
                    Err(e) => {
                        eprintln!("Error al consultar workers conectados: {}", e);
                        Response::text("0").with_status_code(500)
                    }
                };
            }

            if request.method() == "POST" && url.starts_with("/upload_workflow/") {
                let path_suffix = url.replace("/upload_workflow/", "");
                let filepath = format!("{}/{}", workflows_dir, path_suffix);

                if let Some(parent) = std::path::Path::new(&filepath).parent() {
                    let _ = fs::create_dir_all(parent);
                }

                let mut file = match File::create(&filepath) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Error al crear workflow {}: {}", filepath, e);
                        return Response::text("Error interno del servidor").with_status_code(500);
                    }
                };

                let mut body = match request.data() {
                    Some(b) => b,
                    None => return Response::text("Cuerpo HTTP vacío").with_status_code(400),
                };

                return match std::io::copy(&mut body, &mut file) {
                    Ok(_) => {
                        println!("Workflow guardado exitosamente: {}", filepath);
                        Response::text("Workflow subido con éxito").with_status_code(200)
                    }
                    Err(e) => {
                        eprintln!("Error al escribir workflow {}: {}", filepath, e);
                        Response::text("Error al guardar en disco").with_status_code(500)
                    }
                };
            }

            if request.method() == "POST" && request.url() == "/start_task" {
                let req_data: StartTaskRequest = match json_input(request) {
                    Ok(data) => data,
                    Err(_) => return Response::text("JSON inválido").with_status_code(400),
                };

                let db = database.lock().unwrap();
                return match db.set_task_pending(req_data.task_id) {
                    Ok(_) => Response::text("Tarea iniciada").with_status_code(200),
                    Err(e) => Response::text(e).with_status_code(400),
                };
            }

            if request.method() == "POST" && request.url() == "/reset_workflow" {
                let req_data: ResetWorkflowRequest = match json_input(request) {
                    Ok(data) => data,
                    Err(_) => return Response::text("JSON inválido").with_status_code(400),
                };

                let db = database.lock().unwrap();
                return match db.reset_workflow(req_data.user_id, req_data.workflow_name) {
                    Ok(_) => Response::text("Workflow reiniciado").with_status_code(200),
                    Err(_) => return Response::text("JSON inválido").with_status_code(400),
                };
            }

            if request.method() == "GET" && request.url().starts_with("/workflows/") {
                let url = request.url();
                let parts: Vec<&str> = url.split('/').skip(2).collect();
                if parts.len() == 1
                    && let Ok(user_id) = parts[0].parse::<i32>()
                {
                    let db = database.lock().unwrap();
                    match db.get_workflows_for_user(user_id) {
                        Ok(user_workflows) => return Response::json(&user_workflows),
                        Err(e) => {
                            eprintln!("Error de BDD al obtener workflows: {}", e);
                            return Response::text("Error al obtener workflows de la BDD")
                                .with_status_code(500);
                        }
                    }
                }

                return Response::text("Parámetro user_id inválido").with_status_code(400);
            }

            if request.method() == "GET" && url.starts_with("/tasks/") {
                let suffix = url.replace("/tasks/", "");
                let parts: Vec<&str> = suffix.splitn(2, '/').collect();

                if parts.len() == 2
                    && let Ok(user_id) = parts[0].parse::<i32>()
                {
                    let file_name = parts[1];
                    if let Ok(db) = Database::new("arthemis.db")
                        && let Ok(tasks) = db.get_tasks_for_ui(user_id, file_name)
                    {
                        // Devuelve la tupla (id, name, status, logs) directo como JSON
                        return Response::json(&tasks);
                    }
                }
                return Response::text("Error al obtener tareas").with_status_code(500);
            }

            if request.method() == "GET" && url.starts_with("/download/") {
                let filename = url.replace("/download/", "");
                let filepath = format!("{}/{}", artifacts_dir, filename);

                return match File::open(&filepath) {
                    Ok(file) => Response::from_file("application/zip", file),
                    Err(_) => Response::text("Archivo no encontrado").with_status_code(404),
                };
            }

            if request.method() == "POST" && request.url() == "/register" {
                let req_data: AuthRequest = match json_input(request) {
                    Ok(data) => data,
                    Err(_) => return Response::text("JSON inválido").with_status_code(400),
                };

                let db = database.lock().unwrap();
                return match db.register_user(&req_data.username, &req_data.password) {
                    Ok(user_id) => Response::json(&AuthResponse {
                        user_id,
                        username: req_data.username,
                    }),
                    Err(_) => Response::text("El usuario ya existe").with_status_code(400),
                };
            }

            if request.method() == "POST" && request.url() == "/login" {
                let req_data: AuthRequest = match json_input(request) {
                    Ok(data) => data,
                    Err(_) => return Response::text("JSON inválido").with_status_code(400),
                };

                let db = database.lock().unwrap();
                return match db.authenticate_user(&req_data.username, &req_data.password) {
                    Ok(user_id) => Response::json(&AuthResponse {
                        user_id,
                        username: req_data.username,
                    }),
                    Err(_) => Response::text("Credenciales incorrectas").with_status_code(401),
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
        },
        certificate,
        private_key,
    )
    .expect("No se pudo iniciar el servidor HTTPS de artefactos");

    server.run();
}
