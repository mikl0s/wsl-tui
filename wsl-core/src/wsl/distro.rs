//! WSL distro data types and output parsers.
//!
//! Provides the [`DistroInfo`], [`DistroState`], and [`OnlineDistro`] types
//! along with functions to parse `wsl.exe --list --verbose` and
//! `wsl.exe --list --online` output.

use crate::error::CoreError;

/// The runtime state of a WSL distro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistroState {
    /// The distro is currently running.
    Running,
    /// The distro is stopped.
    Stopped,
}

/// Information about an installed WSL distro.
///
/// Produced by parsing `wsl.exe --list --verbose` output.
///
/// # Example
///
/// ```
/// use wsl_core::wsl::distro::{DistroInfo, DistroState};
///
/// let info = DistroInfo {
///     name: "Ubuntu".to_string(),
///     state: DistroState::Running,
///     version: 2,
///     is_default: true,
/// };
/// assert_eq!(info.name, "Ubuntu");
/// assert!(info.is_default);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistroInfo {
    /// Distro name (e.g., "Ubuntu-22.04").
    pub name: String,
    /// Whether the distro is currently running or stopped.
    pub state: DistroState,
    /// WSL version (1 or 2).
    pub version: u8,
    /// Whether this is the current WSL default distro (marked with `*`).
    pub is_default: bool,
}

/// An available distro from the WSL online catalog.
///
/// Produced by parsing `wsl.exe --list --online` output.
///
/// # Example
///
/// ```
/// use wsl_core::wsl::distro::OnlineDistro;
///
/// let distro = OnlineDistro {
///     name: "Ubuntu".to_string(),
///     friendly_name: "Ubuntu".to_string(),
/// };
/// assert_eq!(distro.name, "Ubuntu");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnlineDistro {
    /// Short install name used with `wsl --install <name>`.
    pub name: String,
    /// Human-readable name shown in the UI.
    pub friendly_name: String,
}

/// Parse `wsl.exe --list --verbose` decoded output into a `Vec<DistroInfo>`.
///
/// Expects the format:
/// ```text
///   NAME                   STATE           VERSION
/// * Ubuntu                 Running         2
///   docker-desktop-data    Stopped         2
/// ```
///
/// The first line (header) is always skipped. Lines with `*` in the leading
/// position mark the default distro.
///
/// # Errors
///
/// Returns [`CoreError::WslExec`] if a line cannot be parsed (unexpected
/// column count or unknown state string).
///
/// # Example
///
/// ```
/// use wsl_core::wsl::distro::{parse_list_verbose, DistroState};
///
/// let output = "  NAME    STATE    VERSION\n* Ubuntu  Running  2\n";
/// let distros = parse_list_verbose(output).unwrap();
/// assert_eq!(distros.len(), 1);
/// assert_eq!(distros[0].name, "Ubuntu");
/// assert!(distros[0].is_default);
/// assert_eq!(distros[0].state, DistroState::Running);
/// assert_eq!(distros[0].version, 2);
/// ```
pub fn parse_list_verbose(output: &str) -> Result<Vec<DistroInfo>, CoreError> {
    let mut distros = Vec::new();

    for line in output.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let is_default = trimmed.starts_with('*');
        let rest = trimmed.trim_start_matches('*').trim();

        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(CoreError::WslExec(format!(
                "unexpected column count in wsl --list --verbose line: {:?}",
                line
            )));
        }

        let name = parts[0].to_string();
        let state = match parts[1] {
            "Running" => DistroState::Running,
            "Stopped" => DistroState::Stopped,
            other => {
                return Err(CoreError::WslExec(format!(
                    "unknown distro state: {:?}",
                    other
                )))
            }
        };
        let version = parts[2].parse::<u8>().map_err(|e| {
            CoreError::WslExec(format!("invalid WSL version {:?}: {}", parts[2], e))
        })?;

        distros.push(DistroInfo {
            name,
            state,
            version,
            is_default,
        });
    }

    Ok(distros)
}

