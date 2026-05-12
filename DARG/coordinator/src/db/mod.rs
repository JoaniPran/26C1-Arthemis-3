pub mod init;
pub mod logs;
pub mod models;
pub mod tasks;

pub use models::{TaskRecord, TaskStatus};
use rusqlite::Connection;

pub struct Database {
    pub(crate) conn: Connection,
}
