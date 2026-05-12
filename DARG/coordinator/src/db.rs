use rusqlite::{Connection, Result, params};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Success,
    Failed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.create_table()?;
        Ok(db)
    }

    fn create_table(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS workflows (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workflow_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                command TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'PENDING',
                FOREIGN KEY(workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS dependencies (
                task_id INTEGER NOT NULL,
                depends_on_id INTEGER NOT NULL,
                PRIMARY KEY (task_id, depends_on_id),
                FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY(depends_on_id) REFERENCES tasks(id) ON DELETE CASCADE
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

    pub fn insert_workflow(&self, name: &str) -> Result<i32> {
        self.conn
            .execute("INSERT INTO workflows (name) VALUES (?1)", params![name])?;
        let id = self.conn.last_insert_rowid();
        Ok(id as i32)
    }

    pub fn insert_task(&self, workflow_id: i32, name: &str, command: &str) -> Result<i32> {
        self.conn.execute(
            "INSERT INTO tasks (workflow_id, name, command, status) VALUES (?1, ?2, ?3, 'PENDING')",
            params![workflow_id, name, command],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(id as i32)
    }

    pub fn insert_dependency(&self, task_id: i32, depends_on_id: i32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO dependencies (task_id, depends_on_id) VALUES (?1, ?2)",
            params![task_id, depends_on_id],
        )?;
        Ok(())
    }

    pub fn update_task_status(&self, task_id: i32, status: TaskStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            params![status.to_string(), task_id],
        )?;
        Ok(())
    }

    // pub fn update_task_status_by_name(&self, name: &str, status: TaskStatus) -> Result<()> {
    //     self.conn.execute(
    //         "UPDATE tasks SET status = ?1 WHERE name = ?2",
    //         params![status.to_string(), name],
    //     )?;
    //     Ok(())
    // }

    pub fn get_ready_tasks(&self) -> Result<Vec<TaskRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, command
             FROM tasks 
             WHERE status = 'PENDING' 
             AND id NOT IN (
                 SELECT d.task_id 
                 FROM dependencies d 
                 JOIN tasks t ON d.depends_on_id = t.id 
                 WHERE t.status != 'SUCCESS'
             )",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TaskRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                command: row.get(2)?,
            })
        })?;

        let mut tasks = Vec::new();
        for task in rows {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    pub fn get_workflow_id(&self, name: &str) -> Result<i32> {
        self.conn.query_row(
            "SELECT id FROM workflows WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )
    }

    pub fn reset_workflow(&self, workflow_id: i32) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = 'PENDING' WHERE workflow_id = ?1",
            params![workflow_id],
        )?;

        self.conn.execute(
            "DELETE FROM task_logs WHERE task_id IN (SELECT id FROM tasks WHERE workflow_id = ?1)",
            params![workflow_id],
        )?;
        Ok(())
    }

    pub fn reset_all_running_tasks(&self) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = 'PENDING' WHERE status = 'RUNNING'",
            [],
        )?;
        Ok(())
    }

    pub fn insert_log(&self, task_id: i32, log_line: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO task_logs (task_id, log_line) VALUES (?1, ?2)",
            params![task_id, log_line],
        )?;
        Ok(())
    }

    // pub fn get_workflow_logs(&self, workflow_id: i32) -> Result<Vec<(String, String)>> {
    //     let mut stmt = self.conn.prepare(
    //         "SELECT t.name, l.log_line
    //          FROM task_logs l
    //          JOIN tasks t ON l.task_id = t.id
    //          WHERE t.workflow_id = ?1
    //          ORDER BY l.id ASC"
    //     )?;

    //     let rows = stmt.query_map(params![workflow_id], |row| {
    //         Ok((row.get(0)?, row.get(1)?))
    //     })?;

    //     let mut logs = Vec::new();
    //     for log in rows { logs.push(log?); }
    //     Ok(logs)
    // }

    pub fn get_new_workflow_logs(
        &self,
        workflow_id: i32,
        last_id: i32,
    ) -> Result<Vec<(i32, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT l.id, t.name, l.log_line 
             FROM task_logs l 
             JOIN tasks t ON l.task_id = t.id 
             WHERE t.workflow_id = ?1 AND l.id > ?2 
             ORDER BY l.id ASC",
        )?;

        let rows = stmt.query_map(params![workflow_id, last_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        let mut logs = Vec::new();
        for log in rows {
            logs.push(log?);
        }
        Ok(logs)
    }
}
