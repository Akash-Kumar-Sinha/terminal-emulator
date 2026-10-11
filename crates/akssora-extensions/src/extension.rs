// INFO: Manifest describing an installable extension package and its repository origin.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtensionPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub repository: Option<String>,
    pub installed: bool,
}
