use std::ops::RangeInclusive;

use anyhow::Result;
use bluepencil_core::analysis::{adverbs, cliches, echoes, filter_words, hedges, passive, rhythm, starters, tics};
use bluepencil_core::{Document, Finding};
use serde::Serialize;

use super::report::{Metrics, measure};
use crate::cli::CheckArgs;
use crate::config::{Check, Severity};
use crate::context::Context;
use crate::exit::Status;
use crate::gitdiff;
use crate::output::json;

/// Rules that only make sense over the whole document -- there's no way to attribute
/// a Flesch-Kincaid grade or a dialogue ratio to a handful of changed lines without
/// distorting what the number means. `--since` skips these rather than silently
/// evaluating them against the whole file under a name that implies otherwise.
const NOT_LINE_SCOPED: &[&str] = &["overused", "grade", "mattr", "dialogue_ratio"];

#[derive(Serialize)]
struct Violation {
    file: String,
    rule: &'static str,
    limit: f64,
    actual: f64,
    severity: Severity,
}

pub fn run(ctx: &Context, args: &CheckArgs) -> Result<Status> {
    let docs = ctx.documents(&args.input)?;
    let limits = &ctx.config.check;
    let mut violations = Vec::new();

    for doc in &docs {
        match &args.since {
            Some(since) => {
                let ranges = gitdiff::changed_lines(since, std::path::Path::new(&doc.name))?;
                if !ranges.is_empty() {
                    evaluate_scoped(ctx, doc, &ranges, &mut violations);
                }
            }
            None => {
                let m = measure(ctx, std::slice::from_ref(doc), &doc.name);
                evaluate(limits, &m, &mut violations);
            }
        }
    }
    let failed = violations.iter().any(|v| v.severity == Severity::Error);

    if ctx.json {
        json::print(&serde_json::json!({ "passed": !failed, "violations": violations }))?;
    } else if violations.is_empty() {
        println!("All checks passed ({} files).", docs.len());
    } else {
        for v in &violations {
            let tag = match v.severity {
                Severity::Error => "error",
                Severity::Warn => "warn",
            };
            println!("{}: [{tag}] {} is {} (limit {})", v.file, v.rule, trim(v.actual), trim(v.limit));
        }
        let (errors, warnings) = (
            violations.iter().filter(|v| v.severity == Severity::Error).count(),
            violations.iter().filter(|v| v.severity == Severity::Warn).count(),
        );
        println!("\n{errors} error(s), {warnings} warning(s).");
    }
    if args.since.is_some() && !ctx.json {
        println!("(--since: skipped whole-document rules: {})", NOT_LINE_SCOPED.join(", "));
    }
    Ok(if failed { Status::Failed } else { Status::Ok })
}

fn trim(v: f64) -> String {
    if v.fract() == 0.0 { format!("{v:.0}") } else { format!("{v:.2}") }
}

fn evaluate(c: &Check, m: &Metrics, out: &mut Vec<Violation>) {
    let mut max = |key: &str, rule: &'static str, limit: Option<f64>, actual: f64| {
        if let Some(limit) = limit
            && actual > limit
        {
            out.push(Violation { file: m.name.clone(), rule, limit, actual, severity: c.severity_of(key) });
        }
    };
    max("echoes", "echoes per 1k words", c.max_echoes_per_1k, m.per_1k(m.echoes));
    max("adverbs", "adverbs per 1k words", c.max_adverbs_per_1k, m.per_1k(m.adverbs));
    max("filter", "filter words per 1k words", c.max_filter_per_1k, m.per_1k(m.filter));
    max("hedges", "hedges per 1k words", c.max_hedges_per_1k, m.per_1k(m.hedges));
    max("tics", "tics per 1k words", c.max_tics_per_1k, m.per_1k(m.tics));
    max("passive", "passive per 1k words", c.max_passive_per_1k, m.per_1k(m.passive));
    max("overused", "overused words", c.max_overused.map(|v| v as f64), m.overused as f64);
    max("cliches", "cliches", c.max_cliches.map(|v| v as f64), m.cliches as f64);
    max(
        "repeated_starter_runs",
        "repeated opener runs",
        c.max_repeated_starter_runs.map(|v| v as f64),
        m.repeated_starter_runs as f64,
    );
    max("monotonous_runs", "monotonous runs", c.max_monotonous_runs.map(|v| v as f64), m.monotonous_runs as f64);
    max(
        "sentence_words",
        "longest sentence (words)",
        c.max_sentence_words.map(|v| v as f64),
        m.sentence_lengths.max as f64,
    );
    max("grade", "Flesch-Kincaid grade", c.max_grade, m.readability.flesch_kincaid_grade);
    max("dialogue_ratio", "dialogue ratio", c.max_dialogue_ratio, m.dialogue.ratio);

    let mut min = |key: &str, rule: &'static str, limit: Option<f64>, actual: f64| {
        if let Some(limit) = limit
            && actual < limit
        {
            out.push(Violation { file: m.name.clone(), rule, limit, actual, severity: c.severity_of(key) });
        }
    };
    min("mattr", "MATTR", c.min_mattr, m.diversity.mattr);
    min("dialogue_ratio", "dialogue ratio", c.min_dialogue_ratio, m.dialogue.ratio);
}

