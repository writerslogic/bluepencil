use anyhow::Result;
use bluepencil_core::analysis::repetition::repeats;

use crate::cli::RepeatsArgs;
use crate::context::Context;
use crate::output::human::file_banner;
use crate::output::table::{Align, Table};
use crate::output::{json, location};

pub fn run(ctx: &Context, args: &RepeatsArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let cfg = &ctx.config.repeats;
    let min = args.min.unwrap_or(cfg.min).max(1);
    let max = args.max.unwrap_or(cfg.max).max(min);
    let count = args.count.unwrap_or(cfg.count).max(2);

    if ctx.json {
        let out: Vec<_> = docs
            .iter()
            .map(|d| {
                let reps: Vec<_> = repeats(d, &ctx.lexicons, min, max, count)
                    .into_iter()
                    .map(|r| {
                        let locs: Vec<_> = r.spans.iter().map(|s| location(d, *s)).collect();
                        serde_json::json!({ "phrase": r.phrase, "words": r.words, "count": r.count, "locations": locs })
                    })
                    .collect();
                serde_json::json!({ "file": d.name, "repeats": reps })
            })
            .collect();
        return json::print(&out);
    }

    for doc in &docs {
        file_banner(&docs, doc);
        let reps = repeats(doc, &ctx.lexicons, min, max, count);
        if reps.is_empty() {
            println!("No repeated phrases of {min} to {max} words.");
            continue;
        }
        if args.locations {
            for r in reps.iter().take(ctx.limit) {
                println!("{}  ({}x)", r.phrase, r.count);
                for s in &r.spans {
                    println!("    {}", location(doc, *s));
                }
            }
        } else {
            let mut t = Table::new(&[("count", Align::Right), ("phrase", Align::Left), ("first seen", Align::Left)]);
            for r in reps.iter().take(ctx.limit) {
                t.row(vec![r.count.to_string(), r.phrase.clone(), location(doc, r.spans[0])]);
            }
            t.print();
        }
        if reps.len() > ctx.limit {
            println!("… {} more (use -n to show more)", reps.len() - ctx.limit);
        }
    }
    Ok(())
}
