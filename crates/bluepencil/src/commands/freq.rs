use anyhow::Result;
use bluepencil_core::Document;
use bluepencil_core::analysis::frequency::frequency;
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::FreqArgs;
use crate::context::Context;
use crate::output::{human, json};

#[derive(Serialize, JsonSchema)]
pub struct FreqWord {
    pub word: String,
    pub count: usize,
}

#[derive(Serialize, JsonSchema)]
pub struct FreqOutput {
    pub total_words: usize,
    pub words: Vec<FreqWord>,
}

pub fn run(ctx: &Context, args: &FreqArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let rows = frequency(&docs, &ctx.lexicons, args.all, args.min_length);
    let total: usize = docs.iter().map(Document::word_count).sum();
    if ctx.json {
        let words: Vec<_> = rows.iter().take(ctx.limit).map(|(w, n)| FreqWord { word: w.clone(), count: *n }).collect();
        return json::print(&FreqOutput { total_words: total, words });
    }
    human::ranked(&rows, "word", ctx.limit, Some(total));
    Ok(())
}
