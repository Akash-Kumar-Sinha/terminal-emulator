use std::collections::HashMap;

use crate::plugin::AkssoraPlugin;

// INFO: Central registry maintaining active plugin instances and managing their lifecycles.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn AkssoraPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, mut plugin: Box<dyn AkssoraPlugin>) -> anyhow::Result<()> {
        let manifest = plugin.manifest();
        plugin.on_load()?;
        self.plugins.insert(manifest.id, plugin);
        Ok(())
    }

    pub fn unregister(&mut self, id: &str) -> anyhow::Result<bool> {
        if let Some(mut plugin) = self.plugins.remove(id) {
            plugin.on_unload()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}
