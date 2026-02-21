//! Plugin system for wsl-core.
//!
//! Phase 1 provides compile-time (static) plugin registration only.
//! Runtime plugins (Lua, WASM) are deferred to later phases.
//!
//! # Usage
//!
//! ```rust,ignore
//! use wsl_core::plugin::{Plugin, PluginRegistry};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str { "my-plugin" }
//!     fn version(&self) -> &str { "0.1.0" }
//! }
//!
//! let mut registry = PluginRegistry::new();
//! registry.register(Box::new(MyPlugin));
//!
//! assert!(registry.get("my-plugin").is_some());
//! ```

pub mod registry;

pub use registry::PluginRegistry;

/// The compile-time plugin interface.
///
/// Every plugin must implement this trait.  In Phase 1 all plugins are
/// registered at startup via [`PluginRegistry::register`].  Runtime plugins
/// (Lua/WASM) added in later phases will implement this same trait so the
/// registry stays uniform.
pub trait Plugin: Send + Sync {
    /// The unique, lowercase identifier for this plugin.
    ///
    /// Used as the key in [`PluginRegistry::get`].  Collisions are not
    /// checked — the last registration wins.
    fn name(&self) -> &str;

    /// Semantic version string (e.g. `"0.1.0"`).
    fn version(&self) -> &str;
}
