use std::path::Path;

use serde::{Deserialize, Serialize};

pub const MANIFEST_FILE: &str = ".aboot.json";

/// Records what aboot installed so `update`/`status`/`doctor`/`list` can work
/// deterministically.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Manifest {
    pub version: u32,
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    pub agent: String,
    pub date: String,
    pub files: Vec<String>,
}

impl Manifest {
    pub fn load(root: &Path) -> Option<Manifest> {
        let text = std::fs::read_to_string(root.join(MANIFEST_FILE)).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub fn save(&self, root: &Path) -> crate::error::Result<()> {
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(root.join(MANIFEST_FILE), text + "\n")?;
        Ok(())
    }
}
