//! JSON file storage backend.
//!
//! Uses a simple JSON file (`<config_dir>/data.json`) as a fallback storage
//! backend when libsql is unavailable. Supports a subset of SQL operations
//! sufficient for the application's own queries.
//!
//! This module is fully implemented in Phase 1 Plan 02 Task 2.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

use super::{StorageBackend, StorageRow, StorageValue};

/// In-memory representation of all JSON-backed table data.
///
/// Keys are table names; values are lists of rows (each row is a list of
/// [`StorageValue`] cells).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JsonData {
    pub tables: HashMap<String, Vec<StorageRow>>,
}

/// JSON file storage backend.
///
/// All mutations are written to disk immediately after each [`execute`] call
/// to preserve durability.
pub struct JsonBackend {
    file_path: PathBuf,
    data: Arc<Mutex<JsonData>>,
}

impl JsonBackend {
    /// Open (or create) the JSON storage file at `<config_dir>/data.json`.
    ///
    /// If the file exists, its contents are loaded into memory. If not, an
    /// empty state is used and the file is created on the first write.
    pub fn open(config_dir: &Path) -> Result<Self, CoreError> {
        let file_path = config_dir.join("data.json");
        let data = if file_path.exists() {
            let text = std::fs::read_to_string(&file_path)
                .map_err(|e| CoreError::StorageError(format!("json read failed: {e}")))?;
            serde_json::from_str::<JsonData>(&text)
                .map_err(|e| CoreError::StorageError(format!("json parse failed: {e}")))?
        } else {
            JsonData::default()
        };
        Ok(Self {
            file_path,
            data: Arc::new(Mutex::new(data)),
        })
    }

    /// Persist current in-memory state to disk.
    fn flush(&self, data: &JsonData) -> Result<(), CoreError> {
        let text = serde_json::to_string_pretty(data)
            .map_err(|e| CoreError::StorageError(format!("json serialize failed: {e}")))?;
        std::fs::write(&self.file_path, text)
            .map_err(|e| CoreError::StorageError(format!("json write failed: {e}")))?;
        Ok(())
    }
}

// ── SQL mini-parser helpers ───────────────────────────────────────────────────
//
// The JSON backend does NOT implement a full SQL parser. It handles only the
// specific statements that the application issues itself:
//
//   CREATE TABLE IF NOT EXISTS {name} (...)
//   INSERT INTO {name} (...) VALUES (...)   [with positional ?N params]
//   SELECT ... FROM {name} [WHERE ...]      [returns all rows; WHERE ignored]
//   DELETE FROM {name}                      [truncates table]
//
// Statements are matched case-insensitively via normalised whitespace.

/// Extract the table name from a CREATE TABLE statement.
fn parse_create_table(sql: &str) -> Option<String> {
    let upper = sql.to_uppercase();
    let upper = upper.trim();
    // CREATE TABLE IF NOT EXISTS name (...)
    let stripped = upper
        .strip_prefix("CREATE TABLE IF NOT EXISTS ")
        .or_else(|| upper.strip_prefix("CREATE TABLE "))?;
    let name = stripped
        .split_whitespace()
        .next()?
        .split('(')
        .next()?
        .trim()
        .to_string();
    if name.is_empty() {
        None
    } else {
        Some(name.to_lowercase())
    }
}

/// Extract the table name from an INSERT INTO statement.
fn parse_insert_table(sql: &str) -> Option<String> {
    let upper = sql.to_uppercase();
    let upper = upper.trim();
    let stripped = upper.strip_prefix("INSERT INTO ")?;
    let name = stripped
        .split_whitespace()
        .next()?
        .split('(')
        .next()?
        .trim()
        .to_string();
    if name.is_empty() {
        None
    } else {
        Some(name.to_lowercase())
    }
}

/// Extract the table name from a SELECT … FROM statement.
fn parse_select_table(sql: &str) -> Option<String> {
    let upper = sql.to_uppercase();
    let pos = upper.find(" FROM ")?;
    let after_from = &upper[pos + 6..];
    let name = after_from
        .split_whitespace()
        .next()?
        .split(';')
        .next()?
        .trim()
        .to_string();
    if name.is_empty() {
        None
    } else {
        Some(name.to_lowercase())
    }
}

/// Extract the table name from a DELETE FROM statement.
fn parse_delete_table(sql: &str) -> Option<String> {
    let upper = sql.to_uppercase();
    let upper = upper.trim();
    let stripped = upper.strip_prefix("DELETE FROM ")?;
    let name = stripped
        .split_whitespace()
        .next()?
        .split(';')
        .next()?
        .trim()
        .to_string();
    if name.is_empty() {
        None
    } else {
        Some(name.to_lowercase())
    }
}

