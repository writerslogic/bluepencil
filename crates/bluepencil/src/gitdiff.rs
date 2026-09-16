use std::ops::RangeInclusive;
use std::path::Path;

use anyhow::{Result, bail};

/// Line ranges (1-based, inclusive) added or modified in `path` since `since`, read from
/// `git diff --unified=0`'s hunk headers. A hunk with zero added lines (a pure deletion)
/// contributes nothing, since there's nothing left in the current file to check.
pub fn changed_lines(since: &str, path: &Path) -> Result<Vec<RangeInclusive<usize>>> {
    let output = std::process::Command::new("git")
        .args(["diff", "--unified=0", "--no-color", since, "--", path.to_str().unwrap_or_default()])
        .output()?;
    if !output.status.success() {
        bail!("git diff failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.lines().filter_map(parse_hunk_header).collect())
}

/// Parses `@@ -a,b +c,d @@ ...` for the `+c,d` side. `d` defaults to 1 when omitted
/// (a single-line hunk); a hunk with `d == 0` is a pure deletion, so it's skipped.
fn parse_hunk_header(line: &str) -> Option<RangeInclusive<usize>> {
    let rest = line.strip_prefix("@@ ")?;
    let plus = rest.split(' ').find(|p| p.starts_with('+'))?;
    let (start, count) = match plus[1..].split_once(',') {
        Some((s, c)) => (s.parse().ok()?, c.parse().ok()?),
        None => (plus[1..].parse().ok()?, 1),
    };
    if count == 0 { None } else { Some(start..=(start + count - 1)) }
}

#[cfg(test)]
mod tests {
    use super::parse_hunk_header;

    #[test]
    fn multi_line_hunk() {
        assert_eq!(parse_hunk_header("@@ -10,2 +10,5 @@ fn foo() {"), Some(10..=14));
    }

    #[test]
    fn single_line_hunk_has_no_count() {
        assert_eq!(parse_hunk_header("@@ -3 +3 @@"), Some(3..=3));
    }

    #[test]
    fn pure_deletion_is_skipped() {
        assert_eq!(parse_hunk_header("@@ -5,3 +5,0 @@"), None);
    }

    #[test]
    fn non_hunk_line_is_ignored() {
        assert_eq!(parse_hunk_header("diff --git a/x b/x"), None);
    }
}
