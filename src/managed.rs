use crate::error::{Error, Result};

pub const BEGIN_TOKEN: &str = "aboot:begin ";
pub const END_TOKEN: &str = "aboot:end";

/// A managed region inside a file (line-based).
///
/// Markers use the file's own comment syntax:
/// `<!-- aboot:begin ... -->` / `<!-- aboot:end -->` in markdown,
/// `# aboot:begin ...` / `# aboot:end` in justfiles and .editorconfig.
pub struct Region {
    /// 0-based index of the begin line.
    pub start_line: usize,
    /// 0-based index of the end line.
    pub end_line: usize,
    pub version: u32,
}

fn begin_header(line: &str) -> Option<&str> {
    let t = line.trim_start();
    let inner = t.strip_prefix("<!-- ").or_else(|| t.strip_prefix("# "))?;
    inner.strip_prefix(BEGIN_TOKEN)
}

fn is_end(line: &str) -> bool {
    let t = line.trim_start();
    if let Some(inner) = t.strip_prefix("<!-- ") {
        return inner.strip_suffix(" -->").map(str::trim) == Some(END_TOKEN);
    }
    t.strip_prefix("# ").map(str::trim) == Some(END_TOKEN)
}

fn parse_version(header: &str) -> Option<u32> {
    header
        .split("version=")
        .nth(1)?
        .trim_end()
        .trim_end_matches(" -->")
        .trim_matches('"')
        .parse::<u32>()
        .ok()
}

/// Find the aboot managed region in `text`, if any.
pub fn find_region(text: &str) -> Option<Region> {
    let mut start_line = None;
    let mut version = None;
    for (i, line) in text.lines().enumerate() {
        if start_line.is_none() {
            if let Some(header) = begin_header(line) {
                start_line = Some(i);
                version = Some(parse_version(header)?);
            }
        } else if is_end(line) {
            return Some(Region {
                start_line: start_line?,
                end_line: i,
                version: version?,
            });
        }
    }
    None
}

/// Extract the marker block (begin line through end line) from `text`.
pub fn extract_block(text: &str) -> Option<String> {
    let region = find_region(text)?;
    let lines: Vec<&str> = text.lines().collect();
    Some(lines[region.start_line..=region.end_line].join("\n"))
}

/// Replace the managed region with `replacement` (a full marker block).
/// Content outside the markers is preserved.
pub fn replace_region(text: &str, replacement: &str) -> Result<String> {
    let region = find_region(text).ok_or_else(|| Error::msg("no aboot managed section found"))?;
    let lines: Vec<&str> = text.lines().collect();
    let had_trailing_nl = text.ends_with('\n');
    let mut out: Vec<String> = lines[..region.start_line]
        .iter()
        .map(|s| s.to_string())
        .collect();
    out.extend(replacement.lines().map(|s| s.to_string()));
    out.extend(lines[region.end_line + 1..].iter().map(|s| s.to_string()));
    let mut out = out.join("\n");
    if had_trailing_nl {
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD_TPL: &str =
        "<!-- aboot:begin file=\"AGENTS.md\" version=\"1\" -->\nold content\n<!-- aboot:end -->";
    const JUST_TPL: &str =
        "# aboot:begin file=\"justfile\" version=\"1\"\nbuild:\n\tcargo build\n# aboot:end";

    #[test]
    fn finds_markdown_region_with_version() {
        let region = find_region(MD_TPL).expect("region");
        assert_eq!(region.version, 1);
        assert_eq!(region.start_line, 0);
        assert_eq!(region.end_line, 2);
    }

    #[test]
    fn finds_hash_comment_region() {
        let region = find_region(JUST_TPL).expect("region");
        assert_eq!(region.version, 1);
        assert_eq!(region.start_line, 0);
        assert_eq!(region.end_line, 3);
    }

    #[test]
    fn no_region_returns_none() {
        assert!(find_region("plain text").is_none());
        assert!(find_region("<!-- aboot:end -->").is_none());
        assert!(find_region("# aboot:end").is_none());
    }

    #[test]
    fn replace_preserves_outer_content() {
        let text = "header\n\n".to_string() + MD_TPL + "\n\nuser notes\n";
        let new_tpl =
            "<!-- aboot:begin file=\"AGENTS.md\" version=\"2\" -->\nnew content\n<!-- aboot:end -->";
        let out = replace_region(&text, new_tpl).unwrap();
        assert!(out.starts_with("header\n\n"));
        assert!(out.ends_with("\n\nuser notes\n"));
        assert!(out.contains("new content"));
        assert!(!out.contains("old content"));
        assert_eq!(find_region(&out).unwrap().version, 2);
    }

    #[test]
    fn replace_hash_comment_region() {
        let text = "default:\n\t@just --list\n\n".to_string() + JUST_TPL + "\n";
        let new_tpl = "# aboot:begin file=\"justfile\" version=\"2\"\nbuild:\n\tcargo build --release\n# aboot:end";
        let out = replace_region(&text, new_tpl).unwrap();
        assert!(out.starts_with("default:\n\t@just --list\n\n"));
        assert!(out.contains("cargo build --release"));
        assert_eq!(find_region(&out).unwrap().version, 2);
    }

    #[test]
    fn replace_without_region_errors() {
        assert!(replace_region("plain", "x").is_err());
    }
}
