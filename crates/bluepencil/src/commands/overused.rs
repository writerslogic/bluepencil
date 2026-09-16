use anyhow::Result;
use bluepencil_core::analysis::overuse::overused;

use crate::cli::OverusedArgs;
use crate::context::Context;
use crate::output::json;
use crate::output::table::{Align, Table};

pub fn run(ctx: &Context, args: &OverusedArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let rows = overused(&docs, &ctx.lexicons, args.min_ratio, args.min_count);

    if ctx.json {
        return json::print(&serde_json::json!({ "words": rows.iter().take(ctx.limit).collect::<Vec<_>>() }));
    }

    if rows.is_empty() {
        if ctx.lexicons.english_frequency.is_empty() {
            println!("No baseline frequency data bundled; `overused` has nothing to compare against.");
        } else {
            println!("Nothing found.");
        }
        return Ok(());
    }

    let mut t = Table::new(&[
        ("word", Align::Left),
        ("count", Align::Right),
        ("doc/1M", Align::Right),
        ("baseline/1M", Align::Right),
        ("ratio", Align::Right),
    ]);
    for r in rows.iter().take(ctx.limit) {
        t.row(vec![
            r.word.clone(),
            r.count.to_string(),
            format!("{:.1}", r.document_per_million),
            format!("{:.1}", r.baseline_per_million),
            format!("{:.1}x", r.ratio),
        ]);
    }
    t.print();
    println!("\n{} words found", rows.len());
    Ok(())
}
