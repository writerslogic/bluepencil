use anyhow::Result;
use bluepencil_core::analysis::dialogue::{DialogueRatio, by_section, ratio};

use crate::cli::Input;
use crate::context::Context;
use crate::output::chart::bar;
use crate::output::human::thousands;
use crate::output::json;
use crate::output::table::{Align, Table};

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    if ctx.json {
        let out: Vec<_> = docs
            .iter()
            .map(|d| {
                let secs: Vec<_> = by_section(d)
                    .into_iter()
                    .map(|(t, r)| serde_json::json!({ "section": t, "dialogue": r }))
                    .collect();
                serde_json::json!({ "file": d.name, "dialogue": ratio(d), "sections": secs })
            })
            .collect();
        return json::print(&out);
    }
    let mut t = Table::new(&[
        ("file / section", Align::Left),
        ("dialogue", Align::Right),
        ("narration", Align::Right),
        ("ratio", Align::Right),
        ("", Align::Left),
    ]);
    let cells = |name: String, r: &DialogueRatio| {
        vec![
            name,
            thousands(r.dialogue_words),
            thousands(r.narration_words),
            format!("{:.0}%", r.ratio * 100.0),
            bar((r.ratio * 1000.0) as usize, 1000, 20),
        ]
    };
    let mut total = DialogueRatio::default();
    for d in &docs {
        let r = ratio(d);
        total = total.merge(r);
        t.row(cells(d.name.clone(), &r));
        let secs = by_section(d);
        if secs.len() > 1 {
            for (title, r) in &secs {
                t.row(cells(format!("  {}", title.chars().take(40).collect::<String>()), r));
            }
        }
    }
    if docs.len() > 1 {
        t.row(cells("total".into(), &total));
    }
    t.print();
    Ok(())
}
