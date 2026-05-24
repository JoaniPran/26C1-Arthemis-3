use crate::db::{Database, TaskRecord, TaskStatus};
use rusqlite::{Result, params};

impl Database {
    pub fn insert_workflow(&self, file_name: &str, display_name: &str) -> Result<i32> {
        self.conn.execute(
            "INSERT INTO workflows (file_name, display_name) VALUES (?1, ?2)",
            params![file_name, display_name],
        )?;
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

    pub fn get_task_status_by_id(&self, task_id: i32) -> Result<String> {
        self.conn.query_row(
            "SELECT status FROM tasks WHERE id = ?1",
            params![task_id],
            |row| row.get(0),
        )
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

    pub fn get_workflow_id(&self, file_name: &str) -> Result<i32> {
        self.conn.query_row(
            "SELECT id FROM workflows WHERE file_name = ?1",
            params![file_name],
            |row| row.get(0),
        )
    }

    pub fn get_all_workflows(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT file_name, display_name FROM workflows ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut workflows = Vec::new();
        for row in rows {
            if let Ok(wf) = row {
                workflows.push(wf);
            }
        }
        Ok(workflows)
    }

    pub fn get_tasks_for_ui(
        &self,
        file_name: &str,
    ) -> Result<Vec<(i32, String, String, Vec<String>)>> {
        let wf_id: i32 = match self.get_workflow_id(file_name) {
            Ok(id) => id,
            Err(_) => return Ok(vec![]),
        };

        let mut stmt = self
            .conn
            .prepare("SELECT id, name, status FROM tasks WHERE workflow_id = ?1")?;

        let rows = stmt.query_map([wf_id], |row| {
            let id: i32 = row.get(0)?;
            let name: String = row.get(1)?;
            let status: String = row.get(2)?;
            Ok((id, name, status))
        })?;

        let mut tasks = Vec::new();

        for task_res in rows {
            let (id, name, status) = task_res?;

            let mut log_stmt = self
                .conn
                .prepare("SELECT log_line FROM task_logs WHERE task_id = ?1 ORDER BY id ASC")?;
            let log_rows = log_stmt.query_map([id], |row| row.get::<_, String>(0))?;

            let mut logs = Vec::new();
            for log_res in log_rows {
                if let Ok(l) = log_res {
                    logs.push(l);
                }
            }

            tasks.push((id, name, status, logs));
        }

        Ok(tasks)
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

#[cfg(test)]
mod test {
    use super::*;

    fn setup_memory_db() -> Database {
        Database::new(":memory:").expect("Fallo al crear la BD en memoria")
    }

    #[test]
    fn test_ready_tasks_flow() {
        let db = setup_memory_db();

        let wf_id = db
            .insert_workflow("test_pipeline.yaml", "Test Pipeline")
            .unwrap();

        let t1 = db
            .insert_task(wf_id, "Descargar", "wget localhost")
            .unwrap();
        let t2 = db
            .insert_task(wf_id, "Procesar", "python script.py")
            .unwrap();
        let t3 = db.insert_task(wf_id, "Limpiar", "rm temp").unwrap();

        db.insert_dependency(t2, t1).unwrap();
        db.insert_dependency(t3, t2).unwrap();

        let ready_tasks = db.get_ready_tasks().unwrap();
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, t1);

        db.update_task_status(t1, TaskStatus::Success).unwrap();

        let ready_tasks = db.get_ready_tasks().unwrap();
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, t2);

        db.update_task_status(t2, TaskStatus::Failed).unwrap();

        let ready_tasks = db.get_ready_tasks().unwrap();
        assert_eq!(ready_tasks.len(), 0);
    }

    #[test]
    fn test_workflow_reset() {
        let db = setup_memory_db();
        let wf_id = db
            .insert_workflow("pipeline_reinicio.yaml", "Pipeline a Reiniciar")
            .unwrap();
        let t1 = db.insert_task(wf_id, "Tarea 1", "echo 1").unwrap();

        db.update_task_status(t1, TaskStatus::Success).unwrap();
        db.insert_log(t1, "Log de ejecución 1").unwrap();

        db.reset_workflow(wf_id).unwrap();

        let ready_tasks = db.get_ready_tasks().unwrap();
        assert_eq!(ready_tasks.len(), 1);

        let log_count: i32 = db
            .conn
            .query_row(
                "SELECT count(*) FROM task_logs WHERE task_id = ?1",
                params![t1],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(log_count, 0);
    }
}
