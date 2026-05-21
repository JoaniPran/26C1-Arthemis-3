// src/backend.rs

use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

// 1. Definimos el "Sobre" (Los mensajes que el backend le enviará a la UI)
pub enum BackendMessage {
    // Mensaje para enviar un nuevo log de la terminal
    NewLog(String),
    // Mensaje para avisar que una tarea cambió de estado (ej: de PENDING a RUNNING)
    TaskStatusChanged { task_id: i32, status: String },
}

// 2. Simulamos tu motor de ejecución en un hilo separado
pub fn start_mock_backend(tx: Sender<BackendMessage>) {
    thread::spawn(move || {
        // Simulamos un retraso inicial mientras el sistema arranca
        thread::sleep(Duration::from_secs(2));

        // Enviamos el primer log a la interfaz
        let _ = tx.send(BackendMessage::NewLog("[09:30:00] [System] Coordinador iniciado exitosamente.".to_string()));
        thread::sleep(Duration::from_secs(1));

        let _ = tx.send(BackendMessage::NewLog("[09:30:01] [Pipeline_Ejemplo] Iniciando ejecución...".to_string()));
        
        // Simulamos un log de tarea cada 2 segundos
        for i in 1..=5 {
            thread::sleep(Duration::from_secs(2));
            let _ = tx.send(BackendMessage::NewLog(format!(
                "[09:30:0{}] [Ping_Check] PING 127.0.0.1: icmp_seq={} ttl=64 time=0.123 ms",
                i + 1, i
            )));
        }

        let _ = tx.send(BackendMessage::NewLog("[09:30:10] [Ping_Check] [Status: SUCCESS]".to_string()));
    });
}