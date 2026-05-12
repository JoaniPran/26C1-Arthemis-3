use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct WorkerInfo {
    pub stream: TcpStream,
    pub last_seen: Instant,
    pub assigned_task_id: Option<i32>,
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
        map.insert(
            id,
            WorkerInfo {
                stream: stream,
                last_seen: Instant::now(),
                assigned_task_id: None,
            },
        );
    }

    pub fn remove_worker(&self, id: &str) -> Option<i32> {
        let mut map = self.workers.lock().unwrap();
        map.remove(id).and_then(|info| info.assigned_task_id)
    }

    pub fn update_heartbeat(&self, id: &str) {
        let mut map = self.workers.lock().unwrap();
        if let Some(info) = map.get_mut(id) {
            info.last_seen = Instant::now();
        }
    }

    pub fn assign_worker(&self, task_id: i32) -> Option<(String, TcpStream)> {
        let mut map = self.workers.lock().unwrap();
        for (id, info) in map.iter_mut() {
            if info.assigned_task_id.is_none() {
                if let Ok(stream_clone) = info.stream.try_clone() {
                    info.assigned_task_id = Some(task_id);
                    return Some((id.clone(), stream_clone));
                }
            }
        }
        None
    }

    pub fn set_worker_free(&self, id: &str) {
        let mut map = self.workers.lock().unwrap();
        if let Some(info) = map.get_mut(id) {
            info.assigned_task_id = None;
        }
    }

    pub fn remove_dead_workers(&self, timeout_secs: u64) -> Vec<(String, Option<i32>)> {
        let mut map = self.workers.lock().unwrap();
        let timeout = Duration::from_secs(timeout_secs);
        let now = Instant::now();

        let mut dead = Vec::new();
        map.retain(|id, info| {
            if now.duration_since(info.last_seen) > timeout {
                dead.push((id.clone(), info.assigned_task_id));
                false
            } else {
                true
            }
        });
        dead
    }
}
