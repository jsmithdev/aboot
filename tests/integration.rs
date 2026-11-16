use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn aboot() -> Command {
    let mut cmd = Command::cargo_bin("aboot").expect("aboot binary");
    cmd.env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com");
    cmd
}

fn read(dir: &Path, file: &str) -> String {
    fs::read_to_string(dir.join(file)).unwrap_or_else(|e| panic!("read {file}: {e}"))
}

fn json_of(cmd: &mut Command) -> Value {
    let out = cmd.arg("--json").output().expect("run aboot");
    assert!(
        out.status.success(),
        "expected success, got {:?}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("valid JSON output")
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

// ---------- aboot new ----------

#[test]
fn new_creates_project_with_workflow_files() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Creating project 'demo'"));

    let dir = tmp.path().join("demo");
    for f in [
        "AGENTS.md",
        ".aboot/REQUIREMENTS.md",
        ".aboot/BACKLOG.md",
        ".aboot/BOARD.md",
        ".aboot/DECISIONS.md",
        ".aboot/PROJECT.md",
        ".aboot/SESSIONS.md",
        "README.md",
        "justfile",
        ".editorconfig",
        ".aboot.json",
    ] {
        assert!(dir.join(f).exists(), "{f} missing");
    }
    assert!(dir.join(".git").exists(), "git repo initialized");
    let log = git(&dir, &["log", "--oneline"]);
    assert!(log.contains("scaffold demo with aboot"));

    let manifest: Value = serde_json::from_str(&read(&dir, ".aboot.json")).unwrap();
    assert_eq!(manifest["name"], "demo");
    assert_eq!(manifest["version"], 3);
}

#[test]
fn new_renders_variables_and_seeds_task() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--description")
        .arg("A demo project")
        .arg("--task")
        .arg("Write the first test")
        .arg("--stack")
        .arg("rust")
        .current_dir(tmp.path())
        .assert()
        .success();

    let dir = tmp.path().join("demo");
    let readme = read(&dir, "README.md");
    assert!(readme.starts_with("# demo"));
    assert!(!readme.contains("{{name}}"));

    let agents = read(&dir, "AGENTS.md");
    assert!(agents.contains("- Name: demo"));
    assert!(agents.contains("- Description: A demo project"));
    assert!(agents.contains("- Stack: Rust"));
    assert!(agents.contains("cargo build"));

    let board = read(&dir, ".aboot/BOARD.md");
    assert!(board.contains("## NOW\n- [ ] Write the first test"));
}

#[test]
fn new_stack_presets_create_build_files() {
    let cases = [
        ("rust", "Cargo.toml", "name = \"demo\""),
        ("typescript", "package.json", "\"name\": \"demo\""),
        ("typescript", "tsconfig.json", "strict"),
        ("python", "pyproject.toml", "name = \"demo\""),
        ("go", "go.mod", "module demo"),
    ];
    for (stack, file, needle) in cases {
        let tmp = tempfile::tempdir().unwrap();
        aboot()
            .arg("new")
            .arg("demo")
            .arg("--stack")
            .arg(stack)
            .arg("--no-git")
            .current_dir(tmp.path())
            .assert()
            .success();
        let dir = tmp.path().join("demo");
        assert!(dir.join(".gitignore").exists(), "{stack}: .gitignore");
        let content = read(&dir, file);
        assert!(content.contains(needle), "{stack}: {file} content");
    }
}

#[test]
fn new_no_git_skips_repository() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    assert!(!tmp.path().join("demo").join(".git").exists());
}

#[test]
fn new_git_remote_is_added() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--git-remote")
        .arg("https://example.com/demo.git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");
    let remotes = git(&dir, &["remote", "-v"]);
    assert!(remotes.contains("https://example.com/demo.git"));
}

#[test]
fn new_license_mit_is_embedded() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--license")
        .arg("MIT")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let license = read(&tmp.path().join("demo"), "LICENSE");
    assert!(license.contains("MIT License"));
    assert!(license.contains("demo"));
}

#[test]
fn new_license_unknown_spdx_gets_pointer() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--license")
        .arg("EUPL-1.2")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let license = read(&tmp.path().join("demo"), "LICENSE");
    assert!(license.contains("https://spdx.org/licenses/EUPL-1.2"));
}

