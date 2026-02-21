//! WSL command executor with encoding detection.
//!
//! `wsl.exe` outputs UTF-16LE by default on Windows.  When the `WSL_UTF8`
//! environment variable is set to `"1"`, it switches to UTF-8.  This module
//! detects the mode at runtime and decodes accordingly.
//!
//! # Notes
//!
//! - `wsl.exe` is NOT called in tests — tests exercise the decode/parse
//!   functions directly.  WSL may not be available in CI.
//! - Null bytes (`\0`) are stripped from decoded output; wsl.exe sometimes
//!   appends them in UTF-16LE mode even after conversion.

use std::process::Command;

use crate::error::CoreError;
use crate::wsl::distro::{parse_list_online, parse_list_verbose, DistroInfo, OnlineDistro};

/// Executes `wsl.exe` subcommands and returns decoded stdout as a `String`.
///
/// Stateless in Phase 1–2.  Future phases may store a handle to a running WSL
/// session or a pre-resolved path to `wsl.exe`.
#[derive(Debug, Default, Clone)]
pub struct WslExecutor;

impl WslExecutor {
    /// Create a new `WslExecutor`.
    pub fn new() -> Self {
        Self
    }

    /// Return `true` when `WSL_UTF8=1` is set in the environment.
    ///
    /// `wsl.exe` switches its output encoding to UTF-8 when this variable is
    /// present and equal to `"1"`.  Any other value (including `"0"`, `"true"`,
    /// or absence) means the output is UTF-16LE.
    fn is_utf8_mode() -> bool {
        std::env::var_os("WSL_UTF8")
            .map(|v| v == "1")
            .unwrap_or(false)
    }

    /// Decode raw `wsl.exe` stdout bytes into a `String`.
    ///
    /// - UTF-8 mode (`WSL_UTF8=1`): treat as lossy UTF-8.
    /// - Default mode: decode as UTF-16LE (the native `wsl.exe` encoding).
    /// - Either way: strip trailing null bytes and surrounding whitespace.
    pub fn decode_output(raw: &[u8]) -> String {
        let decoded = if Self::is_utf8_mode() {
            String::from_utf8_lossy(raw).into_owned()
        } else {
            encoding_rs::UTF_16LE.decode(raw).0.into_owned()
        };

        // Strip null bytes then trim surrounding whitespace.
        decoded
            .trim_end_matches('\0')
            .trim()
            .to_string()
    }

