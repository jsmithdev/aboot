use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::manifest::Manifest;
use crate::stacks::Stack;
use crate::template::{Vars, TEMPLATE_VERSION};

/// Docs that moved from the root to `.aboot/` — the cold docs in template
/// v2, `BOARD.md` in v3.
pub const MIGRATED_DOCS: &[&str] = &[
    "REQUIREMENTS.md",
    "BACKLOG.md",
    "BOARD.md",
    "DECISIONS.md",
    "PROJECT.md",
    "SESSIONS.md",
];

/// A project directory with its (optional) aboot manifest and resolved vars.
pub struct Project {
    pub root: PathBuf,
    pub manifest: Option<Manifest>,
    pub vars: Vars,
}

impl Project {
    /// Open a project, resolving template variables from (in priority order):
    /// explicit overrides, the `.aboot.json` manifest, directory name,
    /// and stack detection from build files.
    pub fn open(root: &Path, description: Option<&str>, stack: Option<Stack>) -> Result<Project> {
        let root = root
            .canonicalize()
            .map_err(|_| Error::msg(format!("directory not found: {}", root.display())))?;
        if !root.is_dir() {
            return Err(Error::msg(format!("not a directory: {}", root.display())));
        }
        let manifest = Manifest::load(&root);
        let name = manifest
            .as_ref()
            .map(|m| m.name.clone())
            .or_else(|| root.file_name().map(|f| f.to_string_lossy().to_string()))
            .unwrap_or_else(|| root.display().to_string());
        let description = description
            .map(|s| s.to_string())
            .or_else(|| manifest.as_ref().map(|m| m.description.clone()))
            .unwrap_or_default();
        let stack = stack
            .or_else(|| {
                manifest
                    .as_ref()
                    .and_then(|m| m.stack.as_deref().and_then(Stack::parse))
            })
            .or_else(|| Stack::detect(&root));
        let agent = manifest
            .as_ref()
            .map(|m| m.agent.clone())
            .unwrap_or_else(|| "agents".to_string());
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        Ok(Project {
            root,
            manifest,
            vars: Vars {
                name,
                description,
                date,
                stack,
                agent,
            },
        })
    }

    /// Workflow files expected for this project's agent mode.
    pub fn expected_files(&self) -> Vec<&'static str> {
        let mut files: Vec<&'static str> = Vec::new();
        files.extend(match self.vars.agent.as_str() {
            "claude" => vec!["CLAUDE.md"],
            "both" => vec!["AGENTS.md", "CLAUDE.md"],
            _ => vec!["AGENTS.md"],
        });
        files.extend([
            ".aboot/REQUIREMENTS.md",
            ".aboot/BACKLOG.md",
            ".aboot/BOARD.md",
            ".aboot/DECISIONS.md",
            ".aboot/PROJECT.md",
            ".aboot/SESSIONS.md",
            "justfile",
            ".editorconfig",
        ]);
        files
    }

    /// Move root-level cold docs into `.aboot/` for projects installed by an
    /// older aboot (manifest version below the current template version).
    /// Returns the files that were moved (or would be, with `dry_run`).
    /// Fresh adoptions (no manifest) and up-to-date projects are never
    /// touched, so a user's own root files are safe.
    pub fn migrate_layout(
        root: &Path,
        manifest_version: Option<u32>,
        dry_run: bool,
    ) -> Result<Vec<String>> {
        let Some(v) = manifest_version else {
            return Ok(Vec::new());
        };
        if v >= TEMPLATE_VERSION {
            return Ok(Vec::new());
        }
        let mut moved = Vec::new();
        for f in MIGRATED_DOCS {
            let old = root.join(f);
            let new = root.join(".aboot").join(f);
            if old.is_file() && !new.exists() {
                if !dry_run {
                    std::fs::create_dir_all(root.join(".aboot"))?;
                    std::fs::rename(&old, &new)?;
                }
                moved.push(f.to_string());
            }
        }
        Ok(moved)
    }

    /// Files whose managed sections `aboot update` re-applies.
    pub fn managed_files(&self) -> Vec<&'static str> {
        let mut files: Vec<&'static str> = Vec::new();
        files.extend(match self.vars.agent.as_str() {
            "claude" => vec!["CLAUDE.md"],
            "both" => vec!["AGENTS.md", "CLAUDE.md"],
            _ => vec!["AGENTS.md"],
        });
        files.extend(["justfile", ".editorconfig"]);
        files
    }
}
