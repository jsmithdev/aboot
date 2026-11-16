use serde_json::json;

use crate::cli::PathArgs;
use crate::error::Result;
use crate::managed;
use crate::output::{Output, Reporter};
use crate::project::{Project, MIGRATED_DOCS};
use crate::template::TEMPLATE_VERSION;

pub fn run(args: &PathArgs, _r: &Reporter) -> Result<Output> {
    let mut out = Output::new("doctor");
    let project = Project::open(&args.path, None, None)?;

    let mut errors = 0usize;
    let mut checks: Vec<(u8, String)> = Vec::new(); // 0 = ok, 1 = warn, 2 = error

    match &project.manifest {
        None => checks.push((
            1,
            "no .aboot.json manifest — run `aboot add` to adopt this project".into(),
        )),
        Some(m) => {
            if m.version < TEMPLATE_VERSION {
                checks.push((
                    1,
                    format!(
                        "template version {} is older than v{TEMPLATE_VERSION} — run `aboot update`",
                        m.version
                    ),
                ));
            } else {
                checks.push((0, format!("manifest v{} (agent: {})", m.version, m.agent)));
            }
        }
    }

    for f in project.managed_files() {
        let p = project.root.join(f);
        if !p.is_file() {
            checks.push((1, format!("{f} is missing")));
            continue;
        }
        let text = std::fs::read_to_string(&p)?;
        match managed::find_region(&text) {
            None => {
                errors += 1;
                checks.push((
                    2,
                    format!(
                        "{f} has no aboot managed markers — `aboot update` cannot manage it \
                         (use `aboot update --force` to re-adopt)"
                    ),
                ));
            }
            Some(r) if r.version < TEMPLATE_VERSION => {
                checks.push((
                    1,
                    format!(
                        "{f} managed section is v{}, current is v{TEMPLATE_VERSION} — run `aboot update`",
                        r.version
                    ),
                ));
            }
            Some(r) => checks.push((0, format!("{f} managed (v{})", r.version))),
        }
    }

    // Pre-migration layout: a v1 project still keeps its cold docs at the
    // root; one warning covers all the missing .aboot/ files.
    let pre_migration = project
        .manifest
        .as_ref()
        .is_some_and(|m| m.version < TEMPLATE_VERSION)
        && MIGRATED_DOCS.iter().any(|f| project.root.join(f).is_file());
    if pre_migration {
        checks.push((
            1,
            "pre-migration layout: workflow docs are still at the root — run `aboot update` to move them to .aboot/".into(),
        ));
    }

    for f in project.expected_files() {
        if project.managed_files().contains(&f) {
            continue;
        }
        if !project.root.join(f).is_file() {
            if pre_migration && f.starts_with(".aboot/") {
                continue; // covered by the layout warning above
            }
            checks.push((1, format!("{f} is missing")));
        }
    }

    let board_p = project.root.join(".aboot/BOARD.md");
    if board_p.is_file() {
        let (done, total) = crate::board::progress(&std::fs::read_to_string(&board_p)?);
        if total == 0 {
            checks.push((1, "BOARD.md has no checkbox items".into()));
        } else {
            checks.push((0, format!("BOARD.md has {total} items ({done} done)")));
        }
    }

    for (level, msg) in &checks {
        let mark = match level {
            0 => "✓",
            1 => "!",
            _ => "✗",
        };
        out.line(format!("{mark} {msg}"));
    }
    out.exit_code = if errors > 0 { 1 } else { 0 };
    out.json = json!({
        "command": "doctor",
        "path": project.root.to_string_lossy(),
        "ok": errors == 0,
        "checks": checks.iter().map(|(l, m)| json!({
            "level": match l { 0 => "ok", 1 => "warn", _ => "error" },
            "message": m,
        })).collect::<Vec<_>>(),
    });
    Ok(out)
}
