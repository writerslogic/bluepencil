use anyhow::{Result, bail};

use crate::exit::Status;

/// JSON Schema for a command's `--json` output. Only commands migrated off ad-hoc
/// `serde_json::json!()` shapes onto real structs are listed here; the rest still produce
/// valid JSON, they just don't have a schema to check it against yet.
pub fn run(command: &str) -> Result<Status> {
    let schema = match command {
        "count" => schemars::schema_for!(super::count::CountOutput),
        "adverbs" | "cliches" | "filter" | "hedges" | "passive" | "tics" => {
            schemars::schema_for!(super::ListOutput)
        }
        other => bail!("no schema for `{other}` yet"),
    };
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(Status::Ok)
}
