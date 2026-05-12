use crate::db::{Database, TaskRecord, TaskStatus};
use rusqlite::{Result, params};

impl Database {
    pub fn insert_workflow(&self, name: &str) -> Result<i32> {
        self.conn
            .execute("INSERT INTO workflows (name) VALUES (?1)", params![name])?;
        Ok(self.conn.last_insert_rowid() as i32)
    }

    pub fn insert_task(&self, workflow_id: i32, name: &str, command: &str) -> Result<i32> {
        self.conn.execute(
            "INSERT INTO tasks (workflow_id, name, command, status) VALUES (?1, ?2, ?3, 'PENDING')",
            params![workflow_id, name, command],
        )?;
        Ok(self.conn.last_insert_rowid() as i32)
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
}
