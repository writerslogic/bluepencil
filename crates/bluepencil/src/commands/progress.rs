use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, Result, bail};
use bluepencil_core::{Document, Format};

use crate::cli::{Input, ProgressArgs};
use crate::context::Context;
use crate::gitdiff::repo_relative_path;
use crate::output::human::thousands;
use crate::output::json;
use crate::output::table::{Align, Table};

struct Delta {
    path: String,
    before: usize,
    after: usize,
}

impl Delta {
    fn change(&self) -> i64 {
        self.after as i64 - self.before as i64
    }
}

pub fn run(ctx: &Context, args: &ProgressArgs) -> Result<()> {
    git_output(&["rev-parse", "--git-dir"]).context("progress requires running inside a git repository")?;
    let since = resolve_ref(&args.since)?;
    let until = args.until.as_deref().map(resolve_ref).transpose()?;

    let deltas = match ctx.documents(&args.input) {
        Ok(docs) => docs
            .iter()
            .map(|doc| {
                let rel = repo_relative_path(&doc.name)?;
                let before_boundary = until.as_deref().unwrap_or("HEAD");
                let before = resolve_word_count(&since, before_boundary, &rel)?;
                let after = match &until {
                    Some(rev) => resolve_word_count(rev, "HEAD", &rel)?,
                    None => doc.word_count(),
                };
                Ok(Delta { path: doc.name.clone(), before, after })
            })
            .collect::<Result<Vec<_>>>()?,
        // The file set is normally read off disk, which can't see a path that existed only
        // between `since` and `until` and was deleted before now. When every requested path
        // is a literal name (no glob, no stdin) and `--until` bounds the comparison to two
        // historical points, fall back to resolving both endpoints from git alone instead of
        // surfacing the disk error; any other failure (globs, stdin, no `--until`) still does.
        Err(disk_err) => historical_deltas(&args.input, &since, until.as_deref(), disk_err)?,
    };

    let total_before: usize = deltas.iter().map(|d| d.before).sum();
    let total_after: usize = deltas.iter().map(|d| d.after).sum();

    if ctx.json {
        let files: Vec<_> = deltas
            .iter()
            .map(|d| {
                serde_json::json!({
                    "file": d.path,
                    "before": d.before,
                    "after": d.after,
                    "change": d.change(),
                })
            })
            .collect();
        return json::print(&serde_json::json!({
            "since": since,
            "until": until,
            "files": files,
            "total_before": total_before,
            "total_after": total_after,
            "total_change": total_after as i64 - total_before as i64,
        }));
    }

    let mut t = Table::new(&[
        ("file", Align::Left),
        ("before", Align::Right),
        ("after", Align::Right),
        ("change", Align::Right),
    ]);
    for d in &deltas {
        t.row(vec![d.path.clone(), thousands(d.before), thousands(d.after), signed(d.change())]);
    }
    if deltas.len() > 1 {
        t.row(vec![
            "total".to_string(),
            thousands(total_before),
            thousands(total_after),
            signed(total_after as i64 - total_before as i64),
        ]);
    }
    t.print();
    Ok(())
}

/// Resolves each requested path purely from git history, for a request `Context::documents`
/// couldn't satisfy off disk. Declines (returning the original disk error) unless every path
/// is a literal name and `--until` gives a second historical boundary to resolve against,
/// since a glob pattern or stdin has no meaning against a tree that no longer has the file.
fn historical_deltas(input: &Input, since: &str, until: Option<&str>, disk_err: anyhow::Error) -> Result<Vec<Delta>> {
    let Some(until) = until else { return Err(disk_err) };
    if input.files.is_empty() || input.files.iter().any(|f| f == "-" || has_glob_chars(f)) {
        return Err(disk_err);
    }
    input
        .files
        .iter()
        .map(|name| {
            let rel = repo_relative_path(name)?;
            let before = resolve_word_count(since, until, &rel)?;
            let after = resolve_word_count(until, "HEAD", &rel)?;
            Ok(Delta { path: name.clone(), before, after })
        })
        .collect()
}

fn has_glob_chars(s: &str) -> bool {
    s.contains(['*', '?', '['])
}

fn signed(n: i64) -> String {
    if n >= 0 { format!("+{n}") } else { n.to_string() }
}

/// Word count of `current_path` as of `rev`, falling back to the name it had at `rev` if it
/// was later renamed to `current_path` by the time of `later_rev`. Without this, a file
/// renamed between `rev` and `later_rev` reports a `before` of 0 (git show finds nothing
/// under the new name at the old revision) and so a total word count as if newly written,
/// even though the content carried over.
fn resolve_word_count(rev: &str, later_rev: &str, current_path: &str) -> Result<usize> {
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

/// `None` when the file genuinely doesn't exist at `rev` (e.g. not yet written); propagates
/// any other git failure instead of silently treating it the same way.
fn git_show(rev: &str, git_path: &str) -> Result<Option<String>> {
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
fn renamed_from(rev: &str, later_rev: &str, current_path: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["diff", "-M", "--name-status", rev, later_rev])
        .output()
        .context("running git diff")?;
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

fn resolve_ref(spec: &str) -> Result<String> {
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

fn git_output(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output().context("running git")?;
    if !output.status.success() {
        bail!("git {}: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::looks_like_date;

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
}
