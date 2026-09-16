use std::io::Read;

use anyhow::{Context as _, Result, bail};
use bluepencil_core::analysis::diff::{ChangeKind, word_diff};

use crate::cli::WdiffArgs;
use crate::context::Context;
use crate::output::json;

fn read(path: &str) -> Result<String> {
    if path == "-" {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).context("reading stdin")?;
        return Ok(text);
    }
    std::fs::read_to_string(path).with_context(|| format!("reading {path}"))
}

pub fn run(ctx: &Context, args: &WdiffArgs) -> Result<()> {
    if args.old == "-" && args.new == "-" {
        bail!("only one of `old`/`new` can be `-`");
    }
    let old = read(&args.old)?;
    let new = read(&args.new)?;
    let diff = word_diff(&old, &new);

    if ctx.json {
        return json::print(
            &serde_json::json!({ "added": diff.added, "removed": diff.removed, "changes": diff.changes }),
        );
    }

    for change in &diff.changes {
        match change.tag {
            ChangeKind::Delete => print!("[-{}-] ", change.word),
            ChangeKind::Insert => print!("{{+{}+}} ", change.word),
            ChangeKind::Equal => print!("{} ", change.word),
        }
    }
    println!("\n\n{} word(s) added, {} removed", diff.added, diff.removed);
    Ok(())
}
