use anyhow::Result;
use bluepencil_core::analysis::dialogue_tags::attributions;
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::Input;
use crate::context::Context;
use crate::output::json;
use crate::output::table::{Align, Table};

#[derive(Clone, Serialize, JsonSchema)]
pub struct SpeakerTally {
    pub speaker: String,
    pub lines: usize,
    pub words: usize,
}

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    let mut tallies = std::collections::HashMap::<String, (usize, usize)>::new();

    for doc in &docs {
        for a in attributions(doc, &ctx.lexicons) {
            let speaker = a.speaker.unwrap_or_else(|| "(unattributed)".to_string());
            let words = doc.text(a.dialogue).split_whitespace().count();
            let entry = tallies.entry(speaker).or_default();
            entry.0 += 1;
            entry.1 += words;
        }
    }

    let mut rows: Vec<SpeakerTally> =
        tallies.into_iter().map(|(speaker, (lines, words))| SpeakerTally { speaker, lines, words }).collect();
    rows.sort_by(|a, b| b.words.cmp(&a.words).then_with(|| a.speaker.cmp(&b.speaker)));

    if ctx.json {
        return json::print(&rows.iter().take(ctx.limit).cloned().collect::<Vec<_>>());
    }

    if rows.is_empty() {
        println!("No dialogue found.");
        return Ok(());
    }

    let mut t = Table::new(&[("speaker", Align::Left), ("lines", Align::Right), ("words", Align::Right)]);
    for r in rows.iter().take(ctx.limit) {
        t.row(vec![r.speaker.clone(), r.lines.to_string(), r.words.to_string()]);
    }
    t.print();
    println!("\n{} speaker(s) found", rows.len());
    Ok(())
}
