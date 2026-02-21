//! libsql storage backend.
//!
//! Uses libsql's embedded local database (SQLite-compatible). The database
//! file is stored at `<config_dir>/wsl-tui.db`.
//!
//! The Windows `/STACK:8000000` linker flag (set in `.cargo/config.toml`)
//! prevents a stack overflow during libsql's SQL parser initialisation — this
//! was the critical fix from Phase 1 Plan 01.

use std::path::Path;

use async_trait::async_trait;

use crate::error::CoreError;

use super::{StorageBackend, StorageRow, StorageValue};

/// libsql embedded database backend.
///
/// Holds the [`libsql::Database`] handle (keeps the connection alive) and
/// an open [`libsql::Connection`] for executing statements.
pub struct LibsqlBackend {
    /// Must be kept alive for the duration of the backend's existence.
    _db: libsql::Database,
    conn: libsql::Connection,
}

impl LibsqlBackend {
    /// Open (or create) the libsql database at `<config_dir>/wsl-tui.db`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::StorageError`] if the database cannot be opened
    /// or the connection cannot be established.
    pub async fn open(config_dir: &Path) -> Result<Self, CoreError> {
        let db_path = config_dir.join("wsl-tui.db");
        let db = libsql::Builder::new_local(db_path)
            .build()
            .await
            .map_err(|e| CoreError::StorageError(format!("libsql open failed: {e}")))?;
        let conn = db
            .connect()
            .map_err(|e| CoreError::StorageError(format!("libsql connect failed: {e}")))?;
        Ok(Self { _db: db, conn })
    }

    /// Open an in-memory libsql database.
    ///
    /// Used in tests to avoid touching the filesystem.
    #[cfg(test)]
    pub async fn open_memory() -> Result<Self, CoreError> {
        let db = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .map_err(|e| CoreError::StorageError(format!("libsql memory open failed: {e}")))?;
        let conn = db
            .connect()
            .map_err(|e| CoreError::StorageError(format!("libsql memory connect failed: {e}")))?;
        Ok(Self { _db: db, conn })
    }
}

/// Convert a [`StorageValue`] to a libsql parameter value.
fn storage_value_to_libsql(v: StorageValue) -> libsql::Value {
    match v {
        StorageValue::Null => libsql::Value::Null,
        StorageValue::Integer(i) => libsql::Value::Integer(i),
        StorageValue::Real(f) => libsql::Value::Real(f),
        StorageValue::Text(s) => libsql::Value::Text(s),
        StorageValue::Blob(b) => libsql::Value::Blob(b),
    }
}

/// Convert a libsql [`libsql::Value`] to a [`StorageValue`].
fn libsql_value_to_storage(v: libsql::Value) -> StorageValue {
    match v {
        libsql::Value::Null => StorageValue::Null,
        libsql::Value::Integer(i) => StorageValue::Integer(i),
        libsql::Value::Real(f) => StorageValue::Real(f),
        libsql::Value::Text(s) => StorageValue::Text(s),
        libsql::Value::Blob(b) => StorageValue::Blob(b),
    }
}

#[async_trait]
impl StorageBackend for LibsqlBackend {
    async fn execute(
        &self,
        sql: &str,
        params: Vec<StorageValue>,
    ) -> Result<u64, CoreError> {
        let libsql_params: Vec<libsql::Value> =
            params.into_iter().map(storage_value_to_libsql).collect();

        let rows_affected = self
            .conn
            .execute(sql, libsql_params)
            .await
            .map_err(|e| CoreError::StorageError(format!("libsql execute failed: {e}")))?;

        Ok(rows_affected)
    }

    async fn query(
        &self,
        sql: &str,
        params: Vec<StorageValue>,
    ) -> Result<Vec<StorageRow>, CoreError> {
        let libsql_params: Vec<libsql::Value> =
            params.into_iter().map(storage_value_to_libsql).collect();

        let mut rows = self
            .conn
            .query(sql, libsql_params)
            .await
            .map_err(|e| CoreError::StorageError(format!("libsql query failed: {e}")))?;

        let mut result = Vec::new();
        loop {
            match rows.next().await {
                Ok(Some(row)) => {
                    let col_count = rows.column_count();
                    let mut storage_row: StorageRow = Vec::with_capacity(col_count as usize);
                    for i in 0..col_count {
                        let val = row
                            .get_value(i)
                            .map_err(|e| {
                                CoreError::StorageError(format!(
                                    "libsql get_value col {i} failed: {e}"
                                ))
                            })?;
                        storage_row.push(libsql_value_to_storage(val));
                    }
                    result.push(storage_row);
                }
                Ok(None) => break,
                Err(e) => {
                    return Err(CoreError::StorageError(format!(
                        "libsql row fetch failed: {e}"
                    )));
                }
            }
        }

        Ok(result)
    }

    fn backend_name(&self) -> &'static str {
        "libsql"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// THE critical smoke test: create a table, insert a row, query it back.
    ///
    /// This proves the `/STACK:8000000` (GNU: `-Wl,--stack,8000000`) linker
    /// flag works and libsql's SQL parser does not overflow the stack on
    /// Windows.
    #[tokio::test]
    async fn test_libsql_smoke() {
        let backend = LibsqlBackend::open_memory().await.unwrap();

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
                "INSERT INTO smoke (id, name) VALUES (1, 'hello')",
                vec![],
            )
            .await
            .unwrap();
        assert_eq!(rows_affected, 1);

        // Query it back.
        let rows = backend
            .query("SELECT id, name FROM smoke WHERE id = 1", vec![])
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], StorageValue::Integer(1));
        assert_eq!(rows[0][1], StorageValue::Text("hello".to_string()));
    }

    #[tokio::test]
    async fn test_libsql_backend_name() {
        let backend = LibsqlBackend::open_memory().await.unwrap();
        assert_eq!(backend.backend_name(), "libsql");
    }

    #[tokio::test]
    async fn test_libsql_execute_and_query() {
        let backend = LibsqlBackend::open_memory().await.unwrap();

        backend
            .execute(
                "CREATE TABLE IF NOT EXISTS items (id INTEGER, value TEXT, score REAL)",
                vec![],
            )
            .await
            .unwrap();

        // Insert using positional parameters.
        backend
            .execute(
                "INSERT INTO items (id, value, score) VALUES (?1, ?2, ?3)",
                vec![
                    StorageValue::Integer(42),
                    StorageValue::Text("test".to_string()),
                    StorageValue::Real(3.14),
                ],
            )
            .await
            .unwrap();

        let rows = backend
            .query("SELECT id, value, score FROM items", vec![])
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], StorageValue::Integer(42));
        assert_eq!(rows[0][1], StorageValue::Text("test".to_string()));

        // Real comparison with tolerance.
        if let StorageValue::Real(f) = rows[0][2] {
            assert!((f - 3.14).abs() < 1e-10);
        } else {
            panic!("expected Real, got {:?}", rows[0][2]);
        }
    }

    #[tokio::test]
    async fn test_libsql_null_handling() {
        let backend = LibsqlBackend::open_memory().await.unwrap();

        backend
            .execute(
                "CREATE TABLE IF NOT EXISTS nullable (id INTEGER, val TEXT)",
                vec![],
            )
            .await
            .unwrap();

        backend
            .execute(
                "INSERT INTO nullable (id, val) VALUES (?1, ?2)",
                vec![StorageValue::Integer(1), StorageValue::Null],
            )
            .await
            .unwrap();

        let rows = backend
            .query("SELECT id, val FROM nullable", vec![])
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], StorageValue::Integer(1));
        assert_eq!(rows[0][1], StorageValue::Null);
    }
}
