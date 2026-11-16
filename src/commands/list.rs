use std::path::Path;

use serde_json::{json, Value};

use crate::cli::PathArgs;
use crate::error::{Error, Result};
use crate::manifest::Manifest;
use crate::output::{Output, Reporter};

pub fn run(args: &PathArgs, _r: &Reporter) -> Result<Output> {
    let mut out = Output::new("list");
    let root = args
        .path
        .canonicalize()
        .map_err(|_| Error::msg(format!("directory not found: {}", args.path.display())))?;

    let mut projects: Vec<Value> = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&root)?.collect::<std::result::Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().to_string();
        if dir_name.starts_with('.') {
            continue;
        }
        if let Some(m) = Manifest::load(&p) {
            let (done, total) = board_progress(&p).unwrap_or((0, 0));
            projects.push(json!({
                "name": m.name,
                "path": dir_name,
                "stack": m.stack,
                "version": m.version,
                "board": { "done": done, "total": total },
                "managed": true,
            }));
        } else if p.join("AGENTS.md").is_file() && p.join(".aboot/BOARD.md").is_file() {
            let (done, total) = board_progress(&p).unwrap_or((0, 0));
            projects.push(json!({
                "name": dir_name,
                "path": dir_name,
                "stack": Value::Null,
                "version": Value::Null,
                "board": { "done": done, "total": total },
                "managed": false,
            }));
        }
    }

    if projects.is_empty() {
        out.line("no aboot projects found");
    } else {
        let headers = ["NAME", "STACK", "BOARD", "VERSION"];
        let mut rows: Vec<[String; 4]> = Vec::new();
        for pj in &projects {
            rows.push([
                pj["name"].as_str().unwrap_or("?").to_string(),
                pj["stack"].as_str().unwrap_or("-").to_string(),
                format!("{}/{}", pj["board"]["done"], pj["board"]["total"]),
                pj["version"]
                    .as_u64()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "(unmanaged)".into()),
            ]);
        }
        let mut widths = [0usize; 4];
        for (i, h) in headers.iter().enumerate() {
            widths[i] = h.len();
        }
        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = widths[i].max(cell.len());
            }
        }
        out.line(format!(
            "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}",
            headers[0],
            headers[1],
            headers[2],
            headers[3],
            w0 = widths[0],
            w1 = widths[1],
            w2 = widths[2],
            w3 = widths[3]
        ));
        for row in &rows {
            out.line(format!(
                "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}",
                row[0],
                row[1],
                row[2],
                row[3],
                w0 = widths[0],
                w1 = widths[1],
                w2 = widths[2],
                w3 = widths[3]
            ));
        }
    }

    out.json = json!({
        "command": "list",
        "path": root.to_string_lossy(),
        "projects": projects,
    });
    Ok(out)
}

fn board_progress(root: &Path) -> Option<(usize, usize)> {
    let text = std::fs::read_to_string(root.join(".aboot/BOARD.md")).ok()?;
    Some(crate::board::progress(&text))
}
