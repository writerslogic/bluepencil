use anyhow::Result;
use bluepencil_core::Span;
use bluepencil_core::analysis::echoes::echoes;

use crate::cli::EchoesArgs;
use crate::context::Context;
use crate::output::human::{file_banner, ranked};
use crate::output::{excerpt, json, location};

pub fn run(ctx: &Context, args: &EchoesArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let cfg = &ctx.config.echoes;
    let window = args.window.unwrap_or(cfg.window);
    let min_length = args.min_length.unwrap_or(cfg.min_length);
    let ignore = ctx.echo_ignore();
    let names = args.names || cfg.include_names;
    let results: Vec<_> = docs.iter().map(|d| echoes(d, &ctx.lexicons, window, min_length, &ignore, names)).collect();

    if ctx.json {
        let out: Vec<_> = docs
            .iter()
            .zip(&results)
            .map(|(d, es)| {
                let items: Vec<_> = es
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "word": e.word,
                            "first": location(d, e.first),
                            "second": location(d, e.second),
                            "distance": e.distance,
                        })
                    })
                    .collect();
                serde_json::json!({ "file": d.name, "echoes": items })
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
        }
    }
    println!("\n{total} echoes within {window} words");
    Ok(())
}
