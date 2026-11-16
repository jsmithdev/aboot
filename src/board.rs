use crate::error::{Error, Result};

/// A checkbox item parsed from BOARD.md.
#[derive(Clone)]
pub struct Item {
    /// 1-based position in document order.
    pub index: usize,
    /// 0-based line number in the file.
    pub line: usize,
    pub done: bool,
    pub text: String,
    pub section: Option<String>,
}

/// Parse all checkbox items, tracking their `## ` section.
pub fn parse(text: &str) -> Vec<Item> {
    let mut items = Vec::new();
    let mut section: Option<String> = None;
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("## ") {
            if !rest.starts_with('#') {
                section = Some(rest.trim().to_string());
            }
            continue;
        }
        if let Some((done, item_text)) = parse_checkbox(trimmed) {
            items.push(Item {
                index: items.len() + 1,
                line: i,
                done,
                text: item_text,
                section: section.clone(),
            });
        }
    }
    items
}

fn parse_checkbox(trimmed: &str) -> Option<(bool, String)> {
    let rest = trimmed
        .strip_prefix("- [")
        .or_else(|| trimmed.strip_prefix("* ["))?;
    let (mark, text) = rest.split_once(']')?;
    let done = mark.trim().eq_ignore_ascii_case("x");
    Some((done, text.trim().to_string()))
}

/// (done, total) checkbox counts.
pub fn progress(text: &str) -> (usize, usize) {
    let items = parse(text);
    let done = items.iter().filter(|i| i.done).count();
    (done, items.len())
}

/// Insert `- [ ] <item>` right after the `## <section>` header.
/// If the file has no such header, the item is appended at the end.
/// Returns the updated text and whether the item landed in the section.
pub fn add_item(text: &str, section: &str, item: &str) -> (String, bool) {
    let line = format!("- [ ] {item}");
    let mut out: Vec<String> = Vec::new();
    let mut inserted = false;
    for l in text.lines() {
        out.push(l.to_string());
        if !inserted && l.trim().eq_ignore_ascii_case(&format!("## {section}")) {
            out.push(line.clone());
            inserted = true;
        }
    }
    if !inserted {
        if !text.ends_with('\n') {
            out.push(String::new());
        }
        out.push(line);
    }
    (out.join("\n") + "\n", inserted)
}

/// Mark an item done, by 1-based index or by (case-insensitive) text match.
/// Returns the updated text and the item that was marked.
pub fn mark_done(text: &str, target: &str) -> Result<(String, Item)> {
    let items = parse(text);
    let target_item = if let Ok(n) = target.parse::<usize>() {
        items.get(n - 1).cloned().ok_or_else(|| {
            Error::msg(format!("no item {n} (BOARD.md has {} items)", items.len()))
        })?
    } else {
        items
            .iter()
            .find(|i| i.text.to_lowercase().contains(&target.to_lowercase()))
            .cloned()
            .ok_or_else(|| Error::msg(format!("no BOARD.md item matching '{target}'")))?
    };
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let line = &mut lines[target_item.line];
    *line = line.replacen("[ ]", "[x]", 1);
    Ok((lines.join("\n") + "\n", target_item))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# BOARD\n\n## NOW\n- [ ] first\n- [x] second\n\n## NEXT\n- [ ] third\n";

    #[test]
    fn parses_items_with_sections() {
        let items = parse(DOC);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].section.as_deref(), Some("NOW"));
        assert!(!items[0].done);
        assert!(items[1].done);
        assert_eq!(items[2].section.as_deref(), Some("NEXT"));
        assert_eq!(items[2].text, "third");
    }

    #[test]
    fn counts_progress() {
        assert_eq!(progress(DOC), (1, 3));
    }

    #[test]
    fn add_item_goes_after_header() {
        let (out, in_section) = add_item(DOC, "NEXT", "fourth");
        assert!(in_section);
        assert!(out.contains("## NEXT\n- [ ] fourth\n- [ ] third"));
    }

    #[test]
    fn add_item_appends_without_header() {
        let (out, in_section) = add_item("# BOARD\n", "NOW", "x");
        assert!(!in_section);
        assert!(out.ends_with("- [ ] x\n"));
    }

    #[test]
    fn mark_done_by_index() {
        let (out, item) = mark_done(DOC, "1").unwrap();
        assert_eq!(item.text, "first");
        assert!(out.contains("- [x] first"));
    }

    #[test]
    fn mark_done_by_text() {
        let (out, item) = mark_done(DOC, "THIRD").unwrap();
        assert_eq!(item.text, "third");
        assert!(out.contains("- [x] third"));
    }

    #[test]
    fn mark_done_missing_errors() {
        assert!(mark_done(DOC, "99").is_err());
        assert!(mark_done(DOC, "nope").is_err());
    }
}