#[async_trait]
impl StorageBackend for JsonBackend {
    /// Execute a write statement.
    ///
    /// Supported: `CREATE TABLE IF NOT EXISTS`, `INSERT INTO`, `DELETE FROM`.
    /// After each successful mutation the state is flushed to `data.json`.
    async fn execute(
        &self,
        sql: &str,
        params: Vec<StorageValue>,
    ) -> Result<u64, CoreError> {
        let sql_upper = sql.trim().to_uppercase();
        let mut data = self
            .data
            .lock()
            .map_err(|_| CoreError::StorageError("json lock poisoned".into()))?;

        if sql_upper.starts_with("CREATE TABLE") {
            let table = parse_create_table(sql).ok_or_else(|| {
                CoreError::StorageError(format!("json: cannot parse CREATE TABLE: {sql}"))
            })?;
            data.tables.entry(table).or_default();
            self.flush(&data)?;
            Ok(0)
        } else if sql_upper.starts_with("INSERT INTO") {
            let table = parse_insert_table(sql).ok_or_else(|| {
                CoreError::StorageError(format!("json: cannot parse INSERT INTO: {sql}"))
            })?;
            let rows = data.tables.entry(table).or_default();
            rows.push(params);
            let count = rows.len() as u64;
            self.flush(&data)?;
            // Return 1 — one row inserted.
            let _ = count;
            Ok(1)
        } else if sql_upper.starts_with("DELETE FROM") {
            let table = parse_delete_table(sql).ok_or_else(|| {
                CoreError::StorageError(format!("json: cannot parse DELETE FROM: {sql}"))
            })?;
            let rows = data.tables.entry(table).or_default();
            let count = rows.len() as u64;
            rows.clear();
            self.flush(&data)?;
            Ok(count)
        } else {
            Err(CoreError::StorageError(format!(
                "json backend: unsupported SQL statement: {sql}"
            )))
        }
    }

    /// Execute a read query.
    ///
    /// Supported: `SELECT … FROM {table}` — returns all rows.
    /// `WHERE` clauses and column projections are ignored; all rows and
    /// columns are returned as stored.
    async fn query(
        &self,
        sql: &str,
        _params: Vec<StorageValue>,
    ) -> Result<Vec<StorageRow>, CoreError> {
        let sql_upper = sql.trim().to_uppercase();
        let data = self
            .data
            .lock()
            .map_err(|_| CoreError::StorageError("json lock poisoned".into()))?;

        if sql_upper.starts_with("SELECT") {
            let table = parse_select_table(sql).ok_or_else(|| {
                CoreError::StorageError(format!("json: cannot parse SELECT: {sql}"))
            })?;
            let rows = data.tables.get(&table).cloned().unwrap_or_default();
            Ok(rows)
        } else {
            Err(CoreError::StorageError(format!(
                "json backend: unsupported query: {sql}"
            )))
        }
    }

    fn backend_name(&self) -> &'static str {
        "json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_json_smoke() {
        let tmp = tempdir().unwrap();
        let backend = JsonBackend::open(tmp.path()).unwrap();

        // Create table.
        backend
            .execute(
                "CREATE TABLE IF NOT EXISTS smoke (id INTEGER PRIMARY KEY, name TEXT)",
                vec![],
            )
            .await
            .unwrap();

        // Insert row.
        let rows_affected = backend
            .execute(
                "INSERT INTO smoke (id, name) VALUES (?1, ?2)",
                vec![
                    StorageValue::Integer(1),
                    StorageValue::Text("hello".to_string()),
                ],
            )
            .await
            .unwrap();
        assert_eq!(rows_affected, 1);

        // Query it back.
        let rows = backend
            .query("SELECT id, name FROM smoke", vec![])
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], StorageValue::Integer(1));
        assert_eq!(rows[0][1], StorageValue::Text("hello".to_string()));
    }

    #[tokio::test]
    async fn test_json_backend_name() {
        let tmp = tempdir().unwrap();
        let backend = JsonBackend::open(tmp.path()).unwrap();
        assert_eq!(backend.backend_name(), "json");
    }

    #[tokio::test]
    async fn test_json_persistence() {
        let tmp = tempdir().unwrap();

        // Write data.
        {
            let backend = JsonBackend::open(tmp.path()).unwrap();
            backend
                .execute("CREATE TABLE IF NOT EXISTS persist (val TEXT)", vec![])
                .await
                .unwrap();
            backend
                .execute(
                    "INSERT INTO persist (val) VALUES (?1)",
                    vec![StorageValue::Text("durable".to_string())],
                )
                .await
                .unwrap();
        }

        // Reopen and verify data survives.
        {
            let backend = JsonBackend::open(tmp.path()).unwrap();
            let rows = backend
                .query("SELECT val FROM persist", vec![])
                .await
                .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], StorageValue::Text("durable".to_string()));
        }
    }

    #[tokio::test]
    async fn test_json_delete() {
        let tmp = tempdir().unwrap();
        let backend = JsonBackend::open(tmp.path()).unwrap();

        backend
            .execute("CREATE TABLE IF NOT EXISTS del_test (id INTEGER)", vec![])
            .await
            .unwrap();
        backend
            .execute(
                "INSERT INTO del_test (id) VALUES (?1)",
                vec![StorageValue::Integer(1)],
            )
            .await
            .unwrap();

        let count = backend
            .execute("DELETE FROM del_test", vec![])
            .await
            .unwrap();
        assert_eq!(count, 1);

        let rows = backend
            .query("SELECT id FROM del_test", vec![])
            .await
            .unwrap();
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn test_json_unsupported_statement() {
        let tmp = tempdir().unwrap();
        let backend = JsonBackend::open(tmp.path()).unwrap();

        let result = backend
            .execute("UPDATE foo SET bar = 1", vec![])
            .await;
        assert!(result.is_err());
    }
}
