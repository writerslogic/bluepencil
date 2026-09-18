use std::ops::RangeInclusive;
use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, Result, bail};
use bluepencil_core::{Document, Format};

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

pub fn git_output(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output().context("running git")?;
    if !output.status.success() {
        bail!("git {}: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Resolves a commit, tag, branch, or `YYYY-MM-DD` date to the sha of the commit it names
/// (for a date, the most recent commit at or before it).
pub fn resolve_ref(spec: &str) -> Result<String> {
    if let Ok(sha) = git_output(&["rev-parse", "--verify", &format!("{spec}^{{commit}}")]) {
        return Ok(sha.trim().to_string());
    }
    // `git rev-list --before` treats an unparseable date as "no limit" and silently returns
    // the most recent commit instead of erroring, so a ref typo must be caught before it gets here.
    if !looks_like_date(spec) {
        bail!("`{spec}` is not a git ref bluepencil can resolve (dates must look like YYYY-MM-DD)");
    }
    let sha = git_output(&["rev-list", "-1", "--before", spec, "HEAD"])?;
    let sha = sha.trim();
    if sha.is_empty() {
        bail!("no commit before `{spec}`");
    }
    Ok(sha.to_string())
}

fn looks_like_date(spec: &str) -> bool {
    let date = spec.split(['T', ' ']).next().unwrap_or(spec);
    let mut parts = date.split('-');
    let digits = |s: &str, len: usize| s.len() == len && s.chars().all(|c| c.is_ascii_digit());
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(y), Some(m), Some(d), None) if digits(y, 4) && digits(m, 2) && digits(d, 2)
    )
}

/// The file's text at `rev`, or `None` when it genuinely doesn't exist there; any other git
/// failure is propagated rather than silently folded into "doesn't exist".
pub fn git_show(rev: &str, git_path: &str) -> Result<Option<String>> {
    let spec = format!("{rev}:{git_path}");
    let output = Command::new("git").args(["show", &spec]).output().context("running git show")?;
    if output.status.success() {
        return Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()));
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("does not exist in") || stderr.contains("exists on disk, but not in") {
        return Ok(None);
    }
    bail!("git show {spec}: {}", stderr.trim());
}

/// The name `current_path` had at `rev`, if git's own rename detection pairs it with a
/// differently-named file by `later_rev`. `git diff`'s pathspec restricts rename detection to
/// the intersection of what's present on both sides, so this deliberately runs unrestricted
/// and filters the output instead of passing `current_path` as a pathspec.
pub fn renamed_from(rev: &str, later_rev: &str, current_path: &str) -> Result<Option<String>> {
    let output =
        Command::new("git").args(["diff", "-M", "--name-status", rev, later_rev]).output().context("running git diff")?;
    if !output.status.success() {
        bail!("git diff -M --name-status {rev} {later_rev}: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let mut parts = line.split('\t');
        let Some(status) = parts.next() else { continue };
        if !status.starts_with('R') {
            continue;
        }
        if let (Some(from), Some(to)) = (parts.next(), parts.next())
            && to == current_path
        {
            return Ok(Some(from.to_string()));
        }
    }
    Ok(None)
}

/// Word count of `current_path` as of `rev`, following a rename to `later_rev`'s name if git
/// pairs one, rather than reporting 0 for a file that only looks new under its current name.
pub fn word_count_at(rev: &str, later_rev: &str, current_path: &str) -> Result<usize> {
    if let Some(text) = git_show(rev, current_path)? {
        return Ok(count_words(current_path, text));
    }
    if let Some(old_path) = renamed_from(rev, later_rev, current_path)?
        && let Some(text) = git_show(rev, &old_path)?
    {
        return Ok(count_words(&old_path, text));
    }
    Ok(0)
}

fn count_words(git_path: &str, text: String) -> usize {
    let format = Format::from_path(Path::new(git_path));
    Document::parse(git_path.to_string(), text, format).word_count()
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
    use super::{looks_like_date, parse_hunk_header};

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
    fn iso_date_is_a_date() {
        assert!(looks_like_date("2026-09-01"));
    }

    #[test]
    fn iso_datetime_is_a_date() {
        assert!(looks_like_date("2026-09-01T12:00:00"));
    }

    #[test]
    fn git_ref_is_not_a_date() {
        assert!(!looks_like_date("HEAD~5"));
    }

    #[test]
    fn typo_is_not_a_date() {
        assert!(!looks_like_date("not-a-real-ref"));
    }

    #[test]
    fn branch_name_with_dashes_is_not_a_date() {
        assert!(!looks_like_date("release-2026-09"));
    }

    #[test]
    fn non_hunk_line_is_ignored() {
        assert_eq!(parse_hunk_header("diff --git a/x b/x"), None);
    }
}
