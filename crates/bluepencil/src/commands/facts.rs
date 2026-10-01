//! `facts`: a continuity ledger. The model reads the whole manuscript, records every concrete
//! claim about a character, place, or object, and reports where the claims disagree. Each
//! claim comes back as a quote with a cited line; the quote is then located in the source
//! here, so every reported position is real even though the reading is the model's.

use anyhow::{Context as _, Result};
use bluepencil_core::Document;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::cli::FactsArgs;
use crate::context::Context;
use crate::llm;
use crate::output::json as json_out;

/// Ledgers for a long novel run to thousands of lines of JSON.
const MAX_TOKENS: u32 = 32_000;

const SYSTEM: &str = "\
You are a continuity editor reading a manuscript. Every line is prefixed `file:line| `. Build a \
ledger of concrete, checkable facts the text states or clearly implies about named characters, \
places, and significant objects: physical description (eye and hair color, height, build, scars, \
age), names and spellings, family and other relationships, possessions, where someone lives or \
works, dates, days of the week, time elapsed, weather within a scene, who is present in a scene, \
what someone knows, injuries and their side, and what has been broken, lost, or given away.

For each fact, give the entity's most common name, a short attribute label such as `eye color` or \
`sister`, the value as stated, the file and line it came from, and a short quote copied exactly \
from that line, long enough to be found but no more than a sentence. Skip opinions, metaphors, \
and anything a character says that the text marks as a lie or mistake.

Then list contradictions: sets of two or more facts about the same entity and attribute that \
cannot all be true, allowing for change the story accounts for (hair that is dyed, an injury that \
heals, a child who grows up). For each, cite the fact indices and write one or two sentences \
saying what disagrees and, if the text makes it clear, which version the story needs. Do not \
report a contradiction you are not confident about; the writer will check every one by hand.";

#[derive(Deserialize)]
struct Reply {
    facts: Vec<ReplyFact>,
    contradictions: Vec<ReplyContradiction>,
}

#[derive(Deserialize)]
struct ReplyFact {
    entity: String,
    attribute: String,
    value: String,
    file: String,
    line: usize,
    quote: String,
}

#[derive(Deserialize)]
struct ReplyContradiction {
    entity: String,
    attribute: String,
    facts: Vec<usize>,
    note: String,
}

fn reply_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "facts": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "entity": {"type": "string"},
                        "attribute": {"type": "string"},
                        "value": {"type": "string"},
                        "file": {"type": "string"},
                        "line": {"type": "integer"},
                        "quote": {"type": "string"},
                    },
                    "required": ["entity", "attribute", "value", "file", "line", "quote"],
                    "additionalProperties": false,
                },
            },
            "contradictions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "entity": {"type": "string"},
                        "attribute": {"type": "string"},
                        "facts": {"type": "array", "items": {"type": "integer"}},
                        "note": {"type": "string"},
                    },
                    "required": ["entity", "attribute", "facts", "note"],
                    "additionalProperties": false,
                },
            },
        },
        "required": ["facts", "contradictions"],
        "additionalProperties": false,
    })
}

#[derive(Serialize, JsonSchema)]
pub struct Fact {
    pub entity: String,
    pub attribute: String,
    pub value: String,
    #[serde(flatten)]
    pub pin: llm::Pin,
    pub quote: String,
}

#[derive(Serialize, JsonSchema)]
pub struct Contradiction {
    pub entity: String,
    pub attribute: String,
    /// Indices into `facts`.
    pub facts: Vec<usize>,
    pub note: String,
}

#[derive(Serialize, JsonSchema)]
pub struct FactsOutput {
    pub facts: Vec<Fact>,
    pub contradictions: Vec<Contradiction>,
}

/// Pins each cited quote to the source. Index positions are preserved so contradictions still
/// point at the right facts.
fn verify(docs: &[Document], reply: Reply) -> FactsOutput {
    let facts: Vec<Fact> = reply
        .facts
        .into_iter()
        .map(|f| Fact {
            pin: llm::pin(docs, &f.file, f.line, &f.quote),
            entity: f.entity,
            attribute: f.attribute,
            value: f.value,
            quote: f.quote,
        })
        .collect();
    let contradictions = reply
        .contradictions
        .into_iter()
        .map(|c| Contradiction {
            entity: c.entity,
            attribute: c.attribute,
            facts: c.facts.into_iter().filter(|&i| i < facts.len()).collect(),
            note: c.note,
        })
        .filter(|c| c.facts.len() >= 2)
        .collect();
    FactsOutput { facts, contradictions }
}

