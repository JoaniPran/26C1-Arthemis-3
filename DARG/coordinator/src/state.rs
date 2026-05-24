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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn create_dummy_stream() -> TcpStream {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap()
    }

    #[test]
    fn test_add_and_remove_worker() {
        let state = CoordinatorState::new();
        state.add_worker("worker-1".to_string(), create_dummy_stream());

        let task_id = state.remove_worker("worker-1");
        assert_eq!(task_id, None);

        assert_eq!(state.remove_worker("worker-1"), None);
    }

    #[test]
    fn test_assign_worker_and_free() {
        let state = CoordinatorState::new();
        state.add_worker("worker-1".to_string(), create_dummy_stream());

        let assignment = state.assign_worker(100);
        assert!(assignment.is_some());
        assert_eq!(assignment.unwrap().0, "worker-1");

        let next_assignment = state.assign_worker(101);
        assert!(next_assignment.is_none());

        state.set_worker_free("worker-1");

        let final_assignment = state.assign_worker(101);
        assert!(final_assignment.is_some());
        assert_eq!(final_assignment.unwrap().0, "worker-1");
    }

    #[test]
    fn test_dead_worker_timeout() {
        let state = CoordinatorState::new();
        state.add_worker("worker-1".to_string(), create_dummy_stream());

        state.assign_worker(55);

        {
            let mut map = state.workers.lock().unwrap();
            if let Some(info) = map.get_mut("worker-1") {
                info.last_seen = Instant::now() - Duration::from_secs(20);
            }
        }

        let dead_workers = state.remove_dead_workers(15);

        assert_eq!(dead_workers.len(), 1);
        assert_eq!(dead_workers[0].0, "worker-1");
        assert_eq!(dead_workers[0].1, Some(55));
    }
}
