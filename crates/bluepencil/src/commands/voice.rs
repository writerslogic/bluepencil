//! `voice`: does each character sound like themselves? The model reads all the dialogue,
//! describes how each speaking character talks, and flags lines that sound like someone else.
//! Each flagged line is a quote pinned to the source, as in `facts`.

use anyhow::{Context as _, Result};
use bluepencil_core::Document;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::cli::Input;
use crate::context::Context;
use crate::llm;
use crate::output::json as json_out;

const MAX_TOKENS: u32 = 32_000;

const SYSTEM: &str = "\
You are a dialogue editor reading a manuscript. Every line is prefixed `file:line| `. Attend \
to the dialogue: who speaks, and how.

First, for each character who speaks more than a handful of lines, write a short profile of \
their voice as the text actually shows it: sentence length, vocabulary and register, contractions \
and slang, verbal tics, what they talk about and what they avoid, how they handle other people. \
Describe what is on the page, not what the character is like. Count their lines roughly.

Then flag lines of dialogue that break the pattern: a line that sounds like a different \
character, like the narrator, or like no one in particular, with vocabulary or rhythm the speaker \
has not used anywhere else. Quote the line exactly as it appears, cite its file and line, name the \
speaker, and in one or two sentences say what is off and, if it is clear, who it does sound like. \
Allow for deliberate change: a character under stress, drunk, lying, or performing for someone \
may sound different on purpose, and the surrounding text usually says so. Do not flag a line you \
are not confident about; the writer will check every one by hand. Return no finding for a \
character whose voice is consistent.";

#[derive(Deserialize)]
struct Reply {
    characters: Vec<ReplyCharacter>,
    findings: Vec<ReplyFinding>,
}

#[derive(Deserialize)]
struct ReplyCharacter {
    name: String,
    lines: usize,
    profile: String,
}

#[derive(Deserialize)]
struct ReplyFinding {
    character: String,
    file: String,
    line: usize,
    quote: String,
    note: String,
}

fn reply_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "characters": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "lines": {"type": "integer"},
                        "profile": {"type": "string"},
                    },
                    "required": ["name", "lines", "profile"],
                    "additionalProperties": false,
                },
            },
            "findings": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "character": {"type": "string"},
                        "file": {"type": "string"},
                        "line": {"type": "integer"},
                        "quote": {"type": "string"},
                        "note": {"type": "string"},
                    },
                    "required": ["character", "file", "line", "quote", "note"],
                    "additionalProperties": false,
                },
            },
        },
        "required": ["characters", "findings"],
        "additionalProperties": false,
    })
}

#[derive(Serialize, JsonSchema)]
pub struct Character {
    pub name: String,
    pub lines: usize,
    pub profile: String,
}

#[derive(Serialize, JsonSchema)]
pub struct VoiceFinding {
    pub character: String,
    #[serde(flatten)]
    pub pin: llm::Pin,
    pub quote: String,
    pub note: String,
}

#[derive(Serialize, JsonSchema)]
pub struct VoiceOutput {
    pub characters: Vec<Character>,
    pub findings: Vec<VoiceFinding>,
}

fn verify(docs: &[Document], reply: Reply) -> VoiceOutput {
    let mut characters: Vec<Character> =
        reply.characters.into_iter().map(|c| Character { name: c.name, lines: c.lines, profile: c.profile }).collect();
    characters.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| a.name.cmp(&b.name)));
    let findings = reply
        .findings
        .into_iter()
        .map(|f| VoiceFinding {
            pin: llm::pin(docs, &f.file, f.line, &f.quote),
            character: f.character,
            quote: f.quote,
            note: f.note,
        })
        .collect();
    VoiceOutput { characters, findings }
}

pub fn run(ctx: &Context, input: &Input) -> Result<()> {
    let docs = ctx.documents(input)?;
    let client = llm::client(ctx)?;
    let text = llm::numbered(&docs)?;
    let words: usize = docs.iter().map(Document::word_count).sum();
    eprintln!("bluepencil: reading {} words across {} files with {}…", words, docs.len(), client.model());
    let user = format!("Profile each speaking character's voice and flag off-voice lines.\n\n{text}");
    let value = client.complete_json(SYSTEM, &user, &reply_schema(), MAX_TOKENS)?;
    let reply: Reply = serde_json::from_value(value).context("the model's reply did not match the expected shape")?;
    let out = verify(&docs, reply);

    if ctx.json {
        return json_out::print(&out);
    }

    if out.characters.is_empty() {
        println!("No speaking characters found.");
        return Ok(());
    }
    println!("Voices");
    for c in &out.characters {
        println!("  {} ({} lines)", c.name, c.lines);
        println!("      {}", c.profile);
    }
    if out.findings.is_empty() {
        println!("\nNo off-voice lines found.");
        return Ok(());
    }
    println!("\nOff-voice lines");
    for f in &out.findings {
        println!("  {}  {}{}", f.pin.location, f.character, llm::unverified_mark(&f.pin));
        println!("      \"{}\"", f.quote);
        println!("      → {}", f.note);
    }
    println!("\n{} off-voice lines across {} speaking characters", out.findings.len(), out.characters.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bluepencil_core::Format;

    #[test]
    fn characters_sort_by_lines_and_findings_are_pinned() {
        let doc = Document::parse("a.md", "\"Aye, well.\" said Tom.\n\n\"Indubitably,\" said Tom.", Format::Markdown);
        let reply = Reply {
            characters: vec![
                ReplyCharacter { name: "Ann".into(), lines: 2, profile: "clipped".into() },
                ReplyCharacter { name: "Tom".into(), lines: 9, profile: "dialect".into() },
            ],
            findings: vec![ReplyFinding {
                character: "Tom".into(),
                file: "a.md".into(),
                line: 3,
                quote: "Indubitably,".into(),
                note: "Latinate for a dialect speaker".into(),
            }],
        };
        let out = verify(&[doc], reply);
        assert_eq!(out.characters[0].name, "Tom");
        assert_eq!(out.findings[0].pin.location, "a.md:3:2");
        assert!(out.findings[0].pin.verified);
    }

    #[test]
    fn reply_schema_satisfies_structured_output_rules() {
        llm::assert_schema_is_strict(&reply_schema());
    }
}
