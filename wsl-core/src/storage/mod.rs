//! Storage abstraction layer for wsl-core.
//!
//! Provides a trait-based storage backend system with two implementations:
//! - [`LibsqlBackend`]: embedded libsql (SQLite-compatible) — primary backend
//! - [`JsonBackend`]: file-based JSON storage — transparent fallback
//!
//! Use [`open_storage`] to obtain a backend. Pass [`BackendKind::Auto`] to
//! try libsql first and fall back to JSON automatically.

pub mod json;
pub mod libsql;

use std::path::Path;

use async_trait::async_trait;

use crate::config::StorageMode;
use crate::error::CoreError;

pub use json::JsonBackend;
pub use libsql::LibsqlBackend;

/// A single cell value in a storage row.
///
/// This type is independent of any specific backend, allowing both
/// [`LibsqlBackend`] and [`JsonBackend`] to return the same row format.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StorageValue {
    /// SQL NULL / JSON null.
    Null,
    /// 64-bit signed integer.
    Integer(i64),
    /// 64-bit floating-point number.
    Real(f64),
    /// UTF-8 text string.
    Text(String),
    /// Raw binary blob.
    Blob(Vec<u8>),
}

/// A single row returned by a storage query.
///
/// Each element corresponds to one column in the SELECT result, in order.
pub type StorageRow = Vec<StorageValue>;

/// Backend-agnostic storage interface.
///
/// Both [`LibsqlBackend`] and [`JsonBackend`] implement this trait, so
/// callers never need to know which backend is active.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Execute a write statement (CREATE TABLE, INSERT, DELETE, UPDATE).
    ///
    /// Returns the number of rows affected.
    async fn execute(
        &self,
        sql: &str,
        params: Vec<StorageValue>,
    ) -> Result<u64, CoreError>;

    /// Execute a read query (SELECT).
    ///
    /// Returns all matching rows as [`StorageRow`] vectors.
    async fn query(
        &self,
        sql: &str,
        params: Vec<StorageValue>,
    ) -> Result<Vec<StorageRow>, CoreError>;

    /// Return a short identifier for the active backend.
    ///
    /// Consumed by the TUI status bar (locked decision).
    /// Returns `"libsql"` or `"json"`.
    fn backend_name(&self) -> &'static str;
}

/// Which storage backend to open.
///
/// Maps directly from [`StorageMode`] in the config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendKind {
    /// Try libsql first; fall back to JSON on failure.
    Auto,
    /// Require libsql; return an error if unavailable.
    Libsql,
    /// Always use JSON storage.
    Json,
}

impl From<StorageMode> for BackendKind {
    fn from(mode: StorageMode) -> Self {
        match mode {
            StorageMode::Auto => BackendKind::Auto,
            StorageMode::Libsql => BackendKind::Libsql,
            StorageMode::Json => BackendKind::Json,
        }
    }
}

/// Result of opening a storage backend.
///
/// Contains the active backend and metadata about the chosen configuration.
pub struct StorageResult {
    /// The active storage backend (libsql or json).
    pub backend: Box<dyn StorageBackend>,

    /// Short identifier for the active backend: `"libsql"` or `"json"`.
    ///
    /// Consumed by the TUI status bar (locked decision).
    pub backend_name: &'static str,

    /// `true` when libsql is active AND a `data.json` file exists from a
    /// prior JSON-fallback run.
    ///
    /// The TUI uses this to offer a one-time migration prompt in Phase 2.
    /// No actual data migration happens in Phase 1 — only detection.
    pub migration_available: bool,
}

/// Open a storage backend according to the requested [`BackendKind`].
///
/// # Auto-fallback behaviour
///
/// When `kind` is [`BackendKind::Auto`]:
/// 1. Try to open a [`LibsqlBackend`].
/// 2. If successful, check whether `data.json` exists (prior JSON run)
///    and set `migration_available` accordingly.
/// 3. If libsql fails for any reason, open a [`JsonBackend`] instead
///    (`migration_available` is always `false` for the JSON backend).
///
/// # Errors
///
/// Returns [`CoreError::StorageError`] if the explicitly requested backend
/// (`Libsql` or `Json`) fails to open. In `Auto` mode, errors are swallowed
/// during the libsql attempt and the JSON fallback is used instead.
pub async fn open_storage(
    config_dir: &Path,
    kind: BackendKind,
) -> Result<StorageResult, CoreError> {
    let data_json_path = config_dir.join("data.json");

    match kind {
        BackendKind::Libsql => {
            let backend = LibsqlBackend::open(config_dir).await?;
            let migration_available = data_json_path.exists();
            Ok(StorageResult {
                backend_name: backend.backend_name(),
                backend: Box::new(backend),
                migration_available,
            })
        }
        BackendKind::Json => {
            let backend = JsonBackend::open(config_dir)?;
            Ok(StorageResult {
                backend_name: backend.backend_name(),
                backend: Box::new(backend),
                migration_available: false,
            })
        }
        BackendKind::Auto => {
            match LibsqlBackend::open(config_dir).await {
                Ok(backend) => {
                    let migration_available = data_json_path.exists();
                    Ok(StorageResult {
                        backend_name: backend.backend_name(),
                        backend: Box::new(backend),
                        migration_available,
                    })
                }
                Err(_) => {
                    let backend = JsonBackend::open(config_dir)?;
                    Ok(StorageResult {
                        backend_name: backend.backend_name(),
                        backend: Box::new(backend),
                        migration_available: false,
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_open_storage_auto_libsql_succeeds() {
        let tmp = tempdir().unwrap();
        let result = open_storage(tmp.path(), BackendKind::Auto).await.unwrap();
        // In auto mode, libsql should succeed (stack fix in place).
        assert_eq!(result.backend_name, "libsql");
        // No prior data.json exists.
        assert!(!result.migration_available);
    }

    #[tokio::test]
    async fn test_open_storage_json_explicit() {
        let tmp = tempdir().unwrap();
        let result = open_storage(tmp.path(), BackendKind::Json).await.unwrap();
        assert_eq!(result.backend_name, "json");
        assert!(!result.migration_available);
    }

    #[tokio::test]
    async fn test_open_storage_factory_returns_backend_name() {
        let tmp = tempdir().unwrap();

        let libsql_result = open_storage(tmp.path(), BackendKind::Libsql).await.unwrap();
        assert_eq!(libsql_result.backend_name, libsql_result.backend.backend_name());

        let json_result = open_storage(tmp.path(), BackendKind::Json).await.unwrap();
        assert_eq!(json_result.backend_name, json_result.backend.backend_name());
    }

    #[tokio::test]
    async fn test_open_storage_migration_detection() {
        let tmp = tempdir().unwrap();

        // Simulate a prior JSON run by writing a dummy data.json.
        std::fs::write(tmp.path().join("data.json"), b"{}").unwrap();

        let result = open_storage(tmp.path(), BackendKind::Auto).await.unwrap();
        // libsql should succeed AND data.json exists → migration_available = true.
        assert_eq!(result.backend_name, "libsql");
        assert!(result.migration_available);
    }

    #[test]
    fn test_backend_kind_from_storage_mode() {
        assert_eq!(BackendKind::from(StorageMode::Auto), BackendKind::Auto);
        assert_eq!(BackendKind::from(StorageMode::Libsql), BackendKind::Libsql);
        assert_eq!(BackendKind::from(StorageMode::Json), BackendKind::Json);
    }
}
