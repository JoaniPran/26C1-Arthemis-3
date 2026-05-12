use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    RegisterWorker {
        id: String,
    },
    TaskStatus {
        task_id: i32,
        status: String,
    },
    LogFragment {
        task_id: i32,
        content: String,
    },
    AssignTask {
        task_id: i32,
        task_name: String,
        command: String,
    },
    Heartbeat,
}
