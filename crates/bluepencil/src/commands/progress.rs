use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, Result, bail};
use bluepencil_core::{Document, Format};

use crate::cli::ProgressArgs;
use crate::context::Context;
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

    let docs = ctx.documents(&args.input)?;
    let mut deltas = Vec::with_capacity(docs.len());
    for doc in &docs {
        let rel = repo_relative_path(&doc.name)?;
        let before = word_count_at(&since, &rel)?;
        let after = match &until {
            Some(rev) => word_count_at(rev, &rel)?,
            None => doc.word_count(),
        };
        deltas.push(Delta { path: doc.name.clone(), before, after });
    }

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

fn signed(n: i64) -> String {
    if n >= 0 { format!("+{n}") } else { n.to_string() }
}

/// Turns a document name (relative to the current directory, or absolute when
/// `Context::documents` expanded `project.files` against the config's directory instead) into
/// a path relative to the repo root, in git's own `/` style.
///
/// This asks git itself, via the name's *parent* directory, rather than computing it with
/// `std::env::current_dir` / `strip_prefix`: on Windows those filesystem APIs can normalize a
/// location differently than `git rev-parse` does (verbatim `\\?\` prefixes, short vs. long
/// names), and a name containing `..` (e.g. `../../README.md` from a subdirectory) isn't
/// normalized by `git show <rev>:<path>` the way a shell would. `git -C <parent>` resolves
/// both `..` and the platform's own path quirks the same way it will when `word_count_at`
/// later runs `git show` from this same process.
fn repo_relative_path(name: &str) -> Result<String> {
    if name == "<stdin>" {
        bail!("progress needs real files to compare against git history, not stdin");
    }
    let path = Path::new(name);
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let file_name =
        path.file_name().with_context(|| format!("{name} has no file name"))?.to_string_lossy().replace('\\', "/");
    let parent_prefix = git_output_in(parent, &["rev-parse", "--show-prefix"])
        .with_context(|| format!("{name} is outside the git repository"))?;
    Ok(format!("{}{file_name}", parent_prefix.trim()))
}

fn word_count_at(rev: &str, git_path: &str) -> Result<usize> {
    let spec = format!("{rev}:{git_path}");
    let output = Command::new("git").args(["show", &spec]).output().context("running git show")?;
    if !output.status.success() {
        return Ok(0);
    }
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let format = Format::from_path(Path::new(git_path));
    Ok(Document::parse(git_path.to_string(), text, format).word_count())
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

fn git_output_in(dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").arg("-C").arg(dir).args(args).output().context("running git")?;
    if !output.status.success() {
        bail!("git -C {} {}: {}", dir.display(), args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
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
