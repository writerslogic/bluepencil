use anyhow::{Context as _, Result};
use bluepencil_core::Document;

use super::report::measure;
use crate::context::Context;
use crate::gitdiff::{document_at, git_output, repo_relative_path, resolve_ref};

pub struct TrendPoint {
    pub label: String,
    pub words: usize,
    pub echoes_per_1k: f64,
    pub adverbs_per_1k: f64,
    pub cliches_per_1k: f64,
}

/// Samples word count and a few style-flag rates across git history, so `report --html` can
/// chart drift over time rather than a single snapshot. `docs` is the file set as resolved at
/// HEAD; each sampled commit is measured against whichever of those paths existed there,
/// following a rename via `document_at` so a renamed file doesn't show as a gap.
pub fn collect(ctx: &Context, docs: &[Document], since: &str, points: usize) -> Result<Vec<TrendPoint>> {
    git_output(&["rev-parse", "--git-dir"]).context("--trend-since requires running inside a git repository")?;
    let since = resolve_ref(since)?;
    let rels = docs.iter().map(|d| repo_relative_path(&d.name)).collect::<Result<Vec<_>>>()?;
    let commits = sampled_commits(&since, points.max(2))?;

    commits
        .iter()
        .map(|rev| {
            let sampled =
                rels.iter().filter_map(|rel| document_at(rev, "HEAD", rel).transpose()).collect::<Result<Vec<_>>>()?;
            let m = measure(ctx, &sampled, "trend");
            Ok(TrendPoint {
                label: commit_date(rev)?,
                words: m.counts.words,
                echoes_per_1k: m.per_1k(m.echoes),
                adverbs_per_1k: m.per_1k(m.adverbs),
                cliches_per_1k: m.per_1k(m.cliches),
            })
        })
        .collect()
}

fn commit_date(rev: &str) -> Result<String> {
    Ok(git_output(&["show", "-s", "--format=%cs", rev])?.trim().to_string())
}

/// `points` commits evenly spaced between `since` and `HEAD` inclusive. Returns every commit
/// in the range as-is when there are fewer than `points` of them, rather than padding with
/// duplicates.
fn sampled_commits(since: &str, points: usize) -> Result<Vec<String>> {
    let mut commits = vec![since.to_string()];
    commits.extend(git_output(&["rev-list", "--reverse", &format!("{since}..HEAD")])?.lines().map(str::to_string));
    if commits.len() <= points {
        return Ok(commits);
    }
    let last = commits.len() - 1;
    Ok((0..points).map(|i| commits[i * last / (points - 1)].clone()).collect())
}
