use std::path::Path;

use serde_json::json;

use crate::board::{self, Item};
use crate::cli::BoardCmd;
use crate::error::{Error, Result};
use crate::output::{Output, Reporter};

pub fn run(cmd: &BoardCmd, _r: &Reporter) -> Result<Output> {
    match cmd {
        BoardCmd::List(a) => {
            let mut out = Output::new("board");
            let text = read_board(&a.path)?;
            let items = board::parse(&text);

            let mut groups: Vec<(Option<String>, Vec<&Item>)> = Vec::new();
            for item in &items {
                match groups.iter_mut().find(|(s, _)| *s == item.section) {
                    Some((_, v)) => v.push(item),
                    None => groups.push((item.section.clone(), vec![item])),
                }
            }
            for (section, group) in &groups {
                out.line(section.clone().unwrap_or_else(|| "(top)".into()));
                for item in group {
                    out.line(format!(
                        "  [{}] {}",
                        if item.done { "x" } else { " " },
                        item.text
                    ));
                }
            }

            let (done, total) = board::progress(&text);
            out.json = json!({
                "command": "board",
                "action": "list",
                "items": items.iter().map(|i| json!({
                    "index": i.index,
                    "section": i.section,
                    "done": i.done,
                    "text": i.text,
                })).collect::<Vec<_>>(),
                "done": done,
                "total": total,
            });
            Ok(out)
        }
        BoardCmd::Add {
            path,
            text,
            section,
        } => {
            let mut out = Output::new("board");
            let content = read_board(path)?;
            let item_text = text.join(" ");
            if item_text.trim().is_empty() {
                return Err(Error::msg("item text must not be empty"));
            }
            let (new_text, in_section) = board::add_item(&content, section.header(), &item_text);
            let (done, total) = board::progress(&new_text);
            std::fs::write(path.join(".aboot/BOARD.md"), new_text)?;
            if in_section {
                out.line(format!("added to {}: {item_text}", section.header()));
            } else {
                out.line(format!(
                    "appended (no {} section found): {item_text}",
                    section.header()
                ));
            }
            out.json = json!({
                "command": "board",
                "action": "add",
                "section": section.header(),
                "text": item_text,
                "done": done,
                "total": total,
            });
            Ok(out)
        }
        BoardCmd::Done { path, target } => {
            let mut out = Output::new("board");
            let content = read_board(path)?;
            let (new_text, item) = board::mark_done(&content, target)?;
            let (done, total) = board::progress(&new_text);
            std::fs::write(path.join(".aboot/BOARD.md"), new_text)?;
            out.line(format!("marked done: {}", item.text));
            out.json = json!({
                "command": "board",
                "action": "done",
                "text": item.text,
                "done": done,
                "total": total,
            });
            Ok(out)
        }
    }
}

fn read_board(path: &Path) -> Result<String> {
    let p = path.join(".aboot/BOARD.md");
    std::fs::read_to_string(&p).map_err(|_| {
        Error::msg(format!(
            "no .aboot/BOARD.md in {} (run `aboot add` first)",
            path.display()
        ))
    })
}
