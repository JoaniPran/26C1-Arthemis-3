use std::fmt;

#[derive(Debug, PartialEq)]
pub enum TaskStatus {
    Sleeping,
    Pending,
    Running,
    Success,
    Failed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskStatus::Sleeping => write!(f, "SLEEPING"),
            TaskStatus::Pending => write!(f, "PENDING"),
            TaskStatus::Running => write!(f, "RUNNING"),
            TaskStatus::Success => write!(f, "SUCCESS"),
            TaskStatus::Failed => write!(f, "FAILED"),
        }
    }
}

#[derive(Debug)]
pub struct TaskRecord {
    pub id: i32,
    pub name: String,
    pub command: String,
    pub produces: Option<String>,
}
