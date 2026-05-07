use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    RegisterWorker { id: String },
    TaskStatus { task_name: String, status: String },
    LogFragment { task_name: String, content: String },

    AssignTask { task_name: String, command: String },
    Heartbeat
}