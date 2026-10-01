//! `--explain`: an editor's note on each finding, written by a language model.
//!
//! The deterministic analyses stay the source of truth. This module takes their findings,
//! attaches the surrounding sentence as evidence, asks the model for a verdict and a short
//! note per finding, and hands the notes back aligned to the findings it was given. Nothing
//! here changes what is found or how `check` scores it.

use anyhow::{Context as _, Result};
use bluepencil_core::{Document, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::context::Context;
use crate::llm;
use crate::output::{excerpt, location};

/// How many characters of surrounding sentence to send with each finding.
const CONTEXT_CHARS: usize = 400;
/// Forty short notes fit comfortably; the cap only guards against runaway replies.
const MAX_TOKENS: u32 = 16_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// The finding is a real weakness worth revising.
    Fix,
    /// Defensible either way; the writer should look.
    Consider,
    /// A false positive or a deliberate choice; leave it.
    Keep,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fix => "fix",
            Self::Consider => "consider",
            Self::Keep => "keep",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Note {
    pub verdict: Verdict,
    pub note: String,
}

/// One finding as the model sees it.
#[derive(Serialize)]
struct Item<'a> {
    index: usize,
    file: &'a str,
    location: String,
    rule: &'a str,
    message: &'a str,
    text: &'a str,
    in_dialogue: bool,
    context: String,
}

#[derive(Deserialize)]
struct Reply {
    notes: Vec<ReplyNote>,
}

#[derive(Deserialize)]
struct ReplyNote {
    index: usize,
    verdict: Verdict,
    note: String,
}

/// A located finding that `--explain` can annotate. Commands adapt their own result types to this.
pub struct Target<'a> {
    pub doc: &'a Document,
    pub span: Span,
    pub rule: &'a str,
    pub message: &'a str,
}

const SYSTEM: &str = "\
You are a line editor annotating the output of a deterministic prose linter for a writer. \
Each finding is a word or phrase the linter flagged, with the rule that matched, the sentence \
around it, and whether it sits inside dialogue. For every finding, give one verdict and one note.

Verdicts: `fix` when the flagged text weakens the prose and should change; `consider` when a \
careful writer might keep or change it; `keep` when the match is a false positive, idiomatic, \
deliberate for voice, or a line of dialogue that sounds like a person talking.

Notes are one or two plain sentences addressed to the writer. Name the specific problem in this \
sentence, not the rule in general. When the verdict is `fix`, include a concrete rewrite of the \
flagged words in context. Do not praise, hedge, or restate the rule. Judge each finding on its own \
sentence; do not assume the writer's intent beyond what the text shows.

Return a note for every index you were given, in any order, and no others.";

/// Response schema for the structured-output call. Objects carry `additionalProperties: false`
/// because the API requires it; no recursion and no length constraints, which it rejects.
fn reply_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "notes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "index": {"type": "integer"},
                        "verdict": {"type": "string", "enum": ["fix", "consider", "keep"]},
                        "note": {"type": "string"},
                    },
                    "required": ["index", "verdict", "note"],
                    "additionalProperties": false,
                },
            },
        },
        "required": ["notes"],
        "additionalProperties": false,
    })
}

/// Sentences overlapping `span`, flattened to one line and clipped.
fn surrounding(doc: &Document, span: Span) -> String {
    let mut start = span.start;
    let mut end = span.end;
    for s in doc.sentences() {
        if s.span.end <= span.start || s.span.start >= span.end {
            continue;
        }
        start = start.min(s.span.start);
        end = end.max(s.span.end);
    }
    excerpt(doc, Span::new(start, end), CONTEXT_CHARS)
}