fn in_ranges(ranges: &[RangeInclusive<usize>], line: usize) -> bool {
    ranges.iter().any(|r| r.contains(&line))
}

fn scoped_count(doc: &Document, ranges: &[RangeInclusive<usize>], findings: &[Finding]) -> usize {
    findings.iter().filter(|f| in_ranges(ranges, doc.position(f.span.start).line)).count()
}

/// Evaluates only the rules whose findings carry a location, counted against the word
/// count of the changed lines rather than the whole file so the rate isn't diluted by
/// everything the diff didn't touch. See `NOT_LINE_SCOPED` for what's excluded.
fn evaluate_scoped(ctx: &Context, doc: &Document, ranges: &[RangeInclusive<usize>], out: &mut Vec<Violation>) {
    let c = &ctx.config.check;
    let lex = &ctx.lexicons;
    let cfg = &ctx.config;
    let ignore = ctx.echo_ignore();

    let total = doc.words().filter(|w| in_ranges(ranges, doc.position(w.span.start).line)).count().max(1);
    let per_1k = |n: usize| n as f64 / total as f64 * 1000.0;

    let mut max = |key: &str, rule: &'static str, limit: Option<f64>, actual: f64| {
        if let Some(limit) = limit
            && actual > limit
        {
            out.push(Violation { file: doc.name.clone(), rule, limit, actual, severity: c.severity_of(key) });
        }
    };

    let echo_findings =
        echoes::echoes(doc, lex, cfg.echoes.window, cfg.echoes.min_length, &ignore, cfg.echoes.include_names);
    let echo_count = echo_findings.iter().filter(|e| in_ranges(ranges, doc.position(e.second.start).line)).count();
    max("echoes", "echoes per 1k words (changed lines)", c.max_echoes_per_1k, per_1k(echo_count));
    max(
        "adverbs",
        "adverbs per 1k words (changed lines)",
        c.max_adverbs_per_1k,
        per_1k(scoped_count(doc, ranges, &adverbs::find(doc, lex))),
    );
    max(
        "filter",
        "filter words per 1k words (changed lines)",
        c.max_filter_per_1k,
        per_1k(scoped_count(doc, ranges, &filter_words::find(doc, lex))),
    );
    max(
        "hedges",
        "hedges per 1k words (changed lines)",
        c.max_hedges_per_1k,
        per_1k(scoped_count(doc, ranges, &hedges::find(doc, lex))),
    );
    max(
        "tics",
        "tics per 1k words (changed lines)",
        c.max_tics_per_1k,
        per_1k(scoped_count(doc, ranges, &tics::find(doc, lex))),
    );
    max(
        "passive",
        "passive per 1k words (changed lines)",
        c.max_passive_per_1k,
        per_1k(scoped_count(doc, ranges, &passive::find(doc, lex))),
    );
    max(
        "cliches",
        "cliches (changed lines)",
        c.max_cliches.map(|v| v as f64),
        scoped_count(doc, ranges, &cliches::find(doc, lex)) as f64,
    );

    let starters = starters::starters(doc, cfg.starters.run);
    max(
        "repeated_starter_runs",
        "repeated opener runs (changed lines)",
        c.max_repeated_starter_runs.map(|v| v as f64),
        scoped_count(doc, ranges, &starters.findings) as f64,
    );

    let rh = rhythm::rhythm(doc, cfg.rhythm.run, cfg.rhythm.tolerance, cfg.rhythm.long);
    let monotony: Vec<Finding> = rh.findings.iter().filter(|f| f.rule == "monotony").cloned().collect();
    max(
        "monotonous_runs",
        "monotonous runs (changed lines)",
        c.max_monotonous_runs.map(|v| v as f64),
        scoped_count(doc, ranges, &monotony) as f64,
    );

    let longest_in_range = doc
        .sentences()
        .filter(|s| in_ranges(ranges, doc.position(s.span.start).line))
        .map(|s| s.words.len())
        .max()
        .unwrap_or(0);
    max(
        "sentence_words",
        "longest sentence in changed lines (words)",
        c.max_sentence_words.map(|v| v as f64),
        longest_in_range as f64,
    );
}
