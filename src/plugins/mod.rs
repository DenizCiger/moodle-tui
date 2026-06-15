pub mod manifest;
pub mod protocol;
pub mod registry;
pub mod runtime;

pub use manifest::{PluginManifest, PluginPermission};
pub use registry::{InstalledPlugin, PluginRegistry};
