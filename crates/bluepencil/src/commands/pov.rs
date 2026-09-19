use anyhow::Result;
use bluepencil_core::analysis::pov::{Pov, pov};
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::PovArgs;
use crate::context::Context;
use crate::output::human::{file_banner, findings};
use crate::output::{Located, json, locate};

#[derive(Serialize, JsonSchema)]
pub struct PovOutput {
    pub file: String,
    pub dominant: Pov,
    pub first_sentences: usize,
    pub second_sentences: usize,
    pub third_sentences: usize,
    pub findings: Vec<Located>,
}

pub fn run(ctx: &Context, args: &PovArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let results: Vec<_> = docs.iter().map(|d| pov(d, args.run)).collect();

    if ctx.json {
        let out: Vec<_> = docs
            .iter()
            .zip(&results)
            .map(|(d, r)| PovOutput {
                file: d.name.clone(),
                dominant: r.dominant,
                first_sentences: r.first_sentences,
                second_sentences: r.second_sentences,
                third_sentences: r.third_sentences,
                findings: r.findings.iter().map(|f| locate(d, f)).collect(),
            })
            .collect();
        return json::print(&out);
    }

    for (doc, r) in docs.iter().zip(&results) {
        file_banner(&docs, doc);
        println!(
            "dominant POV: {:?} ({} first, {} second, {} third person)",
            r.dominant, r.first_sentences, r.second_sentences, r.third_sentences
        );
        let items: Vec<_> = r.findings.iter().map(|f| (doc, f)).collect();
        findings(&items);
        println!();
    }
    Ok(())
}
