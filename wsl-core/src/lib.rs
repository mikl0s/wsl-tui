//! `wsl-core` — shared library for the WSL TUI workspace.
//!
//! Provides:
//! - Error types ([`CoreError`])
//! - Application configuration ([`Config`], [`StorageMode`])
//!
//! Future phases will add storage, WSL command execution, and plugin modules.

pub mod config;
pub mod error;

pub use config::{Config, StorageMode};
pub use error::CoreError;
