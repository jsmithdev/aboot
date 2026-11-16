use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::cli::NewArgs;
use crate::error::{Error, Result};
use crate::git;
use crate::licenses;
use crate::manifest::Manifest;
use crate::output::{Output, Reporter};
use crate::template::{self, Vars, TEMPLATE_VERSION};

struct Planned {
    path: String,
    content: String,
    origin: &'static str,
    action: &'static str,
}

pub fn run(args: &NewArgs, _r: &Reporter) -> Result<Output> {
    let mut out = Output::new("new");
    validate_name(&args.name)?;

    let target = PathBuf::from(&args.name);
    if target.exists() {
        if !target.is_dir() {
            return Err(Error::msg(format!(
                "'{}' exists and is not a directory",
                args.name
            )));
        }
        if !args.force && !is_empty_dir(&target)? {
            return Err(Error::msg(format!(
                "target directory '{}' exists and is not empty (use --force)",
                args.name
            )));
        }
    }

    let templates = match &args.template {
        Some(dir) => template::load_custom(dir)?,
        None => template::builtin(),
    };

    let agent = args.agent.unwrap_or_default();
    let vars = Vars {
        name: args.name.clone(),
        description: args.description.clone().unwrap_or_default(),
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        stack: args.stack,
        agent: agent.as_str().to_string(),
    };

    let mut planned: Vec<Planned> = Vec::new();
    let mut have_paths: HashSet<String> = HashSet::new();

    for t in &templates {
        // The AGENTS.md template is routed to AGENTS.md and/or CLAUDE.md.
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
            if p == ".aboot/BOARD.md" {
                if let Some(task) = &args.task {
                    content = template::seed_task(&content, task);
                }
            }
            planned.push(Planned {
                path: p,
                content,
                origin: "template",
                action: "create",
            });
        }
    }

    if let Some(stack) = args.stack {
        if !have_paths.contains(".gitignore") {
            planned.push(Planned {
                path: ".gitignore".into(),
                content: vars.render(stack.gitignore()),
                origin: "stack",
                action: "create",
            });
        }
        for (path, stub) in stack.build_stubs() {
            if !have_paths.contains(path) {
                planned.push(Planned {
                    path: path.into(),
                    content: vars.render(stub),
                    origin: "stack",
                    action: "create",
                });
            }
        }
    }

    if let Some(license) = &args.license {
        if !have_paths.contains("LICENSE") {
            let year = chrono::Local::now().format("%Y").to_string();
            planned.push(Planned {
                path: "LICENSE".into(),
                content: licenses::render_license(license, &year, &args.name),
                origin: "license",
                action: "create",
            });
        }
    }

    let manifest = Manifest {
        version: TEMPLATE_VERSION,
        name: args.name.clone(),
        description: vars.description.clone(),
        stack: args.stack.map(|s| s.as_str().to_string()),
        agent: agent.as_str().to_string(),
        date: vars.date.clone(),
        files: planned.iter().map(|p| p.path.clone()).collect(),
    };
    planned.push(Planned {
        path: ".aboot.json".into(),
        content: serde_json::to_string_pretty(&manifest)? + "\n",
        origin: "manifest",
        action: "create",
    });

    for p in &mut planned {
        if target.join(&p.path).exists() {
            p.action = if args.force { "overwrite" } else { "skip" };
        }
    }

    let mut git_info: Option<Value> = None;
    if !args.dry_run {
        fs::create_dir_all(&target)?;
        for p in &planned {
            if p.action == "skip" {
                continue;
            }
            let full = target.join(&p.path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&full, &p.content)?;
        }

        if !args.no_git {
            git::init(&target)?;
            git::add_all(&target)?;
            let committed = match git::commit(
                &target,
                &format!("chore: scaffold {} with aboot", args.name),
            ) {
                Ok(()) => true,
                Err(e) => {
                    out.line(format!(
                        "warning: {e} (files are staged; run `git commit` to finish)"
                    ));
                    false
                }
            };
            let remote = match &args.git_remote {
                Some(url) => {
                    git::add_remote(&target, url)?;
                    Some(url.clone())
                }
                None => None,
            };
            git_info = Some(json!({
                "initialized": true,
                "committed": committed,
                "remote": remote,
            }));
        }
    }

    out.line(format!(
        "{} project '{}' at {}{}",
        if args.dry_run {
            "Would create"
        } else {
            "Creating"
        },
        args.name,
        target.display(),
        args.stack
            .map(|s| format!(" (stack: {})", s.display_name()))
            .unwrap_or_default()
    ));
    for p in &planned {
        let mark = match p.action {
            "create" => "+",
            "overwrite" => "~",
            _ => "-",
        };
        out.line(format!("  {mark} {}", p.path));
    }
    if args.dry_run {
        out.line("dry run: nothing written");
    }
    if let Some(g) = &git_info {
        let msg = if g["committed"].as_bool() == Some(true) {
            "git: initialized, committed"
        } else {
            "git: initialized (commit pending)"
        };
        out.line(msg);
        if let Some(url) = g["remote"].as_str() {
            out.line(format!("git: remote origin -> {url}"));
        }
    }

    let files: Vec<Value> = planned
        .iter()
        .map(|p| json!({ "path": p.path, "origin": p.origin, "action": p.action }))
        .collect();
    out.json = json!({
        "command": "new",
        "name": args.name,
        "path": target.to_string_lossy(),
        "dry_run": args.dry_run,
        "stack": args.stack.map(|s| s.as_str().to_string()),
        "agent": agent.as_str(),
        "files": files,
        "git": git_info,
    });
    Ok(out)
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::msg("project name must not be empty"));
    }
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err(Error::msg(format!("invalid project name: {name}")));
    }
    if name.starts_with('.') {
        return Err(Error::msg(format!(
            "project name must not start with '.': {name}"
        )));
    }
    Ok(())
}

fn is_empty_dir(dir: &PathBuf) -> Result<bool> {
    Ok(fs::read_dir(dir)?.next().is_none())
}
