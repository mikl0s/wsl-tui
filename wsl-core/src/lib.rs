//! `wsl-core` — shared library for the WSL TUI workspace.
//!
//! Provides:
//! - Error types ([`CoreError`])
//! - Application configuration ([`Config`], [`StorageMode`])
//! - Storage backends ([`storage`])

pub mod config;
pub mod error;
pub mod storage;

pub use config::{Config, StorageMode};
pub use error::CoreError;
