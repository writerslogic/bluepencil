//! `scenes`: a scene audit. The model reads the manuscript with the deterministic pacing
//! profile from `arc` as evidence, splits it into scenes, and for each one names the goal,
//! the conflict, what changes, and whether it earns its length.

use anyhow::{Context as _, Result};
use bluepencil_core::Document;
use bluepencil_core::analysis::arc::arc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::cli::Input;
use crate::context::Context;
use crate::llm;
use crate::output::json as json_out;
use crate::output::table::{Align, Table};

const MAX_TOKENS: u32 = 32_000;

const SYSTEM: &str = "\
You are a developmental editor auditing a manuscript scene by scene. Every line of the text is \
prefixed `file:line| `. Before the text you are given a pacing profile the tool measured for \
each section: word count, share of words in dialogue, and mean sentence length. Use it as \
evidence, not as a verdict.

Split the text into scenes: a continuous stretch in one place and time with one set of \
characters. A heading often but not always starts one; a line break with a change of place, time, \
or viewpoint does too. For each scene, cite the file and the first line, give it a short title, \
and quote the opening few words exactly as they appear so the start can be found.

Then judge it. What does the viewpoint character want in the scene? What stands in the way? What \
is different at the end from the beginning, for the character or for what the reader knows? A \
scene where nothing changes is usually the problem, however well written. Give a verdict: `keep` \
when it does its job at this length; `tighten` when the job is done but the scene outlasts it, \
and say which part; `cut` when nothing changes and nothing later depends on it; `merge` when it \
and a neighbour do the same work, and name the neighbour. Write one to three sentences the writer \
can act on. Be direct, and be specific to this scene; do not restate the definitions above.";

#[derive(Deserialize)]
struct Reply {
    scenes: Vec<ReplyScene>,
}

#[derive(Deserialize)]
struct ReplyScene {
    title: String,
    file: String,
    line: usize,
    opening: String,
    goal: String,
    conflict: String,
    change: String,
    verdict: Verdict,
    note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Keep,
    Tighten,
    Cut,
    Merge,
}

impl Verdict {
    fn label(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Tighten => "tighten",
            Self::Cut => "cut",
            Self::Merge => "merge",
        }
    }
}

fn reply_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "scenes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "file": {"type": "string"},
                        "line": {"type": "integer"},
                        "opening": {"type": "string"},
                        "goal": {"type": "string"},
                        "conflict": {"type": "string"},
                        "change": {"type": "string"},
                        "verdict": {"type": "string", "enum": ["keep", "tighten", "cut", "merge"]},
                        "note": {"type": "string"},
                    },
                    "required": ["title", "file", "line", "opening", "goal", "conflict", "change", "verdict", "note"],
                    "additionalProperties": false,
                },
            },
        },
        "required": ["scenes"],
        "additionalProperties": false,
    })
}

#[derive(Serialize, JsonSchema)]
pub struct Scene {
    pub title: String,
    #[serde(flatten)]
    pub pin: llm::Pin,
    pub goal: String,
    pub conflict: String,
    pub change: String,
    pub verdict: Verdict,
    pub note: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ScenesOutput {
    pub scenes: Vec<Scene>,
}

/// The deterministic pacing profile, as a compact table the model reads before the text.
fn profile(ctx: &Context, docs: &[Document]) -> String {
    let mut out = String::from("Measured pacing profile (section | words | dialogue | mean sentence words):\n");
    for doc in docs {
        for s in arc(doc, &ctx.lexicons) {
            out.push_str(&format!(
                "{} | {} | {} | {:.0}% | {:.1}\n",
                doc.name,
                s.section,
                s.words,
                s.dialogue_ratio * 100.0,
                s.sentence_words_mean
            ));
        }
    }
    out
}

fn verify(docs: &[Document], reply: Reply) -> ScenesOutput {
    let scenes = reply
        .scenes
        .into_iter()
        .map(|s| Scene {
            pin: llm::pin(docs, &s.file, s.line, &s.opening),
            title: s.title,
            goal: s.goal,
            conflict: s.conflict,
            change: s.change,
            verdict: s.verdict,
            note: s.note,
        })
        .collect();
    ScenesOutput { scenes }
}

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    let client = llm::client(ctx)?;
    let text = llm::numbered(&docs)?;
    let words: usize = docs.iter().map(Document::word_count).sum();
    eprintln!("bluepencil: reading {} words across {} files with {}…", words, docs.len(), client.model());
    let user = format!("Audit this manuscript scene by scene.\n\n{}\n{text}", profile(ctx, &docs));
    let value = client.complete_json(SYSTEM, &user, &reply_schema(), MAX_TOKENS)?;
    let reply: Reply = serde_json::from_value(value).context("the model's reply did not match the expected shape")?;
    let out = verify(&docs, reply);

    if ctx.json {
        return json_out::print(&out);
    }

    if out.scenes.is_empty() {
        println!("No scenes found.");
        return Ok(());
    }
    let mut t = Table::new(&[("scene", Align::Left), ("starts", Align::Left), ("verdict", Align::Left)]);
    for s in &out.scenes {
        t.row(vec![s.title.chars().take(40).collect(), s.pin.location.clone(), s.verdict.label().to_string()]);
    }
    t.print();
    for s in &out.scenes {
        println!("\n{}  {}{}", s.pin.location, s.title, llm::unverified_mark(&s.pin));
        println!("    goal      {}", s.goal);
        println!("    conflict  {}", s.conflict);
        println!("    change    {}", s.change);
        println!("    → {}: {}", s.verdict.label(), s.note);
    }
    let flagged = out.scenes.iter().filter(|s| s.verdict != Verdict::Keep).count();
    println!("\n{} scenes, {flagged} to look at", out.scenes.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bluepencil_core::Format;

    #[test]
    fn scene_openings_are_pinned_to_source() {
        let doc = Document::parse("a.md", "# One\n\nThe train was late.\n", Format::Markdown);
        let reply = Reply {
            scenes: vec![ReplyScene {
                title: "Platform".into(),
                file: "a.md".into(),
                line: 3,
                opening: "The train was late".into(),
                goal: "g".into(),
                conflict: "c".into(),
                change: "none".into(),
                verdict: Verdict::Cut,
                note: "n".into(),
            }],
        };
        let out = verify(&[doc], reply);
        assert_eq!(out.scenes[0].pin.location, "a.md:3:1");
        assert_eq!(out.scenes[0].verdict, Verdict::Cut);
    }

    #[test]
    fn reply_schema_satisfies_structured_output_rules() {
        llm::assert_schema_is_strict(&reply_schema());
    }
}
