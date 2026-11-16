use std::path::Path;

use crate::error::{Error, Result};
use crate::stacks::Stack;

/// Version stamp applied to managed sections and recorded in `.aboot.json`.
///
/// v2: the cold docs moved from the root to `.aboot/`.
/// v3: `BOARD.md` moved from the root to `.aboot/`.
pub const TEMPLATE_VERSION: u32 = 3;

pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

fn tf(path: &str, content: &str) -> TemplateFile {
    TemplateFile {
        path: path.to_string(),
        content: content.to_string(),
    }
}

/// Built-in template, embedded from `examples/` (the single source of truth).
pub fn builtin() -> Vec<TemplateFile> {
    vec![
        tf("AGENTS.md", include_str!("../examples/AGENTS.md")),
        tf(
            ".aboot/REQUIREMENTS.md",
            include_str!("../examples/.aboot/REQUIREMENTS.md"),
        ),
        tf(
            ".aboot/BACKLOG.md",
            include_str!("../examples/.aboot/BACKLOG.md"),
        ),
        tf(
            ".aboot/BOARD.md",
            include_str!("../examples/.aboot/BOARD.md"),
        ),
        tf(
            ".aboot/DECISIONS.md",
            include_str!("../examples/.aboot/DECISIONS.md"),
        ),
        tf(
            ".aboot/PROJECT.md",
            include_str!("../examples/.aboot/PROJECT.md"),
        ),
        tf(
            ".aboot/SESSIONS.md",
            include_str!("../examples/.aboot/SESSIONS.md"),
        ),
        tf("README.md", include_str!("../examples/README.md")),
        tf("justfile", include_str!("../examples/justfile")),
        tf(".editorconfig", include_str!("../examples/.editorconfig")),
    ]
}

/// Load a custom template directory (all files, recursively).
pub fn load_custom(dir: &Path) -> Result<Vec<TemplateFile>> {
    if !dir.is_dir() {
        return Err(Error::msg(format!(
            "template directory not found: {}",
            dir.display()
        )));
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<TemplateFile>) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<std::result::Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(root)
                .map_err(|_| Error::msg(format!("bad path: {}", path.display())))?
                .to_string_lossy()
                .replace('\\', "/");
            out.push(TemplateFile {
                path: rel,
                content: std::fs::read_to_string(&path)?,
            });
        }
    }
    Ok(())
}

/// Variables available to templates.
///
/// User-facing: `{{name}}`, `{{description}}`, `{{date}}`, `{{stack}}`.
/// Internal: `{{stack_section}}`, `{{justfile_commands}}`, `{{agent_file}}`.
pub struct Vars {
    pub name: String,
    pub description: String,
    pub date: String,
    pub stack: Option<Stack>,
    pub agent: String,
}

impl Vars {
    pub fn render(&self, content: &str) -> String {
        let stack = self
            .stack
            .map(|s| s.display_name())
            .unwrap_or("none")
            .to_string();
        let stack_section = self
            .stack
            .map(|s| s.commands())
            .unwrap_or("- No stack configured — add build/test/lint commands here.")
            .to_string();
        let justfile_commands = self
            .stack
            .map(|s| s.justfile_commands())
            .unwrap_or("noop:\n\t@echo \"no stack configured\"")
            .to_string();
        let agent_file = match self.agent.as_str() {
            "claude" => "CLAUDE.md",
            "both" => "AGENTS.md / CLAUDE.md",
            _ => "AGENTS.md",
        };
        let description = if self.description.is_empty() {
            "—"
        } else {
            self.description.as_str()
        };
        content
            .replace("{{name}}", &self.name)
            .replace("{{description}}", description)
            .replace("{{date}}", &self.date)
            .replace("{{stack}}", &stack)
            .replace("{{stack_section}}", &stack_section)
            .replace("{{justfile_commands}}", &justfile_commands)
            .replace("{{agent_file}}", agent_file)
    }
}

/// Insert `- [ ] <task>` right after the `## NOW` header (or append at the end).
pub fn seed_task(content: &str, task: &str) -> String {
    let item = format!("- [ ] {task}");
    let mut out: Vec<String> = Vec::new();
    let mut inserted = false;
    for line in content.lines() {
        out.push(line.to_string());
        if !inserted && line.trim() == "## NOW" {
            out.push(item.clone());
            inserted = true;
        }
    }
    if !inserted {
        if !content.ends_with('\n') {
            out.push(String::new());
        }
        out.push(item);
    }
    out.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_user_facing_variables() {
        let vars = Vars {
            name: "demo".into(),
            description: "A demo".into(),
            date: "2026-11-14".into(),
            stack: Some(Stack::Rust),
            agent: "agents".into(),
        };
        let out = vars.render("{{name}} / {{description}} / {{date}} / {{stack}}");
        assert_eq!(out, "demo / A demo / 2026-11-14 / Rust");
    }

    #[test]
    fn empty_description_becomes_placeholder() {
        let vars = Vars {
            name: "demo".into(),
            description: String::new(),
            date: "2026-11-14".into(),
            stack: None,
            agent: "agents".into(),
        };
        let out = vars.render("{{description}} / {{stack}}");
        assert_eq!(out, "— / none");
    }

    #[test]
    fn unknown_variables_are_preserved() {
        let vars = Vars {
            name: "demo".into(),
            description: String::new(),
            date: "2026-11-14".into(),
            stack: None,
            agent: "agents".into(),
        };
        assert_eq!(vars.render("{{custom}}"), "{{custom}}");
    }

    #[test]
    fn stack_section_is_rendered() {
        let vars = Vars {
            name: "demo".into(),
            description: String::new(),
            date: "2026-11-14".into(),
            stack: Some(Stack::Go),
            agent: "agents".into(),
        };
        let out = vars.render("{{stack_section}}");
        assert!(out.contains("`go build ./...`"));
    }

    #[test]
    fn seed_task_inserts_after_now() {
        let content = "# BOARD\n\n## NOW\n\n## NEXT\n\n## LATER\n";
        let out = seed_task(content, "first task");
        assert_eq!(
            out,
            "# BOARD\n\n## NOW\n- [ ] first task\n\n## NEXT\n\n## LATER\n"
        );
    }

    #[test]
    fn seed_task_appends_without_sections() {
        let content = "# BOARD\n";
        let out = seed_task(content, "task");
        assert!(out.ends_with("- [ ] task\n"));
    }
}