pub fn run(ctx: &Context, args: &FactsArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let client = llm::client(ctx)?;
    let text = llm::numbered(&docs)?;
    let words: usize = docs.iter().map(Document::word_count).sum();
    eprintln!("bluepencil: reading {} words across {} files with {}…", words, docs.len(), client.model());
    let user = format!("Build the continuity ledger for this manuscript.\n\n{text}");
    let value = client.complete_json(SYSTEM, &user, &reply_schema(), MAX_TOKENS)?;
    let reply: Reply = serde_json::from_value(value).context("the model's reply did not match the expected shape")?;
    let out = verify(&docs, reply);

    if ctx.json {
        return json_out::print(&out);
    }

    if out.contradictions.is_empty() {
        println!("No contradictions found across {} facts.", out.facts.len());
    }
    for c in &out.contradictions {
        println!("\n{} / {}", c.entity, c.attribute);
        for &i in &c.facts {
            let f = &out.facts[i];
            println!("  {}  {}{}", f.pin.location, f.value, llm::unverified_mark(&f.pin));
            println!("      \"{}\"", f.quote);
        }
        println!("  → {}", c.note);
    }
    if args.ledger {
        println!("\nLedger");
        let mut facts: Vec<&Fact> = out.facts.iter().collect();
        facts.sort_by(|a, b| a.entity.cmp(&b.entity).then_with(|| a.attribute.cmp(&b.attribute)));
        for f in facts {
            println!(
                "  {}  {} / {}: {}{}",
                f.pin.location,
                f.entity,
                f.attribute,
                f.value,
                llm::unverified_mark(&f.pin)
            );
        }
    }
    let unverified = out.facts.iter().filter(|f| !f.pin.verified).count();
    println!(
        "\n{} contradictions across {} facts{}{}",
        out.contradictions.len(),
        out.facts.len(),
        if unverified > 0 { format!(", {unverified} quotes not found verbatim") } else { String::new() },
        if args.ledger { "" } else { " (use --ledger to list every fact)" }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bluepencil_core::Format;

    fn reply() -> Reply {
        Reply {
            facts: vec![
                ReplyFact {
                    entity: "Mara".into(),
                    attribute: "eye color".into(),
                    value: "grey".into(),
                    file: "a.md".into(),
                    line: 1,
                    quote: "her grey eyes".into(),
                },
                ReplyFact {
                    entity: "Mara".into(),
                    attribute: "eye color".into(),
                    value: "brown".into(),
                    file: "a.md".into(),
                    line: 3,
                    quote: "not in the text".into(),
                },
            ],
            contradictions: vec![
                ReplyContradiction {
                    entity: "Mara".into(),
                    attribute: "eye color".into(),
                    facts: vec![0, 1, 7],
                    note: "grey then brown".into(),
                },
                ReplyContradiction {
                    entity: "Mara".into(),
                    attribute: "age".into(),
                    facts: vec![0, 9],
                    note: "dangling".into(),
                },
            ],
        }
    }

    #[test]
    fn quotes_are_pinned_to_source_and_unfound_ones_are_marked() {
        let doc = Document::parse("a.md", "She met her grey eyes.\n\nLater.", Format::Markdown);
        let out = verify(&[doc], reply());
        assert_eq!(out.facts[0].pin.location, "a.md:1:9");
        assert!(out.facts[0].pin.verified);
        assert_eq!(out.facts[1].pin.location, "a.md:3");
        assert!(!out.facts[1].pin.verified);
    }

    #[test]
    fn contradictions_drop_dangling_indices_and_need_two_facts() {
        let doc = Document::parse("a.md", "her grey eyes", Format::Markdown);
        let out = verify(&[doc], reply());
        assert_eq!(out.contradictions.len(), 1, "a contradiction left with one valid fact is dropped");
        assert_eq!(out.contradictions[0].facts, vec![0, 1]);
    }

    #[test]
    fn reply_schema_satisfies_structured_output_rules() {
        llm::assert_schema_is_strict(&reply_schema());
    }
}
