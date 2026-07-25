use crate::db::{Database, TaskRecord};
use rusqlite::{Result, params};

impl Database {
    pub fn insert_workflow(
        &self,
        user_id: i32,
        file_name: &str,
        display_name: &str,
    ) -> Result<i32> {
        self.conn.execute(
            "INSERT INTO workflows (user_id, file_name, display_name) VALUES (?1, ?2, ?3)",
            params![user_id, file_name, display_name],
        )?;
        Ok(self.conn.last_insert_rowid() as i32)
    }

    pub fn insert_task(
        &self,
        workflow_id: i32,
        name: &str,
        command: &str,
        produces: Option<&str>,
    ) -> Result<i32> {
        self.conn.execute(
            "INSERT INTO tasks (workflow_id, name, command, status, produces) VALUES (?1, ?2, ?3, 'SLEEPING', ?4)",
            params![workflow_id, name, command, produces],
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

    pub fn insert_consumption(&self, task_id: i32, consumed_task_id: i32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO task_consumes (task_id, consumed_task_id) VALUES (?1, ?2)",
            params![task_id, consumed_task_id],
        )?;
        Ok(())
    }

    fn update_task_status_internal(&self, task_id: i32, status: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            params![status, task_id],
        )?;
        Ok(())
    }

    pub fn set_task_pending(&self, task_id: i32) -> Result<(), String> {
        let estado_actual = self
            .get_task_status_by_id(task_id)
            .map_err(|e| format!("Error al consultar la tarea {}: {}", task_id, e))?;

        if estado_actual != "SLEEPING" {
            return Err(format!(
                "Transición inválida: No se puede pasar a PENDING una tarea en estado {}",
                estado_actual
            ));
        }

        self.update_task_status_internal(task_id, "PENDING")
            .map_err(|e| format!("Error de BDD al iniciar tarea: {}", e))?;
        Ok(())
    }

    pub fn start_task(&self, task_id: i32) -> Result<(), String> {
        let estado_actual = self
            .get_task_status_by_id(task_id)
            .map_err(|e| format!("Error al consultar la tarea {}: {}", task_id, e))?;

        if estado_actual != "PENDING" {
            return Err(format!(
                "Transición inválida: No se puede pasar a RUNNING una tarea en estado {}",
                estado_actual
            ));
        }

        self.update_task_status_internal(task_id, "RUNNING")
            .map_err(|e| format!("Error de BDD al iniciar tarea: {}", e))?;
        Ok(())
    }

    pub fn complete_task(&self, task_id: i32) -> Result<(), String> {
        let estado_actual = self
            .get_task_status_by_id(task_id)
            .map_err(|e| format!("Error al consultar la tarea {}: {}", task_id, e))?;

        if estado_actual != "RUNNING" {
            return Err(format!(
                "Transición inválida: No se puede completar una tarea en estado {}",
                estado_actual
            ));
        }

        self.update_task_status_internal(task_id, "SUCCESS")
            .map_err(|e| format!("Error de BDD al completar tarea: {}", e))?;
        Ok(())
    }

    pub fn sleeping_task(&self, task_id: i32) -> Result<(), String> {
        let estado_actual = self
            .get_task_status_by_id(task_id)
            .map_err(|e| format!("Error al consultar la tarea {}: {}", task_id, e))?;

        if estado_actual != "SUCCESS" {
            return Err(format!(
                "Transición inválida: No se puede dormir una tarea en estado {}",
                estado_actual
            ));
        }

        self.update_task_status_internal(task_id, "SLEEPING")
            .map_err(|e| format!("Error de BDD al dormir la tarea: {}", e))?;
        Ok(())
    }

