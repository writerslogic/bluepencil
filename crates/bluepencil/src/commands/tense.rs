use anyhow::Result;
use bluepencil_core::analysis::tense::{Tense, tense};
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::TenseArgs;
use crate::context::Context;
use crate::output::human::{file_banner, findings};
use crate::output::{Located, json, locate};

#[derive(Serialize, JsonSchema)]
pub struct TenseOutput {
    pub file: String,
    pub dominant: Tense,
    pub past_sentences: usize,
    pub present_sentences: usize,
    pub findings: Vec<Located>,
}

pub fn run(ctx: &Context, args: &TenseArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let results: Vec<_> = docs.iter().map(|d| tense(d, args.run)).collect();

    if ctx.json {
        let out: Vec<_> = docs
            .iter()
            .zip(&results)
            .map(|(d, r)| TenseOutput {
                file: d.name.clone(),
                dominant: r.dominant,
                past_sentences: r.past_sentences,
                present_sentences: r.present_sentences,
                findings: r.findings.iter().map(|f| locate(d, f)).collect(),
            })
            .collect();
        return json::print(&out);
    }

    for (doc, r) in docs.iter().zip(&results) {
        file_banner(&docs, doc);
        println!("dominant tense: {:?} ({} past, {} present)", r.dominant, r.past_sentences, r.present_sentences);
        let items: Vec<_> = r.findings.iter().map(|f| (doc, f)).collect();
        findings(&items);
        println!();
    }
    Ok(())
}
