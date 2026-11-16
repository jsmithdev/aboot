use std::path::Path;

use serde_json::{json, Map, Value};

use crate::cli::PathArgs;
use crate::error::Result;
use crate::output::{Output, Reporter};
use crate::project::Project;
use crate::template::TEMPLATE_VERSION;

pub fn run(args: &PathArgs, _r: &Reporter) -> Result<Output> {
    let mut out = Output::new("status");
    let project = Project::open(&args.path, None, None)?;
    let manifest = project.manifest.as_ref();
    let version = manifest.map(|m| m.version).unwrap_or(0);
    // Stale only when an installed project (manifest) lags the template;
    // a project without a manifest is not installed, just unmanaged.
    let stale = manifest.is_some_and(|m| m.version < TEMPLATE_VERSION);
    let stack = project.vars.stack.map(|s| s.as_str().to_string());

    let version_str = if stale {
        format!("v{version} (out of date — run aboot update)")
    } else {
        format!("v{version}")
    };
    out.line(format!(
        "{} ({}, {})",
        project.vars.name,
        stack.as_deref().unwrap_or("no stack"),
        version_str
    ));

    let mut files = Map::new();
    for f in project.expected_files() {
        let present = project.root.join(f).is_file();
        files.insert(f.to_string(), json!(present));
        out.line(format!("  {} {}", if present { "✓" } else { "✗" }, f));
    }

    let (done, total) = board_progress(&project.root).unwrap_or((0, 0));
    out.line(format!("BOARD: {done}/{total} done"));

    out.json = json!({
        "command": "status",
        "name": project.vars.name,
        "path": project.root.to_string_lossy(),
        "stack": stack,
        "version": version,
        "stale": stale,
        "files": Value::Object(files),
        "board": { "done": done, "total": total },
    });
    Ok(out)
}

fn board_progress(root: &Path) -> Option<(usize, usize)> {
    let text = std::fs::read_to_string(root.join(".aboot/BOARD.md")).ok()?;
    Some(crate::board::progress(&text))
}
