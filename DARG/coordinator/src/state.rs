use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub struct WorkerInfo {
    pub stream: TcpStream,
    pub last_seen: Instant,
}

#[derive(Clone)]
pub struct CoordinatorState {
    workers: Arc<Mutex<HashMap<String, WorkerInfo>>>,
}

impl CoordinatorState {
    pub fn new() -> Self {
        Self {
            workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_worker(&self, id: String, stream: TcpStream) {
        let mut map = self.workers.lock().unwrap();
        map.insert(id, WorkerInfo {
            stream: stream,
            last_seen: Instant::now(), 
        });
    }

    pub fn remove_worker(&self, id: &str) {
        let mut map = self.workers.lock().unwrap();
        map.remove(id);
    }

    pub fn update_heartbeat(&self, id: &str) {
        let mut map = self.workers.lock().unwrap();
        
        if let Some(info) = map.get_mut(id) {
            info.last_seen = Instant::now();
        }
    }

    pub fn find_dead_workers(&self, timeout_secs: u64) -> Vec<String> {
        let map = self.workers.lock().unwrap();
        let timeout = Duration::from_secs(timeout_secs);
        let now = Instant::now();
        
        map.iter()
            .filter_map(|(id, info)| {
                if now.duration_since(info.last_seen) > timeout {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}