#[test]
fn new_agent_modes() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("claude-only")
        .arg("--agent")
        .arg("claude")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("claude-only");
    assert!(dir.join("CLAUDE.md").exists());
    assert!(!dir.join("AGENTS.md").exists());
    assert!(read(&dir, "CLAUDE.md").contains("file=\"CLAUDE.md\""));

    aboot()
        .arg("new")
        .arg("dual")
        .arg("--agent")
        .arg("both")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("dual");
    assert!(dir.join("CLAUDE.md").exists());
    assert!(dir.join("AGENTS.md").exists());
}

#[test]
fn new_dry_run_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("ghost")
        .arg("--dry-run")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));
    assert!(!tmp.path().join("ghost").exists());
}

#[test]
fn new_into_nonempty_directory_fails() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir(tmp.path().join("demo")).unwrap();
    fs::write(tmp.path().join("demo").join("x.txt"), "x").unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn new_force_overwrites_existing_files() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");
    fs::write(dir.join(".aboot/PROJECT.md"), "user edit\n").unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--force")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    assert!(!read(&dir, ".aboot/PROJECT.md").contains("user edit"));
}

#[test]
fn new_custom_template_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let tpl = tmp.path().join("mytpl");
    fs::create_dir(&tpl).unwrap();
    fs::write(tpl.join("README.md"), "# {{name}} custom\n").unwrap();
    fs::write(tpl.join("AGENTS.md"), "RULES\n- be nice\n").unwrap();

    aboot()
        .arg("new")
        .arg("custom")
        .arg("--template")
        .arg(&tpl)
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("custom");
    assert_eq!(read(&dir, "README.md"), "# custom custom\n");
    assert!(read(&dir, "AGENTS.md").contains("be nice"));
    // manifest is still written
    assert!(dir.join(".aboot.json").exists());
}

#[test]
fn new_rejects_invalid_names() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("a/b")
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid project name"));
}

#[test]
fn new_json_output_is_valid() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cmd = aboot();
    cmd.arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path());
    let json = json_of(&mut cmd);
    assert_eq!(json["command"], "new");
    assert_eq!(json["name"], "demo");
    let files = json["files"].as_array().unwrap();
    assert!(files.iter().any(|f| f["path"] == "AGENTS.md"));
    assert!(files.iter().all(|f| f["action"] == "create"));
}

#[test]
fn new_quiet_suppresses_output() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .arg("--quiet")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// ---------- aboot add ----------

#[test]
fn add_installs_workflow_files_into_existing_project() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("README.md"), "# Legacy\n\nAn old project.\n").unwrap();
    // A user's own root BOARD.md (no manifest) is not an aboot file.
    fs::write(dir.join("BOARD.md"), "# BOARD\n- [ ] old item\n").unwrap();

    aboot().arg("add").arg(&dir).assert().success();

    for f in [
        "AGENTS.md",
        ".aboot/REQUIREMENTS.md",
        ".aboot/BACKLOG.md",
        ".aboot/BOARD.md",
        ".aboot/DECISIONS.md",
        ".aboot/PROJECT.md",
        ".aboot/SESSIONS.md",
        "justfile",
        ".editorconfig",
        ".aboot.json",
    ] {
        assert!(dir.join(f).exists(), "{f} missing");
    }
    // README and the user's root board are never touched; the template
    // board is installed under .aboot/.
    assert_eq!(read(&dir, "README.md"), "# Legacy\n\nAn old project.\n");
    assert_eq!(read(&dir, "BOARD.md"), "# BOARD\n- [ ] old item\n");
    assert!(read(&dir, ".aboot/BOARD.md").contains("## NOW"));
    // description inferred from README
    let manifest: Value = serde_json::from_str(&read(&dir, ".aboot.json")).unwrap();
    assert_eq!(manifest["description"], "An old project.");
}

#[test]
fn add_never_overwrites_by_default() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("AGENTS.md"), "MY OWN RULES\n").unwrap();

    aboot().arg("add").arg(&dir).assert().success();
    assert_eq!(read(&dir, "AGENTS.md"), "MY OWN RULES\n");
}