    /// Run `wsl.exe` with the given arguments and return decoded stdout.
    ///
    /// Returns `Err(CoreError::WslExec)` if `wsl.exe` cannot be spawned, and
    /// `Err(CoreError::WslFailed)` if it exits with a non-zero status.
    pub fn run(&self, args: &[&str]) -> Result<String, CoreError> {
        let output = Command::new("wsl.exe")
            .args(args)
            .output()
            .map_err(|e| CoreError::WslExec(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(CoreError::WslFailed(stderr));
        }

        Ok(Self::decode_output(&output.stdout))
    }

    /// Run `wsl.exe --list --verbose` and return the raw decoded output.
    ///
    /// Parse the result with a higher-level function; this method only
    /// handles execution and encoding.
    pub fn list_verbose(&self) -> Result<String, CoreError> {
        self.run(&["--list", "--verbose"])
    }

    /// List all installed WSL distros with state, version, and default indicator.
    ///
    /// Executes `wsl.exe --list --verbose` and parses the output into a typed
    /// `Vec<DistroInfo>`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::WslExec`] if `wsl.exe` cannot be spawned or if the
    /// output cannot be parsed.  Returns [`CoreError::WslFailed`] if `wsl.exe`
    /// exits with a non-zero status.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wsl_core::WslExecutor;
    ///
    /// let executor = WslExecutor::new();
    /// let distros = executor.list_distros().unwrap();
    /// for d in &distros {
    ///     println!("{} — {:?} v{}", d.name, d.state, d.version);
    /// }
    /// ```
    pub fn list_distros(&self) -> Result<Vec<DistroInfo>, CoreError> {
        let output = self.list_verbose()?;
        parse_list_verbose(&output)
    }

    /// List available WSL distros from the online catalog.
    ///
    /// Executes `wsl.exe --list --online` and parses the output into a typed
    /// `Vec<OnlineDistro>`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::WslExec`] if `wsl.exe` cannot be spawned or if the
    /// output cannot be parsed.  Returns [`CoreError::WslFailed`] if `wsl.exe`
    /// exits with a non-zero status.
    pub fn list_online(&self) -> Result<Vec<OnlineDistro>, CoreError> {
        let output = self.run(&["--list", "--online"])?;
        parse_list_online(&output)
    }

    /// Start a stopped WSL distro without attaching to it.
    ///
    /// Executes `wsl.exe -d <name> -- true`.  This starts the distro in the
    /// background (if not already running) and exits immediately.  It does
    /// not open an interactive shell.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::WslExec`] if `wsl.exe` cannot be spawned.
    /// Returns [`CoreError::WslFailed`] if the distro does not exist or cannot
    /// be started.
    pub fn start_distro(&self, name: &str) -> Result<String, CoreError> {
        self.run(&["-d", name, "--", "true"])
    }

    /// Terminate a running WSL distro.
    ///
    /// Executes `wsl.exe --terminate <name>`.  For WSL2, this is equivalent to
    /// a force-stop — there is no graceful shutdown distinction from
    /// `--terminate`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::WslExec`] if `wsl.exe` cannot be spawned.
    /// Returns [`CoreError::WslFailed`] if the distro does not exist.
    pub fn terminate_distro(&self, name: &str) -> Result<String, CoreError> {
        self.run(&["--terminate", name])
    }

    /// Set the specified distro as the WSL default.
    ///
    /// Executes `wsl.exe --set-default <name>`.  After this call, running
    /// `wsl.exe` without arguments will use this distro.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::WslExec`] if `wsl.exe` cannot be spawned.
    /// Returns [`CoreError::WslFailed`] if the distro does not exist.
    pub fn set_default(&self, name: &str) -> Result<String, CoreError> {
        self.run(&["--set-default", name])
    }

    /// Permanently unregister (remove) a WSL distro and delete all its data.
    ///
    /// Executes `wsl.exe --unregister <name>`.
    ///
    /// **Warning:** This operation is irreversible.  All files inside the
    /// distro are permanently deleted.  The caller is responsible for
    /// obtaining user confirmation before calling this method.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::WslExec`] if `wsl.exe` cannot be spawned.
    /// Returns [`CoreError::WslFailed`] if the distro does not exist.
    pub fn unregister(&self, name: &str) -> Result<String, CoreError> {
        self.run(&["--unregister", name])
    }

    /// Export a WSL distro to a `.tar` file.
    ///
    /// Executes `wsl.exe --export <name> <path>`.  The output file is a
    /// standard tar archive that can be imported with [`import_distro`].
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::WslExec`] if `wsl.exe` cannot be spawned.
    /// Returns [`CoreError::WslFailed`] if the distro does not exist or the
    /// path is not writable.
    ///
    /// [`import_distro`]: WslExecutor::import_distro
    pub fn export_distro(&self, name: &str, path: &str) -> Result<String, CoreError> {
        self.run(&["--export", name, path])
    }

    /// Import a distro from a `.tar` file created by [`export_distro`].
    ///
    /// Executes `wsl.exe --import <name> <install_dir> <tar_path>`.  Creates a
    /// new WSL distro with the given name, installed at `install_dir`, from the
    /// tar archive at `tar_path`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::WslExec`] if `wsl.exe` cannot be spawned.
    /// Returns [`CoreError::WslFailed`] if the tar file does not exist, the
    /// name is already in use, or the install directory cannot be created.
    ///
    /// [`export_distro`]: WslExecutor::export_distro
    pub fn import_distro(
        &self,
        name: &str,
        install_dir: &str,
        tar_path: &str,
    ) -> Result<String, CoreError> {
        self.run(&["--import", name, install_dir, tar_path])
    }

