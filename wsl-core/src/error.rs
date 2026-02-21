/// Core error type for the wsl-core library.
///
/// All public-facing APIs in wsl-core return `Result<T, CoreError>`.
/// Application crates (wsl-tui, wsl-web) wrap these with `anyhow::Error`
/// for ergonomic propagation.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Could not determine the user's home directory.
    #[error("cannot determine home directory — $HOME or USERPROFILE not set")]
    NoHomeDir,

    /// Config file exists but could not be parsed as TOML.
    #[error("config parse error: {0}")]
    ConfigParse(String),

    /// I/O error reading the config file or performing file operations.
    #[error("config read error: {0}")]
    ConfigRead(#[from] std::io::Error),

    /// TOML deserialization error.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// Storage backend initialization or operation failed.
    #[error("storage error: {0}")]
    StorageError(String),

    /// Failed to spawn or communicate with `wsl.exe`.
    #[error("wsl.exe exec error: {0}")]
    WslExec(String),

    /// `wsl.exe` returned a non-zero exit code.
    #[error("wsl.exe failed: {0}")]
    WslFailed(String),

    /// A loaded plugin reported an error.
    #[error("plugin error: {0}")]
    PluginError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_home_dir_display() {
        let err = CoreError::NoHomeDir;
        assert!(err.to_string().contains("home directory"));
    }

    #[test]
    fn test_config_parse_display() {
        let err = CoreError::ConfigParse("bad toml".to_string());
        assert!(err.to_string().contains("config parse error"));
        assert!(err.to_string().contains("bad toml"));
    }

    #[test]
    fn test_config_read_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = CoreError::ConfigRead(io_err);
        assert!(err.to_string().contains("config read error"));
    }

    #[test]
    fn test_storage_error_display() {
        let err = CoreError::StorageError("disk full".to_string());
        assert!(err.to_string().contains("storage error"));
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn test_wsl_exec_display() {
        let err = CoreError::WslExec("permission denied".to_string());
        assert!(err.to_string().contains("wsl.exe exec error"));
    }

    #[test]
    fn test_wsl_failed_display() {
        let err = CoreError::WslFailed("exit code 1".to_string());
        assert!(err.to_string().contains("wsl.exe failed"));
    }

    #[test]
    fn test_plugin_error_display() {
        let err = CoreError::PluginError("lua runtime error".to_string());
        assert!(err.to_string().contains("plugin error"));
        assert!(err.to_string().contains("lua runtime error"));
    }

    #[test]
    fn test_toml_parse_from_error() {
        // A deliberately bad TOML string to trigger a parse error.
        let bad_toml = "[[[ not valid";
        let toml_err = toml::from_str::<toml::Value>(bad_toml).unwrap_err();
        let err = CoreError::TomlParse(toml_err);
        assert!(err.to_string().contains("TOML parse error"));
    }
}
