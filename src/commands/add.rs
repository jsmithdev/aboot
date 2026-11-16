use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde_json::json;

use crate::cli::{AddArgs, AgentMode};
use crate::error::{Error, Result};
use crate::manifest::Manifest;
use crate::output::{Output, Reporter};
use crate::project::Project;
use crate::template::{self, Vars, TEMPLATE_VERSION};

pub fn run(args: &AddArgs, _r: &Reporter) -> Result<Output> {
    let mut out = Output::new("add");

    if args.md_only && args.agent_only {
        return Err(Error::msg(
            "--md-only and --agent-only are mutually exclusive",
        ));
    }
    if args.agent_only && args.task.is_some() {
        out.line("note: --agent-only does not install BOARD.md; --task ignored");
    }

    let pre_manifest = Manifest::load(&args.path);
    let description = args
        .description
        .clone()
        .or_else(|| pre_manifest.as_ref().map(|m| m.description.clone()))
        .or_else(|| infer_description(&args.path));
    let project = Project::open(&args.path, description.as_deref(), args.stack)?;
    let agent = args
        .agent
        .or_else(|| {
            pre_manifest
                .as_ref()
                .and_then(|m| AgentMode::parse(&m.agent))
        })
        .unwrap_or_default();

    // Projects installed by an older aboot keep their cold docs at the root;
    // move them into .aboot/ first, so this run never creates empty .aboot/
    // copies next to populated root files.
    let migrated = Project::migrate_layout(
        &project.root,
        pre_manifest.as_ref().map(|m| m.version),
        args.dry_run,
    )?;
    for f in &migrated {
        out.line(format!("  » {f} -> .aboot/{f} (migrate)"));
    }

    let templates = match &args.template {
        Some(dir) => template::load_custom(dir)?,
        None => template::builtin(),
    };
    let vars = Vars {
        name: project.vars.name.clone(),
        description: project.vars.description.clone(),
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        stack: project.vars.stack,
        agent: agent.as_str().to_string(),
    };

    let mut planned: Vec<(String, String, &'static str, &'static str)> = Vec::new();
    let mut have_paths: HashSet<String> = HashSet::new();

    for t in &templates {
        // `add` never installs README.md — it belongs to the project.
        if t.path == "README.md" {
            continue;
        }
        // Scope filters: --agent-only keeps just the agent file, --md-only
        // keeps just the markdown files.
        if args.agent_only && t.path != "AGENTS.md" {
            continue;
        }
        if args.md_only && !t.path.ends_with(".md") {
            continue;
        }
        let paths: Vec<String> = if t.path == "AGENTS.md" {
            agent.files().iter().map(|f| f.to_string()).collect()
        } else {
            vec![t.path.clone()]
        };
        for p in paths {
            have_paths.insert(p.clone());
            let mut content = vars.render(&t.content);
            if p == "CLAUDE.md" {
                content = content.replace("file=\"AGENTS.md\"", "file=\"CLAUDE.md\"");
            }
            // --task only seeds .aboot/BOARD.md when it is being created.
            if p == ".aboot/BOARD.md" {
                if let Some(task) = &args.task {
                    if project.root.join(".aboot/BOARD.md").exists() {
                        out.line("note: BOARD.md already exists; --task ignored");
                    } else {
                        content = template::seed_task(&content, task);
                    }
                }
            }
            planned.push((p, content, "template", "create"));
        }
    }

    // Stack files (.gitignore, build stubs) are only installed on a full add.
    if let Some(stack) = vars.stack {
        if !args.md_only && !args.agent_only {
            if !have_paths.contains(".gitignore") {
                planned.push((
                    ".gitignore".into(),
                    vars.render(stack.gitignore()),
                    "stack",
                    "create",
                ));
            }
            for (path, stub) in stack.build_stubs() {
                if !have_paths.contains(path) {
                    planned.push((path.to_string(), vars.render(stub), "stack", "create"));
                }
            }
        }
    }

    let mut wrote_any = false;
    for (path, _, _, action) in &mut planned {
        if project.root.join(path).exists() {
            *action = if args.force { "overwrite" } else { "skip" };
        }
        if *action != "skip" {
            wrote_any = true;
        }
    }
    // A no-op re-run on a project installed by an older aboot is the moment
    // to point at `aboot update` — the command that actually refreshes.
    if !wrote_any && migrated.is_empty() {
        if let Some(m) = &pre_manifest {
            if m.version < TEMPLATE_VERSION {
                out.line(format!(
                    "note: installed template v{} is older than v{TEMPLATE_VERSION} — run 'aboot update' to refresh managed sections",
                    m.version
                ));
            }
        }
    }

    let stack_str = vars.stack.map(|s| s.as_str().to_string());
    let mut manifest = match pre_manifest {
        Some(mut m) => {
            if wrote_any {
                m.version = TEMPLATE_VERSION;
            }
            if args.stack.is_some() {
                m.stack = stack_str.clone();
            }
            if args.description.is_some() {
                m.description = vars.description.clone();
            }
            m.agent = agent.as_str().to_string();
            m
        }
        None => Manifest {
            version: TEMPLATE_VERSION,
            name: vars.name.clone(),
            description: vars.description.clone(),
            stack: stack_str.clone(),
            agent: agent.as_str().to_string(),
            date: vars.date.clone(),
            files: Vec::new(),
        },
    };
    // Only recompute the installed-files list when something was written;
    // a no-op re-run must not wipe the existing list.
    if wrote_any {
        manifest.files = planned
            .iter()
            .filter(|(_, _, _, a)| *a != "skip")
            .map(|(p, _, _, _)| p.clone())
            .collect();
    }
    // A migration rewrites the manifest's file list in place (old path ->
    // new path) and bumps the version, leaving entries for files this run
    // did not touch alone.
    if !migrated.is_empty() {
        for f in &migrated {
            let new_path = format!(".aboot/{f}");
            if let Some(pos) = manifest.files.iter().position(|p| p == f) {
                manifest.files[pos] = new_path;
            } else {
                manifest.files.push(new_path);
            }
        }
        manifest.version = TEMPLATE_VERSION;
    }

    if !args.dry_run {
        for (path, content, _, action) in &planned {
            if *action == "skip" {
                continue;
            }
            let full = project.root.join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&full, content)?;
        }
        manifest.save(&project.root)?;
    }

    out.line(format!(
        "{} workflow files to '{}'{}",
        if args.dry_run {
            "Would install"
        } else {
            "Installed"
        },
        project.root.display(),
        vars.stack
            .map(|s| format!(" (stack: {})", s.display_name()))
            .unwrap_or_default()
    ));
    for (path, _, origin, action) in &planned {
        let mark = match *action {
            "create" => "+",
            "overwrite" => "~",
            _ => "-",
        };
        out.line(format!("  {mark} {path} ({origin})"));
    }
    if args.dry_run {
        out.line("dry run: nothing written");
    }

    out.json = json!({
        "command": "add",
        "path": project.root.to_string_lossy(),
        "dry_run": args.dry_run,
        "stack": stack_str,
        "agent": agent.as_str(),
        "md_only": args.md_only,
        "agent_only": args.agent_only,
        "migrated": migrated,
        "files": planned.iter().map(|(p, _, o, a)| json!({ "path": p, "origin": o, "action": a })).collect::<Vec<_>>(),
    });
    Ok(out)
}

/// First non-heading, non-empty line of README.md, as a description guess.
fn infer_description(root: &Path) -> Option<String> {
    let text = fs::read_to_string(root.join("README.md")).ok()?;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("<!--") {
            continue;
        }
        return Some(t.chars().take(120).collect());
    }
    None
}
