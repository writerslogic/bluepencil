use anyhow::Result;
use bluepencil_core::analysis::outline::outline;

use crate::cli::Input;
use crate::context::Context;
use crate::output::chart::bar;
use crate::output::human::{file_banner, thousands};
use crate::output::json;

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    if ctx.json {
        let out: Vec<_> = docs.iter().map(|d| serde_json::json!({ "file": d.name, "sections": outline(d) })).collect();
        return json::print(&out);
    }
    for doc in &docs {
        file_banner(&docs, doc);
        let sections = outline(doc);
        let max = sections.iter().map(|s| s.words).max().unwrap_or(0);
        let width = sections.iter().map(|s| s.title.chars().count() + 2 * s.level as usize).max().unwrap_or(0).min(50);
        for s in &sections {
            let indent = "  ".repeat(s.level.saturating_sub(1) as usize);
            let title: String = format!("{indent}{}", s.title).chars().take(50).collect();
            let pad = width.saturating_sub(title.chars().count());
            println!("{title}{}  {:>7}  {}", " ".repeat(pad), thousands(s.words), bar(s.words, max, 30));
        }
    }
    Ok(())
}
