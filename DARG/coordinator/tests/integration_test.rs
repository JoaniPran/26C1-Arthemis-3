use common::Message;
use coordinator::db::Database;
use coordinator::handler::WorkerHandler;
use coordinator::monitor::LogEvent;
use coordinator::state::CoordinatorState;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

fn setup_integration_env() -> (
    Arc<Mutex<Database>>,
    CoordinatorState,
    mpsc::Sender<LogEvent>,
    mpsc::Receiver<LogEvent>,
    i32,
) {
    let db = Database::new(":memory:").unwrap();
    let wf_id = db
        .insert_workflow("integration.yaml", "Integration_Pipeline")
        .unwrap();
    let task_id = db
        .insert_task(wf_id, "Integration_Task", "echo test")
        .unwrap();

    let db_arc = Arc::new(Mutex::new(db));
    let state = CoordinatorState::new();
    let (tx, rx) = mpsc::channel();

    (db_arc, state, tx, rx, task_id)
}

#[test]
fn test_worker_handler_full_integration() {
    let (db, state, log_tx, log_rx, task_id) = setup_integration_env();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let mut client_stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
    let (server_stream, _) = listener.accept().unwrap();

    let state_clone = state.clone();
    let db_clone = db.clone();

    thread::spawn(move || {
        WorkerHandler::handle_connection(server_stream, state_clone, db_clone, log_tx);
    });

    let msg_reg = Message::RegisterWorker {
        id: "test-worker-1".to_string(),
    };
    let mut reg_json = serde_json::to_string(&msg_reg).unwrap();
    reg_json.push('\n');
    client_stream.write_all(reg_json.as_bytes()).unwrap();

    thread::sleep(Duration::from_millis(100));

    let assign_res = state.assign_worker(task_id);
    assert!(assign_res.is_some());
    assert_eq!(assign_res.unwrap().0, "test-worker-1");

    let msg_log = Message::LogFragment {
        task_id,
        content: "Ejecutando proceso...".to_string(),
    };
    let mut log_json = serde_json::to_string(&msg_log).unwrap();
    log_json.push('\n');
    client_stream.write_all(log_json.as_bytes()).unwrap();

    let received_event = log_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("Timeout: El evento de log no llegó al canal");
    match received_event {
        LogEvent::LogLine {
            workflow_name,
            task_name,
            content,
        } => {
            assert_eq!(workflow_name, "Integration_Pipeline");
            assert_eq!(task_name, "Integration_Task");
            assert_eq!(content, "Ejecutando proceso...");
        }
        _ => panic!("El evento recibido en el canal no es del tipo esperado"),
    }

    let msg_status = Message::TaskStatus {
        task_id,
        status: "Success".to_string(),
    };
    let mut status_json = serde_json::to_string(&msg_status).unwrap();
    status_json.push('\n');
    client_stream.write_all(status_json.as_bytes()).unwrap();

    thread::sleep(Duration::from_millis(100));

    let db_lock = db.lock().unwrap();
    let status = db_lock.get_task_status_by_id(task_id).unwrap();

    assert_eq!(status, "SUCCESS");
}
