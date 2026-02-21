//! WSL command execution module.
//!
//! Provides [`WslExecutor`] for running `wsl.exe` subcommands with
//! automatic encoding detection (UTF-16LE vs UTF-8), and the
//! [`DistroInfo`] / [`DistroState`] / [`OnlineDistro`] types produced
//! by parsing `wsl.exe --list` output.

pub mod distro;
pub mod executor;

pub use distro::{DistroInfo, DistroState, OnlineDistro};
pub use executor::WslExecutor;
