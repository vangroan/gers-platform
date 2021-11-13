//! Schema of the `plugin.toml` file.
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
}
