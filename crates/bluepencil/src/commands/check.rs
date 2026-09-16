use anyhow::Result;
use serde::Serialize;

use super::report::{Metrics, measure};
use crate::cli::Input;
use crate::config::{Check, Severity};
use crate::context::Context;
use crate::exit::Status;
use crate::output::json;

#[derive(Serialize)]
struct Violation {
    file: String,
    rule: &'static str,
    limit: f64,
    actual: f64,
    severity: Severity,
}

pub fn run(ctx: &Context, input: &Input) -> Result<Status> {
    let docs = ctx.documents(input)?;
    let limits = &ctx.config.check;
    let mut violations = Vec::new();
    for doc in &docs {
        let m = measure(ctx, std::slice::from_ref(doc), &doc.name);
        evaluate(limits, &m, &mut violations);
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
