use anyhow::Result;
use bluepencil_core::analysis::arc::{SectionArc, arc};
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::Input;
use crate::context::Context;
use crate::output::chart::bar;
use crate::output::human::{file_banner, thousands};
use crate::output::json;
use crate::output::table::{Align, Table};

#[derive(Serialize, JsonSchema)]
pub struct ArcOutput {
    pub file: String,
    pub sections: Vec<SectionArc>,
}

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    let per_doc: Vec<Vec<SectionArc>> = docs.iter().map(|d| arc(d, &ctx.lexicons)).collect();

    if ctx.json {
        let out: Vec<_> =
            docs.iter().zip(per_doc).map(|(d, sections)| ArcOutput { file: d.name.clone(), sections }).collect();
        return json::print(&out);
    }

    let max_words = per_doc.iter().flatten().map(|s| s.words).max().unwrap_or(1).max(1);
    for (doc, sections) in docs.iter().zip(&per_doc) {
        file_banner(&docs, doc);
        let mut t = Table::new(&[
            ("section", Align::Left),
            ("words", Align::Right),
            ("dialogue", Align::Right),
            ("sent.len", Align::Right),
            ("adverbs/1k", Align::Right),
            ("passive/1k", Align::Right),
            ("", Align::Left),
        ]);
        for s in sections {
            t.row(vec![
                s.section.chars().take(40).collect(),
                thousands(s.words),
                format!("{:.0}%", s.dialogue_ratio * 100.0),
                format!("{:.1}", s.sentence_words_mean),
                format!("{:.1}", s.adverbs_per_1k),
                format!("{:.1}", s.passive_per_1k),
                bar(s.words, max_words, 20),
            ]);
        }
        t.print();
        println!();
    }
    Ok(())
}
