use anyhow::Result;
use bluepencil_core::analysis::counts::Counts;

use crate::cli::Input;
use crate::context::Context;
use crate::output::human::{minutes, thousands};
use crate::output::json;
use crate::output::table::{Align, Table};

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    let rows: Vec<(String, Counts)> = docs.iter().map(|d| (d.name.clone(), Counts::of(d))).collect();
    let total = rows.iter().fold(Counts::default(), |acc, (_, c)| acc.merge(*c));

    if ctx.json {
        let files: Vec<_> = rows.iter().map(|(n, c)| serde_json::json!({ "file": n, "counts": c })).collect();
        return json::print(&serde_json::json!({ "files": files, "total": total }));
    }

    let mut t = Table::new(&[
        ("file", Align::Left),
        ("words", Align::Right),
        ("chars", Align::Right),
        ("sentences", Align::Right),
        ("paragraphs", Align::Right),
        ("reading", Align::Right),
    ]);
    let cells = |name: &str, c: &Counts| {
        vec![
            name.to_string(),
            thousands(c.words),
            thousands(c.characters),
            thousands(c.sentences),
            thousands(c.paragraphs),
            minutes(c.reading_minutes),
        ]
    };
    for (name, c) in &rows {
        t.row(cells(name, c));
    }
    if rows.len() > 1 {
        t.row(cells("total", &total));
    }
    t.print();
    Ok(())
}