fn items<'a>(targets: &'a [Target<'a>]) -> Vec<Item<'a>> {
    targets
        .iter()
        .enumerate()
        .map(|(index, t)| Item {
            index,
            file: &t.doc.name,
            location: location(t.doc, t.span),
            rule: t.rule,
            message: t.message,
            text: t.doc.text(t.span),
            in_dialogue: t.doc.in_dialogue(t.span),
            context: surrounding(t.doc, t.span),
        })
        .collect()
}

fn user_message(items: &[Item<'_>]) -> Result<String> {
    let findings = serde_json::to_string_pretty(items)?;
    Ok(format!("Annotate these {} findings.\n\n{findings}", items.len()))
}

/// Turns the model's reply into notes aligned with `targets`; a missing or out-of-range index
/// leaves `None` rather than shifting the rest.
fn align(reply: Reply, count: usize) -> Vec<Option<Note>> {
    let mut notes = vec![None; count];
    for n in reply.notes {
        if let Some(slot) = notes.get_mut(n.index) {
            *slot = Some(Note { verdict: n.verdict, note: n.note });
        }
    }
    notes
}

/// Asks the model for a note on each target. Only the first `model.max_findings` targets are
/// sent; the rest come back as `None`, and a line on stderr says so.
pub fn annotate(ctx: &Context, targets: &[Target<'_>]) -> Result<Vec<Option<Note>>> {
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    let cfg = &ctx.config.model;
    let client = llm::client(ctx)?;
    let sent = targets.len().min(cfg.max_findings);
    if sent < targets.len() {
        eprintln!(
            "bluepencil: explaining the first {sent} of {} findings (raise model.max_findings to send more)",
            targets.len()
        );
    } else {
        eprintln!("bluepencil: explaining {sent} findings with {}…", client.model());
    }
    let items = items(&targets[..sent]);
    let value = client.complete_json(SYSTEM, &user_message(&items)?, &reply_schema(), MAX_TOKENS)?;
    let reply: Reply = serde_json::from_value(value).context("the model's reply did not match the expected shape")?;
    let mut notes = align(reply, sent);
    notes.resize(targets.len(), None);
    Ok(notes)
}

/// Human-readable rendering of one note, indented under its finding.
pub fn print_note(note: &Note) {
    println!("    → {}: {}", note.verdict.label(), note.note);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bluepencil_core::Format;

    fn doc() -> Document {
        Document::parse("ch.md", "He was very tired. \"I just think so,\" she said. The end.", Format::Markdown)
    }

    fn find(doc: &Document, needle: &str) -> Span {
        let start = doc.source.find(needle).unwrap();
        Span::new(start, start + needle.len())
    }

    #[test]
    fn context_is_the_enclosing_sentence_and_dialogue_is_marked() {
        let doc = doc();
        let very = find(&doc, "very");
        let just = find(&doc, "just");
        let targets = [
            Target { doc: &doc, span: very, rule: "hedges", message: "very" },
            Target { doc: &doc, span: just, rule: "hedges", message: "just" },
        ];
        let items = items(&targets);
        assert_eq!(items[0].context, "He was very tired.");
        assert!(!items[0].in_dialogue);
        assert_eq!(items[1].index, 1);
        assert!(items[1].in_dialogue, "`just` sits inside quoted speech");
        assert!(items[1].context.contains("I just think so"));
    }

    #[test]
    fn notes_align_by_index_and_tolerate_gaps_and_junk() {
        let reply = Reply {
            notes: vec![
                ReplyNote { index: 2, verdict: Verdict::Keep, note: "c".into() },
                ReplyNote { index: 0, verdict: Verdict::Fix, note: "a".into() },
                ReplyNote { index: 9, verdict: Verdict::Fix, note: "out of range".into() },
            ],
        };
        let notes = align(reply, 3);
        assert_eq!(notes[0].as_ref().unwrap().verdict, Verdict::Fix);
        assert!(notes[1].is_none());
        assert_eq!(notes[2].as_ref().unwrap().note, "c");
    }

    #[test]
    fn reply_schema_satisfies_structured_output_rules() {
        llm::assert_schema_is_strict(&reply_schema());
    }
}
