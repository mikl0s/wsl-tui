//! `wsl-core` — shared library for the WSL TUI workspace.
//!
//! Provides:
//! - Error types ([`CoreError`])
//! - Application configuration ([`Config`], [`StorageMode`])
//! - Storage backends ([`storage`])
//! - WSL command execution ([`WslExecutor`])
//! - Compile-time plugin system ([`Plugin`], [`PluginRegistry`])

pub mod config;
pub mod error;
pub mod plugin;
pub mod storage;
pub mod wsl;

pub use config::{Config, StorageMode};
pub use error::CoreError;
pub use plugin::{Plugin, PluginRegistry};
pub use wsl::WslExecutor;