    pub fn fail_task(&self, task_id: i32) -> Result<(), String> {
        let estado_actual = self
            .get_task_status_by_id(task_id)
            .map_err(|e| format!("Error al consultar la tarea {}: {}", task_id, e))?;

        if estado_actual != "RUNNING" {
            return Err(format!(
                "Transición inválida: No se puede fallar una tarea en estado {}",
                estado_actual
            ));
        }

        self.update_task_status_internal(task_id, "FAILED")
            .map_err(|e| format!("Error de BDD al fallar tarea: {}", e))?;
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
            "SELECT t.id, t.name, t.command, t.produces, u.username
            FROM tasks t
            JOIN workflows w ON t.workflow_id = w.id
            JOIN users u ON w.user_id = u.id
            WHERE t.status = 'PENDING' 
            AND t.id NOT IN (
                SELECT d.task_id 
                FROM dependencies d 
                JOIN tasks dep_t ON d.depends_on_id = dep_t.id 
                WHERE dep_t.status != 'SUCCESS'
            )",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TaskRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                command: row.get(2)?,
                produces: row.get(3)?,
                username: row.get(4)?,
            })
        })?;

        let mut tasks = Vec::new();
        for task in rows {
            tasks.push(task?);
        }

        Ok(tasks)
    }

    pub fn get_workflow_id(&self, user_id: i32, file_name: &str) -> Result<i32> {
        self.conn.query_row(
            "SELECT id FROM workflows WHERE user_id = ?1 AND file_name = ?2",
            params![user_id, file_name],
            |row| row.get(0),
        )
    }

    pub fn get_workflows_for_user(&self, user_id: i32) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT file_name, display_name FROM workflows WHERE user_id = ?1 ORDER BY id ASC",
        )?;

        let rows = stmt.query_map([user_id], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut workflows = Vec::new();
        for row in rows.flatten() {
            workflows.push(row);
        }
        Ok(workflows)
    }

    pub fn get_tasks_for_ui(
        &self,
        user_id: i32,
        file_name: &str,
    ) -> Result<Vec<(i32, String, String, Vec<String>)>> {
        let wf_id: i32 = match self.get_workflow_id(user_id, file_name) {
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
            for log_res in log_rows.flatten() {
                logs.push(log_res);
            }

            tasks.push((id, name, status, logs));
        }

        Ok(tasks)
    }

    pub fn get_artifacts_to_download(&self, task_id: i32) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.consumed_task_id, u.username 
                 FROM task_consumes c
                 JOIN tasks t ON c.consumed_task_id = t.id
                 JOIN workflows w ON t.workflow_id = w.id
                 JOIN users u ON w.user_id = u.id
                 WHERE c.task_id = ?1",
        )?;

        let rows = stmt.query_map([task_id], |row| {
            let id: i32 = row.get(0)?;
            let username: String = row.get(1)?;
            Ok(format!("{}/artefacto_tarea_{}.zip", username, id))
        })?;

        let mut artifacts = Vec::new();
        for row in rows.flatten() {
            artifacts.push(row);
        }

        Ok(artifacts)
    }

    pub fn reset_workflow(&self, user_id: i32, workflow_name: String) -> Result<()> {
        let workflow_id = self.get_workflow_id(user_id, &workflow_name)?;

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

    pub fn delete_workflows(&self, workflow_id: i32) -> Result<()> {
        self.conn
            .execute("DELETE FROM workflows WHERE id = ?1", params![workflow_id])?;
        Ok(())
    }

    pub fn register_user(&self, username: &str, password_hash: &str) -> Result<i32> {
        self.conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            params![username, password_hash],
        )?;
        Ok(self.conn.last_insert_rowid() as i32)
    }

    pub fn authenticate_user(&self, username: &str, password_hash: &str) -> Result<i32> {
        self.conn.query_row(
            "SELECT id FROM users WHERE username = ?1 AND password_hash = ?2",
            params![username, password_hash],
            |row| row.get(0),
        )
    }

    pub fn get_user_id_by_username(&self, username: &str) -> rusqlite::Result<i32> {
        self.conn.query_row(
            "SELECT id FROM users WHERE username = ?1",
            rusqlite::params![username],
            |row| row.get(0),
        )
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

        let user_id = db.register_user("test_user", "hash123").unwrap();

        let wf_id = db
            .insert_workflow(user_id, "test_pipeline.yaml", "Test Pipeline")
            .unwrap();

        let t1 = db
            .insert_task(wf_id, "Descargar", "wget localhost", None)
            .unwrap();
        let t2 = db
            .insert_task(wf_id, "Procesar", "python script.py", None)
            .unwrap();
        let t3 = db.insert_task(wf_id, "Limpiar", "rm temp", None).unwrap();

        db.insert_dependency(t2, t1).unwrap();
        db.insert_dependency(t3, t2).unwrap();

        db.set_task_pending(t1).unwrap();
        db.set_task_pending(t2).unwrap();
        db.set_task_pending(t3).unwrap();

        let ready_tasks = db.get_ready_tasks().unwrap();
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, t1);

        db.start_task(t1).unwrap();
        db.complete_task(t1).unwrap();

        let ready_tasks = db.get_ready_tasks().unwrap();
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, t2);

        db.start_task(t2).unwrap();
        db.fail_task(t2).unwrap();

        let ready_tasks = db.get_ready_tasks().unwrap();
        assert_eq!(ready_tasks.len(), 0);
    }

    #[test]
    fn test_workflow_reset() {
        let db = setup_memory_db();

        let user_id = db.register_user("test_user", "hash123").unwrap();

        let wf_id = db
            .insert_workflow(user_id, "pipeline_reinicio.yaml", "Pipeline a Reiniciar")
            .unwrap();
        let t1 = db.insert_task(wf_id, "Tarea 1", "echo 1", None).unwrap();

        db.set_task_pending(t1).unwrap();
        db.start_task(t1).unwrap();
        db.complete_task(t1).unwrap();
        db.insert_log(t1, "Log de ejecución 1").unwrap();

        db.reset_workflow(user_id, "pipeline_reinicio.yaml".into())
            .unwrap();

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
