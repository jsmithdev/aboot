use serde_json::json;

use crate::cli::UpdateArgs;
use crate::error::{Error, Result};
use crate::managed;
use crate::output::{Output, Reporter};
use crate::project::Project;
use crate::template::{self, TEMPLATE_VERSION};

pub fn run(args: &UpdateArgs, _r: &Reporter) -> Result<Output> {
    let mut out = Output::new("update");
    let project = Project::open(&args.path, None, None)?;
    let manifest = project
        .manifest
        .clone()
        .ok_or_else(|| Error::msg("no .aboot.json manifest — run `aboot add` first"))?;
    let templates = template::builtin();

    // Pre-migration projects keep their cold docs at the root; move them
    // into .aboot/ before re-applying managed sections.
    let migrated = Project::migrate_layout(&project.root, Some(manifest.version), args.dry_run)?;
    for f in &migrated {
        out.line(format!("moved: {f} -> .aboot/{f}"));
    }

    let mut updated: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for f in project.managed_files() {
        let tpl = if f == "CLAUDE.md" {
            templates.iter().find(|t| t.path == "AGENTS.md")
        } else {
            templates.iter().find(|t| t.path == f)
        }
        .ok_or_else(|| Error::msg(format!("no built-in template for {f}")))?;
        let mut rendered = project.vars.render(&tpl.content);
        if f == "CLAUDE.md" {
            rendered = rendered.replace("file=\"AGENTS.md\"", "file=\"CLAUDE.md\"");
        }
        // Only the marker block replaces the existing region; anything the
        // template keeps outside the markers (e.g. the file heading) is not
        // re-applied on top of user content.
        let block = managed::extract_block(&rendered).unwrap_or_else(|| rendered.clone());

        let p = project.root.join(f);
        if !p.is_file() {
            if !args.dry_run {
                std::fs::write(&p, &rendered)?;
            }
            updated.push(format!("{f} (recreated)"));
            continue;
        }
        let existing = std::fs::read_to_string(&p)?;
        match managed::find_region(&existing) {
            None => {
                if args.force {
                    if !args.dry_run {
                        std::fs::write(&p, &rendered)?;
                    }
                    updated.push(format!("{f} (re-adopted)"));
                } else {
                    skipped.push(format!("{f} (no managed markers; use --force)"));
                }
            }
            Some(_) => {
                let new = managed::replace_region(&existing, &block)?;
                if new != existing {
                    if !args.dry_run {
                        std::fs::write(&p, new)?;
                    }
                    updated.push(f.to_string());
                } else {
                    skipped.push(format!("{f} (up to date)"));
                }
            }
        }
    }

    if !args.dry_run {
        let mut m = manifest.clone();
        m.version = TEMPLATE_VERSION;
        m.files = project
            .expected_files()
            .into_iter()
            .filter(|f| project.root.join(f).is_file())
            .map(|s| s.to_string())
            .collect();
        m.save(&project.root)?;
    }

    for u in &updated {
        out.line(format!("updated: {u}"));
    }
    for s in &skipped {
        out.line(format!("skipped: {s}"));
    }
    if args.dry_run {
        out.line("dry run: nothing written");
    }

    out.json = json!({
        "command": "update",
        "path": project.root.to_string_lossy(),
        "dry_run": args.dry_run,
        "version": TEMPLATE_VERSION,
        "migrated": migrated,
        "updated": updated,
        "skipped": skipped,
    });
    Ok(out)
}
