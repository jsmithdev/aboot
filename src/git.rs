use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

fn run(dir: &Path, args: &[&str]) -> Result<std::process::Output> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| Error::msg(format!("failed to run git ({e}); is git installed?")))?;
    if !out.status.success() {
        return Err(Error::Git(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(out)
}

/// `git init` with an initial `main` branch (falls back for old git).
pub fn init(dir: &Path) -> Result<()> {
    match run(dir, &["init", "-b", "main"]) {
        Ok(_) => Ok(()),
        Err(_) => {
            run(dir, &["init"])?;
            let _ = run(dir, &["symbolic-ref", "HEAD", "refs/heads/main"]);
            Ok(())
        }
    }
}

pub fn add_all(dir: &Path) -> Result<()> {
    run(dir, &["add", "-A"]).map(|_| ())
}

pub fn commit(dir: &Path, msg: &str) -> Result<()> {
    run(dir, &["commit", "-m", msg]).map(|_| ())
}

pub fn add_remote(dir: &Path, url: &str) -> Result<()> {
    run(dir, &["remote", "add", "origin", url]).map(|_| ())
}
