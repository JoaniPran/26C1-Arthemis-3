use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Message {
    RegisterWorker {
        id: String,
    },
    TaskStatus {
        task_id: i32,
        status: String,
        output_artifact: Option<Vec<u8>>,
    },
    LogFragment {
        task_id: i32,
        content: String,
    },
    AssignTask {
        task_id: i32,
        task_name: String,
        command: String,
        input_artifact: Option<Vec<u8>>,
        artifact_path: Option<String>,
    },
    Heartbeat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_heartbeat() {
        let msg = Message::Heartbeat;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, "\"Heartbeat\"");

        let decoded: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_serialize_deserialize_assign_task() {
        let msg = Message::AssignTask {
            task_id: 42,
            task_name: "build".to_string(),
            command: "cargo build".to_string(),
            input_artifact: Some(vec![1, 2, 3, 4]),
            artifact_path: Some("taller".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("AssignTask"));
        assert!(json.contains("42"));
        assert!(json.contains("cargo build"));

        let decoded: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }
}
