use std::ops::RangeInclusive;
use std::path::Path;

use anyhow::{Context as _, Result, bail};

/// Line ranges (1-based, inclusive) added or modified in `name` since `since`, read from
/// `git diff --unified=0`'s hunk headers. A hunk with zero added lines (a pure deletion)
/// contributes nothing, since there's nothing left in the current file to check.
pub fn changed_lines(since: &str, name: &str) -> Result<Vec<RangeInclusive<usize>>> {
    let git_path = repo_relative_path(name)?;
    // `git diff`'s pathspec is resolved relative to the current working directory, not the
    // repo root, so a repo-root-relative path from `repo_relative_path` needs the `:/` magic
    // to anchor it back to the top regardless of where this process was run from.
    let output = std::process::Command::new("git")
        .args(["diff", "--unified=0", "--no-color", since, "--", &format!(":/{git_path}")])
        .output()?;
    if !output.status.success() {
        bail!("git diff failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.lines().filter_map(parse_hunk_header).collect())
}

/// Turns a document name (relative to the current directory, or absolute when
/// `Context::documents` expanded `project.files` against the config's directory instead) into
/// a path relative to the repo root, in git's own `/` style.
///
/// This asks git itself, via the name's *parent* directory, rather than computing it with
/// `std::env::current_dir` / `strip_prefix`: on Windows those filesystem APIs can normalize a
/// location differently than `git rev-parse` does (verbatim `\\?\` prefixes, short vs. long
/// names), and a name containing `..` (e.g. `../../README.md` from a subdirectory) isn't
/// normalized the way a shell would. `git -C <parent>` resolves both `..` and the platform's
/// own path quirks the same way `git show <rev>:<path>` expects.
///
/// The returned path is repo-root-relative, which is exactly what `git show` wants but NOT
/// what `git diff`'s pathspec wants: `git diff`'s pathspec is resolved relative to the
/// current working directory, not the repo root, so a caller passing this to `git diff` must
/// re-anchor it to the top with `:/` (see `changed_lines`) or it silently matches nothing,
/// and no matches, from a subdirectory other than the repo root.
pub fn repo_relative_path(name: &str) -> Result<String> {
    if name == "<stdin>" {
        bail!("comparing against git history needs a real file, not stdin");
    }
    let path = Path::new(name);
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let file_name =
        path.file_name().with_context(|| format!("{name} has no file name"))?.to_string_lossy().replace('\\', "/");
    let parent_prefix = git_output_in(parent, &["rev-parse", "--show-prefix"])
        .with_context(|| format!("{name} is outside the git repository"))?;
    Ok(format!("{}{file_name}", parent_prefix.trim()))
}

fn git_output_in(dir: &Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git").arg("-C").arg(dir).args(args).output().context("running git")?;
    if !output.status.success() {
        bail!("git -C {} {}: {}", dir.display(), args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
