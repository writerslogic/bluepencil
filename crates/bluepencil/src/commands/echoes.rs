use anyhow::Result;
use bluepencil_core::Span;
use bluepencil_core::analysis::echoes::echoes;
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::EchoesArgs;
use crate::context::Context;
use crate::explain::{self, Note, Target};
use crate::output::human::{file_banner, ranked};
use crate::output::{excerpt, json, location};

#[derive(Serialize, JsonSchema)]
pub struct EchoLocated {
    pub word: String,
    pub first: String,
    pub second: String,
    pub distance: usize,
    /// Present only with `--explain`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Note>,
}

#[derive(Serialize, JsonSchema)]
pub struct EchoesFileOutput {
    pub file: String,
    pub echoes: Vec<EchoLocated>,
}

pub fn run(ctx: &Context, args: &EchoesArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let cfg = &ctx.config.echoes;
    let window = args.window.unwrap_or(cfg.window);
    let min_length = args.min_length.unwrap_or(cfg.min_length);
    let ignore = ctx.echo_ignore();
    let names = args.names || cfg.include_names;
    let results: Vec<_> = docs.iter().map(|d| echoes(d, &ctx.lexicons, window, min_length, &ignore, names)).collect();

    // Echoes are explained as the span from the first occurrence to the second, so the model
    // sees both uses; notes line up with `results` flattened in order.
    let messages: Vec<Vec<String>> = results
        .iter()
        .map(|es| es.iter().map(|e| format!("\"{}\" repeated {} words later", e.word, e.distance)).collect())
        .collect();
    let mut notes = if ctx.explain && !args.summary {
        let targets: Vec<_> = docs
            .iter()
            .zip(&results)
            .zip(&messages)
            .flat_map(|((d, es), ms)| {
                es.iter().zip(ms).map(move |(e, m)| Target {
                    doc: d,
                    span: Span::new(e.first.start, e.second.end),
                    rule: "echoes",
                    message: m,
                })
            })
            .collect();
        explain::annotate(ctx, &targets)?.into_iter()
    } else {
        Vec::new().into_iter()
    };

    if ctx.json {
        let out: Vec<_> = docs
            .iter()
            .zip(&results)
            .map(|(d, es)| {
                let echoes: Vec<_> = es
                    .iter()
                    .map(|e| EchoLocated {
                        word: e.word.clone(),
                        first: location(d, e.first),
                        second: location(d, e.second),
                        distance: e.distance,
                        note: notes.next().flatten(),
                    })
                    .collect();
                EchoesFileOutput { file: d.name.clone(), echoes }
            })
            .collect();
        return json::print(&out);
    }

    let mut total = 0;
    for (doc, es) in docs.iter().zip(&results) {
        total += es.len();
        file_banner(&docs, doc);
        if args.summary {
            let mut tally = std::collections::HashMap::<&str, usize>::new();
            for e in es {
                *tally.entry(e.word.as_str()).or_default() += 1;
            }
            let mut rows: Vec<(String, usize)> = tally.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
            rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            ranked(&rows, "word", ctx.limit, None);
            continue;
        }
        for e in es {
            println!("{}  {} ({} words apart)", location(doc, e.second), doc.text(e.second), e.distance);
            println!("    {}", excerpt(doc, Span::new(e.first.start, e.second.end), 100));
            if let Some(note) = notes.next().flatten() {
                explain::print_note(&note);
            }
        }
    }
    println!("\n{total} echoes within {window} words");
    Ok(())
}