    /// Update the WSL kernel to the latest version.
    ///
    /// Executes `wsl.exe --update`.  Downloads and installs the latest WSL
    /// kernel package from Microsoft.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::WslExec`] if `wsl.exe` cannot be spawned.
    /// Returns [`CoreError::WslFailed`] if the update fails (e.g., no network
    /// access).
    pub fn update_wsl(&self) -> Result<String, CoreError> {
        self.run(&["--update"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use crate::wsl::distro::{parse_list_verbose, DistroState};

    /// Serialize tests that set/unset `WSL_UTF8` so they don't interfere.
    static WSL_UTF8_LOCK: Mutex<()> = Mutex::new(());

    fn utf8_guard() -> std::sync::MutexGuard<'static, ()> {
        let guard = WSL_UTF8_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("WSL_UTF8");
        guard
    }

    // ── is_utf8_mode ──────────────────────────────────────────────────────────

    #[test]
    fn test_is_utf8_mode_absent() {
        let _guard = utf8_guard();
        assert!(!WslExecutor::is_utf8_mode(), "should be false when WSL_UTF8 is absent");
    }

    #[test]
    fn test_is_utf8_mode_set_to_one() {
        let _guard = utf8_guard();
        std::env::set_var("WSL_UTF8", "1");
        assert!(WslExecutor::is_utf8_mode());
    }

    #[test]
    fn test_is_utf8_mode_set_to_zero() {
        let _guard = utf8_guard();
        std::env::set_var("WSL_UTF8", "0");
        assert!(!WslExecutor::is_utf8_mode());
    }

    #[test]
    fn test_is_utf8_mode_set_to_other() {
        let _guard = utf8_guard();
        std::env::set_var("WSL_UTF8", "true");
        assert!(!WslExecutor::is_utf8_mode());
    }

    // ── decode_output: UTF-8 mode ─────────────────────────────────────────────

    #[test]
    fn test_decode_output_utf8() {
        let _guard = utf8_guard();
        std::env::set_var("WSL_UTF8", "1");

        let input = b"Ubuntu 22.04 LTS";
        let result = WslExecutor::decode_output(input);
        assert_eq!(result, "Ubuntu 22.04 LTS");
    }

    #[test]
    fn test_decode_output_utf8_with_newline() {
        let _guard = utf8_guard();
        std::env::set_var("WSL_UTF8", "1");

        let input = b"  Ubuntu  \n";
        let result = WslExecutor::decode_output(input);
        assert_eq!(result, "Ubuntu");
    }

    // ── decode_output: UTF-16LE mode ──────────────────────────────────────────

    #[test]
    fn test_decode_output_utf16le() {
        let _guard = utf8_guard();
        // No WSL_UTF8 set → UTF-16LE mode.

        // "Hello" encoded as UTF-16LE:
        // H=0x48,0x00  e=0x65,0x00  l=0x6C,0x00  l=0x6C,0x00  o=0x6F,0x00
        let input: &[u8] = &[0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00];
        let result = WslExecutor::decode_output(input);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_decode_output_utf16le_multiword() {
        let _guard = utf8_guard();
        // "WSL" as UTF-16LE: W=0x57,0x00  S=0x53,0x00  L=0x4C,0x00
        let input: &[u8] = &[0x57, 0x00, 0x53, 0x00, 0x4C, 0x00];
        let result = WslExecutor::decode_output(input);
        assert_eq!(result, "WSL");
    }

    // ── null byte stripping ───────────────────────────────────────────────────

    #[test]
    fn test_null_byte_stripping_utf8_mode() {
        let _guard = utf8_guard();
        std::env::set_var("WSL_UTF8", "1");

        // UTF-8 text with trailing null bytes.
        let mut input = b"Ubuntu".to_vec();
        input.extend_from_slice(&[0x00, 0x00]);
        let result = WslExecutor::decode_output(&input);
        assert_eq!(result, "Ubuntu");
    }

    #[test]
    fn test_null_byte_stripping_utf16le_mode() {
        let _guard = utf8_guard();
        // UTF-16LE "Hi" + trailing null bytes.
        // H=0x48,0x00  i=0x69,0x00  then two null bytes.
        let input: &[u8] = &[0x48, 0x00, 0x69, 0x00, 0x00, 0x00];
        let result = WslExecutor::decode_output(input);
        // After UTF-16LE decode we get "Hi\0" then trim_end_matches strips the \0.
        assert_eq!(result, "Hi");
    }

    // ── WslExecutor::new and default ──────────────────────────────────────────

    #[test]
    fn test_executor_new() {
        let _exec = WslExecutor::new();
        // Constructing WslExecutor should not panic.
    }

    #[test]
    fn test_executor_default() {
        let _exec = WslExecutor::default();
    }

    // ── list_distros / list_online parse integration ──────────────────────────

    /// Verify that parse_list_verbose returns Vec<DistroInfo> with the expected
    /// structure when given representative sample data.
    #[test]
    fn test_list_distros_parse_integration() {
        let sample = "  NAME                   STATE           VERSION\n\
                      * Ubuntu                 Running         2\n\
                        docker-desktop-data    Stopped         2\n";

        let distros =
            parse_list_verbose(sample).expect("parse_list_verbose should succeed on sample data");

        assert_eq!(distros.len(), 2, "expected 2 distros from sample data");
        assert_eq!(distros[0].name, "Ubuntu");
        assert_eq!(distros[0].state, DistroState::Running);
        assert!(distros[0].is_default);
        assert_eq!(distros[0].version, 2);

        assert_eq!(distros[1].name, "docker-desktop-data");
        assert_eq!(distros[1].state, DistroState::Stopped);
        assert!(!distros[1].is_default);
    }

    /// Verify that the executor methods exist and have the correct signatures
    /// by calling them and observing the expected WslExec/WslFailed error
    /// (wsl.exe is not available in CI, or the named distro does not exist).
    #[test]
    fn test_start_distro_args() {
        let executor = WslExecutor::new();
        // This will fail with either WslExec (wsl.exe not found) or WslFailed
        // (distro not found).  Either way the method exists and is callable.
        let result = executor.start_distro("__nonexistent_distro__");
        assert!(
            result.is_err(),
            "start_distro should return Err for a non-existent distro or absent wsl.exe"
        );
    }
}
