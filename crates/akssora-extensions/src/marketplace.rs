use std::collections::HashMap;

use crate::extension::ExtensionPackage;

// INFO: Package manager tracking local installations and extension marketplace interactions.
#[derive(Default)]
pub struct ExtensionsManager {
    installed: HashMap<String, ExtensionPackage>,
}

impl ExtensionsManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install(&mut self, mut package: ExtensionPackage) -> anyhow::Result<()> {
        package.installed = true;
        self.installed.insert(package.id.clone(), package);
        Ok(())
    }

    pub fn uninstall(&mut self, id: &str) -> anyhow::Result<bool> {
        Ok(self.installed.remove(id).is_some())
    }

    pub fn list_installed(&self) -> Vec<&ExtensionPackage> {
        self.installed.values().collect()
    }
}
