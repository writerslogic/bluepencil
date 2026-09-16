use anyhow::Result;
use bluepencil_core::Document;
use bluepencil_core::analysis::frequency::frequency;

use crate::cli::FreqArgs;
use crate::context::Context;
use crate::output::{human, json};

pub fn run(ctx: &Context, args: &FreqArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let rows = frequency(&docs, &ctx.lexicons, args.all, args.min_length);
    let total: usize = docs.iter().map(Document::word_count).sum();
    if ctx.json {
        let items: Vec<_> =
            rows.iter().take(ctx.limit).map(|(w, n)| serde_json::json!({ "word": w, "count": n })).collect();
        return json::print(&serde_json::json!({ "total_words": total, "words": items }));
    }
    human::ranked(&rows, "word", ctx.limit, Some(total));
    Ok(())
}
