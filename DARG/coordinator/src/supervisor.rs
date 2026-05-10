use std::thread;
use std::time::Duration;
use crate::state::CoordinatorState;

pub fn start_watchdog(state: CoordinatorState) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            
            let dead_workers = state.find_dead_workers(15);
            
            for worker_id in dead_workers {
                println!("SUPERVISOR: El worker {} no responde (timeout). Expulsando...", worker_id);
                state.remove_worker(&worker_id);
            }
        }
    });
}