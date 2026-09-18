use anyhow::Result;
use bluepencil_core::analysis::readability::{SectionReadability, document, sections};
use bluepencil_core::stats::readability::Readability;
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::Input;
use crate::context::Context;
use crate::output::human::{file_banner, thousands};
use crate::output::json;
use crate::output::table::{Align, Table};

#[derive(Serialize, JsonSchema)]
pub struct FileReadability {
    pub file: String,
    pub document: Readability,
    pub sections: Vec<SectionReadability>,
}

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    if ctx.json {
        let out: Vec<_> = docs
            .iter()
            .map(|d| FileReadability { file: d.name.clone(), document: document(d), sections: sections(d) })
            .collect();
        return json::print(&out);
    }
    for doc in &docs {
        file_banner(&docs, doc);
        let mut t = Table::new(&[
            ("section", Align::Left),
            ("words", Align::Right),
            ("ease", Align::Right),
            ("grade", Align::Right),
            ("fog", Align::Right),
            ("cli", Align::Right),
            ("ari", Align::Right),
        ]);
        let cells = |title: &str, r: &Readability| {
            vec![
                title.chars().take(40).collect(),
                thousands(r.words),
                format!("{:.1}", r.flesch_reading_ease),
                format!("{:.1}", r.flesch_kincaid_grade),
                format!("{:.1}", r.gunning_fog),
                format!("{:.1}", r.coleman_liau),
                format!("{:.1}", r.automated_readability),
            ]
        };
        let secs = sections(doc);
        if secs.len() > 1 {
            for s in &secs {
                t.row(cells(&s.title, &s.scores));
            }
        }
        t.row(cells("whole document", &document(doc)));
        t.print();
    }
    println!(
        "\nease: Flesch reading ease (higher is easier). grade: Flesch-Kincaid. fog: Gunning. cli: Coleman-Liau. ari: Automated Readability."
    );
    Ok(())
}
