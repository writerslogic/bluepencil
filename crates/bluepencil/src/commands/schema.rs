use anyhow::{Result, bail};

use crate::exit::Status;

/// JSON Schema for a command's `--json` output. Only commands migrated off ad-hoc
/// `serde_json::json!()` shapes onto real structs are listed here; the rest still produce
/// valid JSON, they just don't have a schema to check it against yet.
pub fn run(command: &str) -> Result<Status> {
    let schema = match command {
        "count" => schemars::schema_for!(super::count::CountOutput),
        "adverbs" | "cliches" | "filter" | "hedges" | "passive" | "tics" | "nominal" => {
            schemars::schema_for!(super::ListOutput)
        }
        "check" => schemars::schema_for!(super::check::CheckOutput),
        "continuity" => schemars::schema_for!(super::continuity::ContinuityOutput),
        "cast" => schemars::schema_for!(super::cast::CastOutput),
        "tags" => schemars::schema_for!(super::tags::TagsOutput),
        "speakers" => schemars::schema_for!(Vec<super::speakers::SpeakerTally>),
        "tense" => schemars::schema_for!(Vec<super::tense::TenseOutput>),
        "pov" => schemars::schema_for!(Vec<super::pov::PovOutput>),
        "arc" => schemars::schema_for!(Vec<super::arc::ArcOutput>),
        "facts" => schemars::schema_for!(super::facts::FactsOutput),
        "voice" => schemars::schema_for!(super::voice::VoiceOutput),
        "scenes" => schemars::schema_for!(super::scenes::ScenesOutput),
        "config" => schemars::schema_for!(super::config::ConfigOutput),
        "freq" => schemars::schema_for!(super::freq::FreqOutput),
        "progress" => schemars::schema_for!(super::progress::ProgressOutput),
        "echoes" => schemars::schema_for!(Vec<super::echoes::EchoesFileOutput>),
        "repeats" => schemars::schema_for!(Vec<super::repeats::RepeatsFileOutput>),
        "wdiff" => schemars::schema_for!(super::wdiff::WdiffOutput),
        "overused" => schemars::schema_for!(super::overused::OverusedOutput),
        "dialogue" => schemars::schema_for!(Vec<super::dialogue::DialogueOutput>),
        "outline" => schemars::schema_for!(Vec<super::outline::OutlineOutput>),
        "rhythm" => schemars::schema_for!(Vec<super::rhythm::RhythmOutput>),
        "starters" => schemars::schema_for!(Vec<super::starters::StartersOutput>),
        "readability" => schemars::schema_for!(Vec<super::readability::FileReadability>),
        "report" => schemars::schema_for!(super::report::ReportOutput),
        "unique" => schemars::schema_for!(super::unique::UniqueOutput),
        "histogram" => schemars::schema_for!(bluepencil_core::analysis::distribution::Distribution),
        other => bail!("no schema for `{other}` yet"),
    };
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(Status::Ok)
}