#[test]
fn add_force_overwrites() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("AGENTS.md"), "MY OWN RULES\n").unwrap();

    aboot()
        .arg("add")
        .arg(&dir)
        .arg("--force")
        .assert()
        .success();
    assert!(read(&dir, "AGENTS.md").contains("Agent Instructions"));
}

#[test]
fn add_dry_run_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();
    aboot()
        .arg("add")
        .arg(&dir)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));
    assert!(!dir.join("AGENTS.md").exists());
    assert!(!dir.join(".aboot.json").exists());
}

#[test]
fn add_detects_stack_from_build_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("go.mod"), "module legacy\n\ngo 1.22\n").unwrap();

    aboot().arg("add").arg(&dir).assert().success();
    let manifest: Value = serde_json::from_str(&read(&dir, ".aboot.json")).unwrap();
    assert_eq!(manifest["stack"], "go");
    assert!(dir.join(".gitignore").exists());
    // existing go.mod is not clobbered
    assert_eq!(read(&dir, "go.mod"), "module legacy\n\ngo 1.22\n");
}

#[test]
fn add_task_seeds_only_new_board_md() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();

    aboot()
        .arg("add")
        .arg(&dir)
        .arg("--task")
        .arg("first")
        .assert()
        .success();
    assert!(read(&dir, ".aboot/BOARD.md").contains("- [ ] first"));

    fs::write(dir.join(".aboot/BOARD.md"), "# BOARD\n- [ ] keep\n").unwrap();
    aboot()
        .arg("add")
        .arg(&dir)
        .arg("--task")
        .arg("second")
        .assert()
        .success()
        .stdout(predicate::str::contains("--task ignored"));
    assert_eq!(read(&dir, ".aboot/BOARD.md"), "# BOARD\n- [ ] keep\n");
}

#[test]
fn add_md_only_installs_markdown_files_only() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();

    aboot()
        .arg("add")
        .arg(&dir)
        .arg("--stack")
        .arg("rust")
        .arg("--md-only")
        .arg("--task")
        .arg("first")
        .assert()
        .success();

    for f in [
        "AGENTS.md",
        ".aboot/REQUIREMENTS.md",
        ".aboot/BACKLOG.md",
        ".aboot/BOARD.md",
        ".aboot/DECISIONS.md",
        ".aboot/PROJECT.md",
        ".aboot/SESSIONS.md",
        ".aboot.json",
    ] {
        assert!(dir.join(f).exists(), "{f} missing");
    }
    for f in ["justfile", ".editorconfig", ".gitignore", "Cargo.toml"] {
        assert!(!dir.join(f).exists(), "{f} should not be installed");
    }
    // --task still seeds .aboot/BOARD.md, which is a markdown file
    assert!(read(&dir, ".aboot/BOARD.md").contains("- [ ] first"));
}

#[test]
fn add_agent_only_installs_agent_file_only() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();

    aboot()
        .arg("add")
        .arg(&dir)
        .arg("--agent-only")
        .assert()
        .success();

    assert!(dir.join("AGENTS.md").exists());
    for f in [
        ".aboot/REQUIREMENTS.md",
        ".aboot/BACKLOG.md",
        ".aboot/BOARD.md",
        ".aboot/DECISIONS.md",
        ".aboot/PROJECT.md",
        ".aboot/SESSIONS.md",
        "justfile",
        ".editorconfig",
    ] {
        assert!(!dir.join(f).exists(), "{f} should not be installed");
    }
    let manifest: Value = serde_json::from_str(&read(&dir, ".aboot.json")).unwrap();
    assert_eq!(manifest["files"][0], "AGENTS.md");
    assert_eq!(manifest["files"].as_array().unwrap().len(), 1);
}

#[test]
fn add_agent_only_ignores_task() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();

    aboot()
        .arg("add")
        .arg(&dir)
        .arg("--agent-only")
        .arg("--task")
        .arg("first")
        .assert()
        .success()
        .stdout(predicate::str::contains("--task ignored"));
    assert!(!dir.join(".aboot/BOARD.md").exists());
}

#[test]
fn add_md_only_and_agent_only_are_mutually_exclusive() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();

    aboot()
        .arg("add")
        .arg(&dir)
        .arg("--md-only")
        .arg("--agent-only")
        .assert()
        .failure()
        .stderr(predicate::str::contains("mutually exclusive"));
    assert!(!dir.join("AGENTS.md").exists());
}

