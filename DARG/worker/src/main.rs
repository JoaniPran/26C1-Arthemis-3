mod client;
mod executor;
mod heartbeat;

use std::env;

fn main() {
    let worker_id = env::args().nth(1).unwrap_or_else(|| {
        println!("No se especifico ID para el worker. Usando 'worker-default'.");
        "worker-default".to_string()
    });

    let server_addr = env::args()
        .nth(2)
        .unwrap_or_else(|| "100.103.147.37:8080".to_string());

    println!("Iniciando Worker: {}...", worker_id);
    println!("Intentando conectar al Coordinador en {}...", server_addr);

    client::run(worker_id, &server_addr);
}
