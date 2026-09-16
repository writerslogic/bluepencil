use anyhow::{Context as _, Result};

use crate::cli::JoinArgs;
use crate::context::Context;

pub fn run(ctx: &Context, args: &JoinArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let joined = docs.iter().map(|d| d.source.trim_end()).collect::<Vec<_>>().join("\n\n");

    match &args.out {
        Some(path) => {
            std::fs::write(path, joined + "\n").with_context(|| format!("writing {}", path.display()))?;
            println!("Wrote {} ({} files)", path.display(), docs.len());
        }
        None => println!("{joined}"),
    }
    Ok(())
}