#[test]
fn add_notes_stale_template_on_noop_rerun() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");

    // Up-to-date manifest: a re-run is a silent no-op.
    aboot()
        .arg("add")
        .arg(".")
        .current_dir(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("note: installed template").not());

    // Age the manifest: a re-run points at `aboot update`.
    let manifest = read(&dir, ".aboot.json");
    fs::write(
        dir.join(".aboot.json"),
        manifest.replace("\"version\": 3", "\"version\": 0"),
    )
    .unwrap();
    aboot()
        .arg("add")
        .arg(".")
        .current_dir(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "note: installed template v0 is older than v3 — run 'aboot update' to refresh managed sections",
        ));
}

#[test]
fn add_migrates_v1_layout_to_aboot_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();
    // Simulate a v1 install: root-level cold docs plus a v1 manifest.
    for f in [
        "REQUIREMENTS.md",
        "BACKLOG.md",
        "DECISIONS.md",
        "PROJECT.md",
        "SESSIONS.md",
    ] {
        fs::write(dir.join(f), format!("user {f}\n")).unwrap();
    }
    fs::write(dir.join("BOARD.md"), "user BOARD.md\n").unwrap();
    fs::write(
        dir.join(".aboot.json"),
        r#"{"version": 1, "name": "legacy", "description": "", "agent": "agents", "date": "2026-11-01", "files": ["AGENTS.md", "BOARD.md", "REQUIREMENTS.md", "BACKLOG.md", "DECISIONS.md", "PROJECT.md", "SESSIONS.md", "justfile", ".editorconfig"]}"#,
    )
    .unwrap();

    aboot()
        .arg("add")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "REQUIREMENTS.md -> .aboot/REQUIREMENTS.md",
        ));

    // Content preserved, root copies gone.
    assert_eq!(
        read(&dir, ".aboot/REQUIREMENTS.md"),
        "user REQUIREMENTS.md\n"
    );
    assert_eq!(read(&dir, ".aboot/SESSIONS.md"), "user SESSIONS.md\n");
    assert_eq!(read(&dir, ".aboot/BOARD.md"), "user BOARD.md\n");
    for f in [
        "REQUIREMENTS.md",
        "BACKLOG.md",
        "DECISIONS.md",
        "PROJECT.md",
        "SESSIONS.md",
    ] {
        assert!(!dir.join(f).exists(), "{f} should have been moved");
    }
    assert!(
        !dir.join("BOARD.md").exists(),
        "BOARD.md should have been moved"
    );
    // Manifest bumped, file list rewritten in place.
    let manifest: Value = serde_json::from_str(&read(&dir, ".aboot.json")).unwrap();
    assert_eq!(manifest["version"], 3);
    let files = manifest["files"].as_array().unwrap();
    assert!(files.iter().any(|f| f == ".aboot/REQUIREMENTS.md"));
    assert!(!files.iter().any(|f| f == "REQUIREMENTS.md"));
    assert!(files.iter().any(|f| f == ".aboot/BOARD.md"));
    assert!(!files.iter().any(|f| f == "BOARD.md"));
}

#[test]
fn add_fresh_adoption_leaves_user_root_files_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();
    // No manifest: a user's own root files are not aboot files.
    fs::write(dir.join("REQUIREMENTS.md"), "my own notes\n").unwrap();
    fs::write(dir.join("BOARD.md"), "# BOARD\n- [ ] mine\n").unwrap();

    aboot().arg("add").arg(&dir).assert().success();

    assert_eq!(read(&dir, "REQUIREMENTS.md"), "my own notes\n");
    assert!(dir.join(".aboot/REQUIREMENTS.md").exists());
    assert_eq!(read(&dir, "BOARD.md"), "# BOARD\n- [ ] mine\n");
    assert!(dir.join(".aboot/BOARD.md").exists());
}

