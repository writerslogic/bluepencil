use anyhow::Result;
use bluepencil_core::analysis::diversity::diversity;

use crate::cli::UniqueArgs;
use crate::context::Context;
use crate::output::human::thousands;
use crate::output::json;
use crate::output::table::{Align, Table};

pub fn run(ctx: &Context, args: &UniqueArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let mut rows = Vec::new();
    let mut all: Vec<&str> = Vec::new();
    for d in &docs {
        let words: Vec<&str> = d.words().map(|w| w.lower.as_str()).collect();
        rows.push((d.name.clone(), diversity(&words, args.window)));
        all.extend(words);
    }
    let total = diversity(&all, args.window);

    if ctx.json {
        let files: Vec<_> = rows.iter().map(|(n, d)| serde_json::json!({ "file": n, "diversity": d })).collect();
        return json::print(&serde_json::json!({ "files": files, "total": total }));
    }
    let mut t = Table::new(&[
        ("file", Align::Left),
        ("words", Align::Right),
        ("unique", Align::Right),
        ("ttr", Align::Right),
        ("mattr", Align::Right),
    ]);
    let cells = |n: &str, d: &bluepencil_core::analysis::diversity::Diversity| {
        vec![n.to_string(), thousands(d.words), thousands(d.unique), format!("{:.3}", d.ttr), format!("{:.3}", d.mattr)]
    };
    for (n, d) in &rows {
        t.row(cells(n, d));
    }
    if rows.len() > 1 {
        t.row(cells("total", &total));
    }
    t.print();
    println!(
        "\nMATTR uses a {}-word window and is comparable across texts of different lengths; raw TTR is not.",
        args.window
    );
    Ok(())
}
