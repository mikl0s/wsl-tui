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

/// Executes `wsl.exe` subcommands and returns decoded stdout as a `String`.
///
/// Stateless in Phase 1.  Future phases may store a handle to a running WSL
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

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
}
