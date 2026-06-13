use crate::db::Database;
use rusqlite::{Connection, Result};

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        let db = Database { conn };
        db.create_table()?;
        Ok(db)
    }

    fn create_table(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS workflows (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_name TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workflow_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                command TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'PENDING',
                produces TEXT,
                FOREIGN KEY(workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS dependencies (
                task_id INTEGER NOT NULL,
                depends_on_id INTEGER NOT NULL,
                PRIMARY KEY (task_id, depends_on_id),
                FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY(depends_on_id) REFERENCES tasks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS task_consumes (
                task_id INTEGER NOT NULL,
                consumed_task_id INTEGER NOT NULL,
                PRIMARY KEY (task_id, consumed_task_id),
                FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY(consumed_task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS task_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                log_line TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );
            ",
        )?;
        Ok(())
    }
}
