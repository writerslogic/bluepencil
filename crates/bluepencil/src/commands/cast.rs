use anyhow::Result;
use bluepencil_core::analysis::cast::{SectionMentions, mentions_by_section};
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::Input;
use crate::context::Context;
use crate::output::human::file_banner;
use crate::output::json;
use crate::output::table::{Align, Table};

#[derive(Serialize, JsonSchema)]
pub struct FileMentions {
    pub file: String,
    pub sections: Vec<SectionMentions>,
}

#[derive(Serialize, JsonSchema)]
pub struct CastOutput {
    pub files: Vec<FileMentions>,
}

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    let ignore: std::collections::HashSet<String> =
        ctx.config.continuity.ignore.iter().map(|s| s.to_lowercase()).collect();

    if ctx.json {
        let files = docs
            .iter()
            .map(|d| FileMentions { file: d.name.clone(), sections: mentions_by_section(d, &ignore) })
            .collect();
        return json::print(&CastOutput { files });
    }

    let mut any = false;
    for doc in &docs {
        let sections = mentions_by_section(doc, &ignore);
        if sections.is_empty() {
            continue;
        }
        any = true;
        file_banner(&docs, doc);
        let mut t = Table::new(&[("section", Align::Left), ("name", Align::Left), ("count", Align::Right)]);
        for s in &sections {
            for (i, n) in s.names.iter().enumerate() {
                t.row(vec![
                    if i == 0 { s.section.clone() } else { String::new() },
                    n.spelling.clone(),
                    n.count.to_string(),
                ]);
            }
        }
        t.print();
        println!();
    }
    if !any {
        println!("No character mentions found.");
    }
    Ok(())
}
