//! The model-backed layer. Everything here is opt-in, leaves the machine, and is
//! non-deterministic; the analyses in `bluepencil-core` never depend on it.

pub mod claude;

use anyhow::{Result, bail};
use bluepencil_core::Document;

use crate::context::Context;

/// Rough upper bound on prompt size, in characters, before a request is refused outright.
/// About 3.5 characters per token leaves room under a 1M-token context for the reply.
const MAX_PROMPT_CHARS: usize = 3_000_000;

pub fn client(ctx: &Context) -> Result<claude::Claude> {
    claude::Claude::from_config(&ctx.config.model)
}

/// Every document as `file:line| text`, so the model can cite a line we can check.
pub fn numbered(docs: &[Document]) -> Result<String> {
    let mut out = String::new();
    for doc in docs {
        for (i, line) in doc.source.lines().enumerate() {
            out.push_str(&doc.name);
            out.push(':');
            out.push_str(&(i + 1).to_string());
            out.push_str("| ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
    }
    if out.len() > MAX_PROMPT_CHARS {
        bail!("{} characters of manuscript is more than one request can hold; pass fewer files", out.len());
    }
    Ok(out)
}

/// Finds `quote` in `doc`, preferring the occurrence closest to the line the model cited, and
/// returns its byte span. `None` when the quote is not in the document at all.
pub fn locate_quote(doc: &Document, quote: &str, line_hint: usize) -> Option<bluepencil_core::Span> {
    let quote = quote.trim();
    if quote.is_empty() {
        return None;
    }
    let mut best: Option<(usize, usize)> = None;
    let mut from = 0;
    while let Some(i) = doc.source[from..].find(quote) {
        let start = from + i;
        let line = doc.position(start).line;
        let dist = line.abs_diff(line_hint);
        if best.is_none_or(|(_, d)| dist < d) {
            best = Some((start, dist));
        }
        from = start + quote.len();
    }
    best.map(|(start, _)| bluepencil_core::Span::new(start, start + quote.len()))
}

/// A model citation pinned to the source: the quote's real `file:line:column` when it was
/// found, the cited `file:line` otherwise.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct Pin {
    pub location: String,
    /// Whether the quote was found verbatim in the manuscript.
    pub verified: bool,
}

pub fn pin(docs: &[Document], file: &str, line: usize, quote: &str) -> Pin {
    let doc = docs.iter().find(|d| d.name == file);
    match doc.and_then(|d| locate_quote(d, quote, line).map(|s| (d, s))) {
        Some((d, span)) => Pin { location: crate::output::location(d, span), verified: true },
        None => Pin { location: format!("{file}:{line}"), verified: false },
    }
}

/// Marker appended to a human-readable line whose quote could not be found.
pub fn unverified_mark(pin: &Pin) -> &'static str {
    if pin.verified { "" } else { "  (quote not found verbatim)" }
}

/// Checks every object in a structured-output schema carries `additionalProperties: false`,
/// which the API requires. Used by each command's schema test.
#[cfg(test)]
pub fn assert_schema_is_strict(v: &serde_json::Value) {
    if v["type"] == "object" {
        assert_eq!(v["additionalProperties"], false, "every object needs additionalProperties: false");
        for p in v["properties"].as_object().unwrap().values() {
            assert_schema_is_strict(p);
        }
    }
    if v["type"] == "array" {
        assert_schema_is_strict(&v["items"]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bluepencil_core::Format;

    #[test]
    fn numbered_lines_carry_file_and_one_based_line() {
        let doc = Document::parse("a.md", "first\nsecond", Format::Markdown);
        assert_eq!(numbered(&[doc]).unwrap(), "a.md:1| first\na.md:2| second\n\n");
    }

    #[test]
    fn quote_prefers_the_occurrence_nearest_the_cited_line() {
        let doc = Document::parse("a.md", "grey eyes\n\n\n\ngrey eyes\n", Format::Markdown);
        let near_top = locate_quote(&doc, "grey eyes", 1).unwrap();
        let near_bottom = locate_quote(&doc, "grey eyes", 5).unwrap();
        assert_eq!(doc.position(near_top.start).line, 1);
        assert_eq!(doc.position(near_bottom.start).line, 5);
        assert!(locate_quote(&doc, "blue eyes", 1).is_none());
    }
}
