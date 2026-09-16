use anyhow::Result;
use bluepencil_core::analysis::hapax::hapax;

use crate::cli::Input;
use crate::context::Context;
use crate::output::{human, json, locate};

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    let found = hapax(&docs);
    if ctx.json {
        let out: Vec<_> = found.iter().map(|(d, f)| locate(&docs[*d], f)).collect();
        return json::print(&out);
    }
    let items: Vec<_> = found.iter().map(|(d, f)| (&docs[*d], f)).collect();
    human::findings(&items);
    println!("\n{} words used exactly once", found.len());
    Ok(())
}
