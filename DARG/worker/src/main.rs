mod executor;
mod heartbeat;
mod client;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let worker_id = args.get(1).cloned().unwrap_or_else(|| "worker-default".to_string());

    println!("Starting Worker: {}", worker_id);
    client::run(worker_id, "127.0.0.1:8080");
}
