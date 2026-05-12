use crate::db::Database;
use rusqlite::{Result, params};

impl Database {
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
    //             FROM task_logs l
    //             JOIN tasks t ON l.task_id = t.id
    //             WHERE t.workflow_id = ?1
    //             ORDER BY l.id ASC"
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
