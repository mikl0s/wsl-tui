//! Plugin registry for compile-time plugin registration.
//!
//! The registry holds [`Plugin`] trait objects in insertion order.
//! Retrieval is linear — the expected number of plugins per application is
//! small (< 20) so a `Vec` is preferable to a `HashMap` for simplicity.

use super::Plugin;

/// Holds all registered plugins.
///
/// Created once at application startup; plugins are registered during
/// initialization and the registry is then treated as read-only at runtime.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin.
    ///
    /// If a plugin with the same name was already registered it is not
    /// replaced — both entries remain.  Callers should ensure uniqueness if
    /// that matters for their use case.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Look up a plugin by name.
    ///
    /// Returns `None` if no plugin with that name is registered.
    /// If multiple plugins share the same name, the first match is returned.
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    /// Return a slice of all registered plugins in insertion order.
    pub fn all(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    /// Return the number of registered plugins.
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: &'static str,
        version: &'static str,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            self.name
        }
        fn version(&self) -> &str {
            self.version
        }
    }

    fn make_plugin(name: &'static str, version: &'static str) -> Box<dyn Plugin> {
        Box::new(TestPlugin { name, version })
    }

    // ── register & get ────────────────────────────────────────────────────────

    #[test]
    fn test_register_and_get() {
        let mut registry = PluginRegistry::new();
        registry.register(make_plugin("alpha", "1.0.0"));

        let plugin = registry.get("alpha");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().name(), "alpha");
        assert_eq!(plugin.unwrap().version(), "1.0.0");
    }

    #[test]
    fn test_get_nonexistent() {
        let registry = PluginRegistry::new();
        assert!(registry.get("does-not-exist").is_none());
    }

    #[test]
    fn test_get_returns_first_match_on_duplicate_name() {
        let mut registry = PluginRegistry::new();
        registry.register(make_plugin("dup", "1.0.0"));
        registry.register(make_plugin("dup", "2.0.0"));

        // First registration wins.
        let plugin = registry.get("dup").unwrap();
        assert_eq!(plugin.version(), "1.0.0");
    }

    // ── all ───────────────────────────────────────────────────────────────────

    #[test]
    fn test_all_empty() {
        let registry = PluginRegistry::new();
        assert!(registry.all().is_empty());
    }

    #[test]
    fn test_all_returns_all_plugins() {
        let mut registry = PluginRegistry::new();
        registry.register(make_plugin("alpha", "1.0.0"));
        registry.register(make_plugin("beta", "2.0.0"));
        registry.register(make_plugin("gamma", "3.0.0"));

        let all = registry.all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].name(), "alpha");
        assert_eq!(all[1].name(), "beta");
        assert_eq!(all[2].name(), "gamma");
    }

    // ── count ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_count_empty() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_count_after_registration() {
        let mut registry = PluginRegistry::new();
        registry.register(make_plugin("a", "0.1.0"));
        assert_eq!(registry.count(), 1);

        registry.register(make_plugin("b", "0.1.0"));
        assert_eq!(registry.count(), 2);
    }

    // ── Default ───────────────────────────────────────────────────────────────

    #[test]
    fn test_default_is_empty() {
        let registry = PluginRegistry::default();
        assert_eq!(registry.count(), 0);
    }
}
