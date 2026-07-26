pub mod init;
pub mod logs;
pub mod models;
pub mod tasks;
pub use models::{TaskRecord, TaskStatus};

pub type UiTask = (i32, String, String, Vec<String>);
use rusqlite::Connection;

pub struct Database {
    pub(crate) conn: Connection,
}
