use serde_json::{json, Value};

/// A command's result: human-readable lines plus a JSON payload.
pub struct Output {
    pub json: Value,
    pub lines: Vec<String>,
    pub exit_code: u8,
}

impl Output {
    pub fn new(command: &str) -> Self {
        Self {
            json: json!({ "command": command }),
            lines: Vec::new(),
            exit_code: 0,
        }
    }

    pub fn line(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
}

pub struct Reporter {
    pub json: bool,
    pub quiet: bool,
}

impl Reporter {
    pub fn print(&self, out: &Output) {
        if self.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&out.json).unwrap_or_else(|_| out.json.to_string())
            );
        } else if !self.quiet {
            for line in &out.lines {
                println!("{line}");
            }
        }
    }

    pub fn print_error(&self, err: &crate::error::Error) {
        if self.json {
            println!("{}", json!({ "error": err.to_string() }));
        } else {
            eprintln!("error: {err}");
        }
    }
}
