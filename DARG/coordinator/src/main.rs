use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{BufReader, Write};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use common::Message;
use serde_json::Deserializer;

// Definimos el tipo para nuestro registro de Workers
type WorkerMap = Arc<Mutex<HashMap<String, TcpStream>>>;

fn handle_worker(mut stream: TcpStream, workers: WorkerMap) { // 1. Agregamos 'mut' aquí
    // 2. Clonamos el stream para la lectura. Esto libera al 'stream' original para escribir.
    let read_stream = stream.try_clone().expect("No se pudo clonar el stream para lectura");
    let reader = BufReader::new(read_stream);
    
    let mut stream_iter = Deserializer::from_reader(reader).into_iter::<Message>();
    let mut current_worker_id = String::new();

    while let Some(Ok(msg)) = stream_iter.next() {
        match msg {
            Message::RegisterWorker { id } => {
                current_worker_id = id.clone();
                let mut map = workers.lock().unwrap();
                // Guardamos una copia del stream en el mapa para uso futuro del Coordinador
                map.insert(id.clone(), stream.try_clone().expect("Error al guardar en el registro"));
                
                println!("✅ Worker registrado: {}.", id);

                // PRUEBA DE LOGS: Enviamos una tarea inmediata al registrarse
                let assign_msg = Message::AssignTask { 
                    /*task_name: "Prueba_Logs".to_string(), 
                    command: "cargo test".to_string() */
                    task_name: "Prueba_Streaming".to_string(), 
                    command: "ping 127.0.0.1 -n 6".to_string() // Enviará 6 pings, uno por segundo
                };
                let json_msg = serde_json::to_string(&assign_msg).unwrap();
                // 3. Ahora esto funcionará porque 'stream' es mutable y no está bloqueado por el lector
                stream.write_all(json_msg.as_bytes()).ok(); 
            }
            Message::LogFragment { task_name, content } => {
                println!("[{}] LOG: {}", task_name, content);
            }
            _ => println!("Mensaje recibido no manejado."),
        }
    }

    // Limpieza al desconectarse para evitar fugas de hilos (Criterio de Aceptación)
    if !current_worker_id.is_empty() {
        let mut map = workers.lock().unwrap();
        map.remove(&current_worker_id);
        println!("❌ Worker {} desconectado.", current_worker_id);
    }
}

fn main() {
    // Tarea: Puerto configurable (puedes usar variables de entorno o argumentos)
    let port = std::env::args().nth(1).unwrap_or_else(|| "8080".to_string());
    let addr = format!("127.0.0.1:{}", port);
    
    let listener = TcpListener::bind(&addr).expect("No se pudo bindear el puerto.");
    let workers: WorkerMap = Arc::new(Mutex::new(HashMap::new()));

    println!("🚀 Coordinador Arthemis 3 escuchando en {}...", addr);

    // Tarea: Bucle de aceptación que no bloquea a los demás clientes
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let workers_clone = Arc::clone(&workers);
                // Tarea: Spawn de hilo nativo por cada conexión
                thread::spawn(move || handle_worker(s, workers_clone));
            }
            Err(e) => println!("Error de conexión: {}.", e),
        }
    }
}