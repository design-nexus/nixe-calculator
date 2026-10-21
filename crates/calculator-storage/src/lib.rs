use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub id: i64,
    pub expression: String,
    pub result: String,
    pub raw_result: String,
    pub mode: String,
    pub timestamp: i64,
    pub is_pinned: bool,
}

#[derive(Clone)]
pub struct CalculatorDb {
    conn: Arc<Mutex<Connection>>,
}

impl CalculatorDb {
    pub fn open_default() -> anyhow::Result<Self> {
        let dir = default_data_dir();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("calculator.db");
        Self::open_path(&path)
    }

    pub fn open_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn: Arc::new(Mutex::new(conn)) };
        db.init_schema()?;
        Ok(db)
    }

    pub fn open_path(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let db = Self { conn: Arc::new(Mutex::new(conn)) };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                expression TEXT NOT NULL,
                result TEXT NOT NULL,
                raw_result TEXT NOT NULL,
                mode TEXT NOT NULL DEFAULT 'standard',
                timestamp INTEGER NOT NULL,
                is_pinned INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_history_timestamp ON history (timestamp DESC, id DESC);
            CREATE INDEX IF NOT EXISTS idx_history_pinned ON history (is_pinned DESC, timestamp DESC, id DESC);
            ",
        )?;
        Ok(())
    }

    pub fn save_calculation(&self, expression: &str, result: &str, raw_result: &str, mode: &str) -> anyhow::Result<i64> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO history (expression, result, raw_result, mode, timestamp, is_pinned)
             VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            params![expression, result, raw_result, mode, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_history(&self, query: Option<&str>, limit: usize, offset: usize) -> anyhow::Result<Vec<HistoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut entries = Vec::new();

        if let Some(q) = query.filter(|s| !s.trim().is_empty()) {
            let escaped = escape_like(q.trim());
            let pattern = format!("%{escaped}%");
            let mut stmt = conn.prepare(
                "SELECT id, expression, result, raw_result, mode, timestamp, is_pinned
                 FROM history
                 WHERE expression LIKE ?1 ESCAPE '\\' OR result LIKE ?1 ESCAPE '\\'
                 ORDER BY is_pinned DESC, timestamp DESC, id DESC
                 LIMIT ?2 OFFSET ?3",
            )?;
            let rows = stmt.query_map(params![pattern, limit as i64, offset as i64], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    expression: row.get(1)?,
                    result: row.get(2)?,
                    raw_result: row.get(3)?,
                    mode: row.get(4)?,
                    timestamp: row.get(5)?,
                    is_pinned: row.get::<_, i64>(6)? != 0,
                })
            })?;
            for r in rows {
                entries.push(r?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, expression, result, raw_result, mode, timestamp, is_pinned
                 FROM history
                 ORDER BY is_pinned DESC, timestamp DESC, id DESC
                 LIMIT ?1 OFFSET ?2",
            )?;
            let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    expression: row.get(1)?,
                    result: row.get(2)?,
                    raw_result: row.get(3)?,
                    mode: row.get(4)?,
                    timestamp: row.get(5)?,
                    is_pinned: row.get::<_, i64>(6)? != 0,
                })
            })?;
            for r in rows {
                entries.push(r?);
            }
        }

        Ok(entries)
    }

    pub fn toggle_pin(&self, id: i64) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let current: i64 = conn.query_row("SELECT is_pinned FROM history WHERE id = ?1", params![id], |row| row.get(0))?;
        let new_state = if current == 0 { 1 } else { 0 };
        conn.execute("UPDATE history SET is_pinned = ?1 WHERE id = ?2", params![new_state, id])?;
        Ok(new_state != 0)
    }

    pub fn delete_history(&self, id: i64) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_history(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history", [])?;
        Ok(())
    }

    pub fn prune_history(&self, max_items: usize) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        // Keep all pinned items, and keep the latest (max_items) unpinned items
        conn.execute(
            "DELETE FROM history
             WHERE is_pinned = 0 AND id NOT IN (
                 SELECT id FROM history WHERE is_pinned = 0 ORDER BY timestamp DESC LIMIT ?1
             )",
            params![max_items as i64],
        )?;
        Ok(())
    }
}

fn default_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(dir).join("nixe-calculator")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share/nixe-calculator")
    } else {
        PathBuf::from("./data")
    }
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_crud_and_search() {
        let db = CalculatorDb::open_in_memory().unwrap();

        let id1 = db.save_calculation("10 + 20", "30", "30", "standard").unwrap();
        let id2 = db.save_calculation("5 * 8", "40", "40", "standard").unwrap();
        let id3 = db.save_calculation("100 + 10%", "110", "110", "standard").unwrap();

        let all = db.list_history(None, 10, 0).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].id, id3);

        // Search
        let filtered = db.list_history(Some("10%"), 10, 0).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, id3);

        // Toggle pin
        let is_pinned = db.toggle_pin(id1).unwrap();
        assert!(is_pinned);

        // Pinned item appears first
        let reordered = db.list_history(None, 10, 0).unwrap();
        assert_eq!(reordered[0].id, id1);

        // Delete single
        db.delete_history(id2).unwrap();
        let after_del = db.list_history(None, 10, 0).unwrap();
        assert_eq!(after_del.len(), 2);

        // Clear all
        db.clear_history().unwrap();
        assert_eq!(db.list_history(None, 10, 0).unwrap().len(), 0);
    }

    #[test]
    fn test_history_pruning() {
        let db = CalculatorDb::open_in_memory().unwrap();
        for i in 0..10 {
            db.save_calculation(&format!("{i} + 1"), &format!("{}", i + 1), "raw", "standard").unwrap();
        }
        assert_eq!(db.list_history(None, 20, 0).unwrap().len(), 10);

        // Prune to 5
        db.prune_history(5).unwrap();
        assert_eq!(db.list_history(None, 20, 0).unwrap().len(), 5);
    }
}
