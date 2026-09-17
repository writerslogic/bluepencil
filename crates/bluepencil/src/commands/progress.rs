use std::path::{Path, PathBuf};
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
    let repo_root = repo_root()?;
    let since = resolve_ref(&args.since)?;
    let until = args.until.as_deref().map(resolve_ref).transpose()?;

    let docs = ctx.documents(&args.input)?;
    let mut deltas = Vec::with_capacity(docs.len());
    for doc in &docs {
        let rel = repo_relative_path(&repo_root, &doc.name)?;
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

fn repo_relative_path(repo_root: &Path, name: &str) -> Result<PathBuf> {
    if name == "<stdin>" {
        bail!("progress needs real files to compare against git history, not stdin");
    }
    let path = Path::new(name);
    let absolute = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir()?.join(path) };
    absolute
        .strip_prefix(repo_root)
        .map(Path::to_path_buf)
        .with_context(|| format!("{name} is outside the git repository at {}", repo_root.display()))
}

fn word_count_at(rev: &str, path: &Path) -> Result<usize> {
    let spec = format!("{rev}:{}", path.display());
    let output = Command::new("git").args(["show", &spec]).output().context("running git show")?;
    if !output.status.success() {
        return Ok(0);
    }
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let format = Format::from_path(path);
    Ok(Document::parse(path.display().to_string(), text, format).word_count())
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

fn repo_root() -> Result<PathBuf> {
    let text =
        git_output(&["rev-parse", "--show-toplevel"]).context("progress requires running inside a git repository")?;
    Ok(PathBuf::from(text.trim()))
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
