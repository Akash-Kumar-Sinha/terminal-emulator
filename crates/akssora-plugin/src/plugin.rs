#[derive(Clone, Debug, PartialEq)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

// INFO: Extension trait defining lifecycle callbacks for dynamically registered terminal plugins.
pub trait AkssoraPlugin: Send + Sync {
    fn manifest(&self) -> PluginManifest;
    fn on_load(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    fn on_unload(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
