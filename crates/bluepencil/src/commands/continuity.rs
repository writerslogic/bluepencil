use anyhow::Result;
use bluepencil_core::analysis::cast::{NameCluster, name_variants};
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::Input;
use crate::context::Context;
use crate::output::json;
use crate::output::table::{Align, Table};

#[derive(Serialize, JsonSchema)]
pub struct ContinuityOutput {
    pub clusters: Vec<NameCluster>,
}

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    let ignore: std::collections::HashSet<String> =
        ctx.config.continuity.ignore.iter().map(|s| s.to_lowercase()).collect();
    let clusters = name_variants(&docs, &ignore);

    if ctx.json {
        return json::print(&ContinuityOutput { clusters });
    }

    if clusters.is_empty() {
        println!("No likely spelling drift found.");
        return Ok(());
    }

    let mut t = Table::new(&[("spelling", Align::Left), ("count", Align::Right)]);
    for (i, cluster) in clusters.iter().enumerate() {
        if i > 0 {
            t.row(vec![String::new(), String::new()]);
        }
        for v in &cluster.variants {
            t.row(vec![v.spelling.clone(), v.count.to_string()]);
        }
    }
    t.print();
    println!(
        "\n{} possible name{} found. If any are genuinely distinct characters, add them to \
        [continuity].ignore in bluepencil.toml.",
        clusters.len(),
        if clusters.len() == 1 { "" } else { "s" }
    );
    Ok(())
}