#[test]
fn add_noop_rerun_preserves_manifest_files() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");

    let before: Value = serde_json::from_str(&read(&dir, ".aboot.json")).unwrap();
    assert!(!before["files"].as_array().unwrap().is_empty());

    aboot()
        .arg("add")
        .arg(".")
        .current_dir(&dir)
        .assert()
        .success();

    let after: Value = serde_json::from_str(&read(&dir, ".aboot.json")).unwrap();
    assert_eq!(
        after["files"], before["files"],
        "no-op re-run must not empty the manifest files list"
    );
}

// ---------- aboot status / doctor / list ----------

#[test]
fn status_reports_files_and_progress() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--task")
        .arg("t1")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");

    aboot()
        .arg("status")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("BOARD: 0/1 done"));

    let mut cmd = aboot();
    cmd.arg("status").arg(&dir);
    let json = json_of(&mut cmd);
    assert_eq!(json["name"], "demo");
    assert_eq!(json["files"]["AGENTS.md"], true);
    assert_eq!(json["files"][".aboot/REQUIREMENTS.md"], true);
    assert_eq!(json["board"]["done"], 0);
    assert_eq!(json["board"]["total"], 1);
}

#[test]
fn status_marks_stale_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");

    // Fresh manifest: no staleness marker.
    aboot()
        .arg("status")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("out of date").not());

    // Age the manifest: the version line carries the staleness marker.
    let manifest = read(&dir, ".aboot.json");
    fs::write(
        dir.join(".aboot.json"),
        manifest.replace("\"version\": 3", "\"version\": 0"),
    )
    .unwrap();

    aboot()
        .arg("status")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "v0 (out of date — run aboot update)",
        ));

    let mut cmd = aboot();
    cmd.arg("status").arg(&dir);
    let json = json_of(&mut cmd);
    assert_eq!(json["stale"], true);
}

#[test]
fn doctor_passes_on_fresh_project() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    aboot()
        .arg("doctor")
        .arg(tmp.path().join("demo"))
        .assert()
        .success();
}

#[test]
fn doctor_fails_on_unmanaged_file() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");
    // Strip the managed markers from the justfile.
    let content = read(&dir, "justfile");
    let stripped: Vec<&str> = content
        .lines()
        .filter(|l| !l.contains("aboot:begin") && !l.contains("aboot:end"))
        .collect();
    fs::write(dir.join("justfile"), stripped.join("\n") + "\n").unwrap();

    aboot().arg("doctor").arg(&dir).assert().failure();
}

#[test]
fn doctor_warns_on_missing_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("plain");
    fs::create_dir(&dir).unwrap();
    fs::create_dir_all(dir.join(".aboot")).unwrap();
    fs::write(dir.join(".aboot/BOARD.md"), "# BOARD\n- [ ] x\n").unwrap();
    aboot()
        .arg("doctor")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("no .aboot.json manifest"));
}

#[test]
fn doctor_flags_pre_migration_layout() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("REQUIREMENTS.md"), "user requirements\n").unwrap();
    fs::write(
        dir.join(".aboot.json"),
        r#"{"version": 1, "name": "legacy", "description": "", "agent": "agents", "date": "2026-11-01", "files": []}"#,
    )
    .unwrap();

    aboot()
        .arg("doctor")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("pre-migration layout"));
}

#[test]
fn list_finds_managed_and_unmanaged_projects() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("managed")
        .arg("--stack")
        .arg("rust")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let unmanaged = tmp.path().join("unmanaged");
    fs::create_dir(&unmanaged).unwrap();
    fs::write(unmanaged.join("AGENTS.md"), "rules\n").unwrap();
    fs::create_dir_all(unmanaged.join(".aboot")).unwrap();
    fs::write(unmanaged.join(".aboot/BOARD.md"), "# BOARD\n- [ ] x\n").unwrap();

    aboot()
        .arg("list")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("managed")
                .and(predicate::str::contains("rust"))
                .and(predicate::str::contains("unmanaged")),
        );

    let mut cmd = aboot();
    cmd.arg("list").arg(tmp.path());
    let json = json_of(&mut cmd);
    let projects = json["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0]["name"], "managed");
    assert_eq!(projects[0]["stack"], "rust");
    assert_eq!(projects[1]["managed"], false);
}

// ---------- aboot board ----------

