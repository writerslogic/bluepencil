//! `config`: the effective configuration after defaults and the file have been merged, and
//! which file it came from.

use anyhow::Result;
use schemars::JsonSchema;
use serde::Serialize;

use crate::config::Config;
use crate::context::Context;
use crate::output::json;

#[derive(Serialize, JsonSchema)]
pub struct ConfigOutput<'a> {
    /// The `bluepencil.toml` in use, or `null` when every value is a default.
    pub path: Option<String>,
    pub config: &'a Config,
}

pub fn run(ctx: &Context) -> Result<()> {
    let path = ctx.config_path().map(|p| p.display().to_string());
    if ctx.json {
        return json::print(&ConfigOutput { path, config: &ctx.config });
    }
    match &path {
        Some(p) => println!("# effective configuration from {p}"),
        None => println!("# effective configuration: built-in defaults (no bluepencil.toml found)"),
    }
    print!("{}", toml::to_string_pretty(&ctx.config)?);
    Ok(())
}
