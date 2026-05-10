use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::io::Write;
use common::Message;

pub fn start_heartbeat_loop(mut stream: TcpStream) {
    thread::spawn(move || {
        loop {
            let heartbeat = Message::Heartbeat;
            let mut msg_str = serde_json::to_string(&heartbeat).unwrap();
            msg_str.push('\n');
            
            if stream.write_all(msg_str.as_bytes()).is_err() {
                break; 
            }
            thread::sleep(Duration::from_secs(5));
        }
    });
}