#[test]
fn board_add_done_list_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");

    aboot()
        .arg("board")
        .arg("add")
        .arg("first task")
        .arg("--path")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("added to NOW"));
    aboot()
        .arg("board")
        .arg("add")
        .arg("second")
        .arg("--path")
        .arg(&dir)
        .arg("--section")
        .arg("later")
        .assert()
        .success();
    aboot()
        .arg("board")
        .arg("done")
        .arg("1")
        .arg("--path")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("marked done: first task"));

    let board = read(&dir, ".aboot/BOARD.md");
    assert!(board.contains("## NOW\n- [x] first task"));
    assert!(board.contains("## LATER\n- [ ] second"));

    aboot()
        .arg("board")
        .arg("list")
        .arg(&dir)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[x] first task").and(predicate::str::contains("[ ] second")),
        );
}

#[test]
fn board_done_by_text_match() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--task")
        .arg("unique thing")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");
    aboot()
        .arg("board")
        .arg("done")
        .arg("UNIQUE")
        .arg("--path")
        .arg(&dir)
        .assert()
        .success();
    assert!(read(&dir, ".aboot/BOARD.md").contains("- [x] unique thing"));
}

#[test]
fn board_missing_board_md_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("empty");
    fs::create_dir(&dir).unwrap();
    aboot()
        .arg("board")
        .arg("list")
        .arg(&dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("no .aboot/BOARD.md"));
}

// ---------- aboot update ----------

#[test]
fn update_restores_managed_section_and_preserves_user_content() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--stack")
        .arg("rust")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");

    // User edits inside and outside the managed section.
    let mut content = read(&dir, "AGENTS.md");
    content = content.replace("cargo build", "cargo build --release");
    content.push_str("\n## My custom section\n- user content\n");
    fs::write(dir.join("AGENTS.md"), &content).unwrap();

    aboot()
        .arg("update")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("updated: AGENTS.md"));

    let restored = read(&dir, "AGENTS.md");
    assert!(restored.contains("- Build: `cargo build`\n"));
    assert!(!restored.contains("cargo build --release"));
    assert!(restored.contains("## My custom section\n- user content\n"));
}

#[test]
fn update_dry_run_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");
    let before = read(&dir, "AGENTS.md");
    aboot()
        .arg("update")
        .arg(&dir)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));
    assert_eq!(read(&dir, "AGENTS.md"), before);
}

#[test]
fn update_requires_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("plain");
    fs::create_dir(&dir).unwrap();
    aboot().arg("update").arg(&dir).assert().failure();
}

#[test]
fn update_recreates_missing_managed_file() {
    let tmp = tempfile::tempdir().unwrap();
    aboot()
        .arg("new")
        .arg("demo")
        .arg("--no-git")
        .current_dir(tmp.path())
        .assert()
        .success();
    let dir = tmp.path().join("demo");
    fs::remove_file(dir.join("justfile")).unwrap();
    aboot()
        .arg("update")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("justfile (recreated)"));
    assert!(dir.join("justfile").exists());
}

#[test]
fn update_migrates_v1_layout_to_aboot_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("legacy");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("REQUIREMENTS.md"), "user requirements\n").unwrap();
    fs::write(dir.join("BACKLOG.md"), "user backlog\n").unwrap();
    fs::write(dir.join("BOARD.md"), "user board\n").unwrap();
    fs::write(
        dir.join(".aboot.json"),
        r#"{"version": 1, "name": "legacy", "description": "", "agent": "agents", "date": "2026-11-01", "files": ["REQUIREMENTS.md", "BACKLOG.md", "BOARD.md"]}"#,
    )
    .unwrap();

    aboot()
        .arg("update")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "moved: REQUIREMENTS.md -> .aboot/REQUIREMENTS.md",
        ));

    assert_eq!(read(&dir, ".aboot/REQUIREMENTS.md"), "user requirements\n");
    assert_eq!(read(&dir, ".aboot/BACKLOG.md"), "user backlog\n");
    assert_eq!(read(&dir, ".aboot/BOARD.md"), "user board\n");
    assert!(!dir.join("REQUIREMENTS.md").exists());
    assert!(!dir.join("BACKLOG.md").exists());
    assert!(!dir.join("BOARD.md").exists());
    let manifest: Value = serde_json::from_str(&read(&dir, ".aboot.json")).unwrap();
    assert_eq!(manifest["version"], 3);
}
