use anyhow::Result;
use serde::Serialize;

use super::report::{Metrics, measure};
use crate::cli::Input;
use crate::config::Check;
use crate::context::Context;
use crate::exit::Status;
use crate::output::json;

#[derive(Serialize)]
struct Violation {
    file: String,
    rule: &'static str,
    limit: f64,
    actual: f64,
}

pub fn run(ctx: &Context, input: &Input) -> Result<Status> {
    let docs = ctx.documents(input)?;
    let limits = &ctx.config.check;
    let mut violations = Vec::new();
    for doc in &docs {
        let m = measure(ctx, std::slice::from_ref(doc), &doc.name);
        evaluate(limits, &m, &mut violations);
    }

    if ctx.json {
        json::print(&serde_json::json!({ "passed": violations.is_empty(), "violations": violations }))?;
    } else if violations.is_empty() {
        println!("All checks passed ({} files).", docs.len());
    } else {
        for v in &violations {
            println!("{}: {} is {} (limit {})", v.file, v.rule, trim(v.actual), trim(v.limit));
        }
        println!("\n{} check(s) failed.", violations.len());
    }
    Ok(if violations.is_empty() { Status::Ok } else { Status::Failed })
}

fn trim(v: f64) -> String {
    if v.fract() == 0.0 { format!("{v:.0}") } else { format!("{v:.2}") }
}

fn evaluate(c: &Check, m: &Metrics, out: &mut Vec<Violation>) {
    let mut max = |rule: &'static str, limit: Option<f64>, actual: f64| {
        if let Some(limit) = limit
            && actual > limit
        {
            out.push(Violation { file: m.name.clone(), rule, limit, actual });
        }
    };
    max("echoes per 1k words", c.max_echoes_per_1k, m.per_1k(m.echoes));
    max("adverbs per 1k words", c.max_adverbs_per_1k, m.per_1k(m.adverbs));
    max("filter words per 1k words", c.max_filter_per_1k, m.per_1k(m.filter));
    max("hedges per 1k words", c.max_hedges_per_1k, m.per_1k(m.hedges));
    max("tics per 1k words", c.max_tics_per_1k, m.per_1k(m.tics));
    max("cliches", c.max_cliches.map(|v| v as f64), m.cliches as f64);
    max("repeated opener runs", c.max_repeated_starter_runs.map(|v| v as f64), m.repeated_starter_runs as f64);
    max("monotonous runs", c.max_monotonous_runs.map(|v| v as f64), m.monotonous_runs as f64);
    max("longest sentence (words)", c.max_sentence_words.map(|v| v as f64), m.sentence_lengths.max as f64);
    max("Flesch-Kincaid grade", c.max_grade, m.readability.flesch_kincaid_grade);
    max("dialogue ratio", c.max_dialogue_ratio, m.dialogue.ratio);

    let mut min = |rule: &'static str, limit: Option<f64>, actual: f64| {
        if let Some(limit) = limit
            && actual < limit
        {
            out.push(Violation { file: m.name.clone(), rule, limit, actual });
        }
    };
    min("MATTR", c.min_mattr, m.diversity.mattr);
    min("dialogue ratio", c.min_dialogue_ratio, m.dialogue.ratio);
}
