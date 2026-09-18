use anyhow::{Context as _, Result};

use crate::cli::{Input, ProgressArgs};
use crate::context::Context;
use crate::gitdiff::{git_output, repo_relative_path, resolve_ref, word_count_at};
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
                let before = word_count_at(&since, before_boundary, &rel)?;
                let after = match &until {
                    Some(rev) => word_count_at(rev, "HEAD", &rel)?,
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
            let before = word_count_at(since, until, &rel)?;
            let after = word_count_at(until, "HEAD", &rel)?;
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