/// Parse `wsl.exe --list --online` decoded output into a `Vec<OnlineDistro>`.
///
/// Expects the format:
/// ```text
/// NAME                                   FRIENDLY NAME
/// Ubuntu                                 Ubuntu
/// Debian                                 Debian GNU/Linux
/// ```
///
/// The first line (header) is always skipped. Columns are separated by two or
/// more consecutive spaces.
///
/// # Errors
///
/// Returns [`CoreError::WslExec`] if a line cannot be split into exactly two
/// columns.
///
/// # Example
///
/// ```
/// use wsl_core::wsl::distro::parse_list_online;
///
/// let output = "NAME    FRIENDLY NAME\nUbuntu  Ubuntu\n";
/// let distros = parse_list_online(output).unwrap();
/// assert_eq!(distros.len(), 1);
/// assert_eq!(distros[0].name, "Ubuntu");
/// assert_eq!(distros[0].friendly_name, "Ubuntu");
/// ```
pub fn parse_list_online(output: &str) -> Result<Vec<OnlineDistro>, CoreError> {
    let mut distros = Vec::new();

    for line in output.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Split on two or more consecutive spaces as the column separator.
        // Using a simple approach: find the first run of 2+ spaces.
        let parts: Vec<&str> = trimmed.splitn(2, "  ").collect();
        if parts.len() < 2 {
            return Err(CoreError::WslExec(format!(
                "unexpected column format in wsl --list --online line: {:?}",
                line
            )));
        }

        let name = parts[0].trim().to_string();
        let friendly_name = parts[1].trim().to_string();

        if name.is_empty() || friendly_name.is_empty() {
            return Err(CoreError::WslExec(format!(
                "empty column in wsl --list --online line: {:?}",
                line
            )));
        }

        distros.push(OnlineDistro { name, friendly_name });
    }

    Ok(distros)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_list_verbose ────────────────────────────────────────────────────

    #[test]
    fn test_parse_list_verbose_typical() {
        let output = "  NAME                   STATE           VERSION\n\
                      * Ubuntu                 Running         2\n\
                        docker-desktop-data    Stopped         2\n";

        let distros = parse_list_verbose(output)
            .expect("parse should succeed on typical output");

        assert_eq!(distros.len(), 2);

        assert_eq!(distros[0].name, "Ubuntu");
        assert_eq!(distros[0].state, DistroState::Running);
        assert_eq!(distros[0].version, 2);
        assert!(distros[0].is_default);

        assert_eq!(distros[1].name, "docker-desktop-data");
        assert_eq!(distros[1].state, DistroState::Stopped);
        assert_eq!(distros[1].version, 2);
        assert!(!distros[1].is_default);
    }

    #[test]
    fn test_parse_list_verbose_single_default() {
        let output = "  NAME    STATE    VERSION\n\
                      * Ubuntu  Running  2\n";

        let distros = parse_list_verbose(output)
            .expect("parse should succeed with single default distro");

        assert_eq!(distros.len(), 1);
        assert_eq!(distros[0].name, "Ubuntu");
        assert!(distros[0].is_default);
        assert_eq!(distros[0].state, DistroState::Running);
        assert_eq!(distros[0].version, 2);
    }

    #[test]
    fn test_parse_list_verbose_empty() {
        let output = "  NAME    STATE    VERSION\n";

        let distros = parse_list_verbose(output)
            .expect("parse should succeed with header-only output");

        assert_eq!(distros.len(), 0);
    }

    #[test]
    fn test_parse_list_verbose_running_and_stopped() {
        let output = "  NAME         STATE    VERSION\n\
                      * Ubuntu       Running  2\n\
                        Debian       Stopped  2\n\
                        Alpine       Running  1\n";

        let distros = parse_list_verbose(output)
            .expect("parse should succeed with mixed states");

        assert_eq!(distros.len(), 3);

        assert_eq!(distros[0].state, DistroState::Running);
        assert!(distros[0].is_default);

        assert_eq!(distros[1].name, "Debian");
        assert_eq!(distros[1].state, DistroState::Stopped);
        assert!(!distros[1].is_default);

        assert_eq!(distros[2].name, "Alpine");
        assert_eq!(distros[2].state, DistroState::Running);
        assert_eq!(distros[2].version, 1);
    }

    // ── parse_list_online ─────────────────────────────────────────────────────

    #[test]
    fn test_parse_list_online_typical() {
        let output = "NAME                                   FRIENDLY NAME\n\
                      Ubuntu                                 Ubuntu\n\
                      Debian                                 Debian GNU/Linux\n";

        let distros = parse_list_online(output)
            .expect("parse should succeed on typical online output");

        assert_eq!(distros.len(), 2);

        assert_eq!(distros[0].name, "Ubuntu");
        assert_eq!(distros[0].friendly_name, "Ubuntu");

        assert_eq!(distros[1].name, "Debian");
        assert_eq!(distros[1].friendly_name, "Debian GNU/Linux");
    }

    #[test]
    fn test_parse_list_online_empty() {
        let output = "NAME    FRIENDLY NAME\n";

        let distros = parse_list_online(output)
            .expect("parse should succeed with header-only output");

        assert_eq!(distros.len(), 0);
    }
}
