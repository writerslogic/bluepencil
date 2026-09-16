use anyhow::Result;
use bluepencil_core::analysis::hedges;

use crate::cli::ListArgs;
use crate::context::Context;

pub fn run(ctx: &Context, args: &ListArgs) -> Result<()> {
    super::list_command(ctx, args, hedges::find, "hedge")
}
