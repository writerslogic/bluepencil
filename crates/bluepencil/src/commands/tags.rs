use anyhow::Result;
use bluepencil_core::analysis::dialogue_tags::{TagKind, attributions};
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::ListArgs;
use crate::context::Context;
use crate::output::human::ranked;
use crate::output::table::{Align, Table};
use crate::output::{json, location};

#[derive(Serialize, JsonSchema)]
pub struct TagRow {
    pub location: String,
    pub kind: &'static str,
    pub verb: String,
}

#[derive(Serialize, JsonSchema)]
pub struct TagsOutput {
    pub plain: usize,
    pub adverb: usize,
    pub showy: usize,
    pub untagged: usize,
    pub tags: Vec<TagRow>,
}

fn bucket(kind: TagKind, adverb: bool) -> &'static str {
    match (kind, adverb) {
        (TagKind::Showy, _) => "showy",
        (TagKind::Plain, true) => "adverb",
        (TagKind::Plain, false) => "plain",
    }
}

pub fn run(ctx: &Context, args: &ListArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let mut untagged = 0usize;
    let mut rows = Vec::new();

    for doc in &docs {
        for a in attributions(doc, &ctx.lexicons) {
            let Some(tag_span) = a.tag else {
                untagged += 1;
                continue;
            };
            let kind = bucket(a.kind.expect("tag implies kind"), a.adverb);
            rows.push(TagRow { location: location(doc, tag_span), kind, verb: doc.text(tag_span).to_lowercase() });
        }
    }

    let plain = rows.iter().filter(|r| r.kind == "plain").count();
    let adverb = rows.iter().filter(|r| r.kind == "adverb").count();
    let showy = rows.iter().filter(|r| r.kind == "showy").count();

    if ctx.json {
        return json::print(&TagsOutput {
            plain,
            adverb,
            showy,
            untagged,
            tags: rows.into_iter().take(ctx.limit).collect(),
        });
    }

    if rows.is_empty() && untagged == 0 {
        println!("No dialogue found.");
        return Ok(());
    }

    if args.summary {
        let tally: Vec<(String, usize)> = {
            let mut m = std::collections::HashMap::<String, usize>::new();
            for r in &rows {
                *m.entry(format!("{} ({})", r.verb, r.kind)).or_default() += 1;
            }
            let mut v: Vec<_> = m.into_iter().collect();
            v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            v
        };
        ranked(&tally, "tag", ctx.limit, None);
    } else {
        let mut t = Table::new(&[("location", Align::Left), ("kind", Align::Left), ("verb", Align::Left)]);
        for r in rows.iter().take(ctx.limit) {
            t.row(vec![r.location.clone(), r.kind.to_string(), r.verb.clone()]);
        }
        t.print();
    }
    println!("\n{plain} plain, {adverb} adverb-modified, {showy} showy, {untagged} untagged");
    Ok(())
}
