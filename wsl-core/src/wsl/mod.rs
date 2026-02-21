//! WSL command execution module.
//!
//! Provides [`WslExecutor`] for running `wsl.exe` subcommands with
//! automatic encoding detection (UTF-16LE vs UTF-8).

pub mod executor;

pub use executor::WslExecutor;
