use coordinator::server::start_server;

fn main() {
    let port = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "8080".to_string());
    start_server(&port, None);
}
