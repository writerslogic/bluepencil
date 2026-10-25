use bluepencil_core::analysis::{
    adverbs, cliches, counts::Counts, echoes, filter_words, hedges, passive, rhythm, starters,
};
use bluepencil_core::lexicon::WordSet;
use bluepencil_core::{Document, Finding, Format, Lexicons};
use serde::Serialize;
use wasm_bindgen::prelude::*;

const MAX_INPUT_BYTES: usize = 100_000;
const MAX_FINDINGS: usize = 150;

#[derive(Serialize)]
struct FindingView {
    rule: String,
    message: String,
    excerpt: String,
    line: usize,
    column: usize,
    start: usize,
    end: usize,
}

#[derive(Serialize)]
struct Analysis {
    words: usize,
    sentences: usize,
    paragraphs: usize,
    findings: Vec<FindingView>,
    truncated: bool,
}

fn view(doc: &Document, finding: &Finding) -> FindingView {
    let pos = doc.position(finding.span.start);
    let sentence = doc
        .sentences()
        .find(|s| s.span.start <= finding.span.start && finding.span.start < s.span.end)
        .map_or(finding.span, |s| s.span);
    FindingView {
        rule: finding.rule.to_string(),
        message: finding.message.clone(),
        excerpt: sentence.slice(&doc.source).split_whitespace().collect::<Vec<_>>().join(" "),
        line: pos.line,
        column: pos.column,
        start: doc.source[..finding.span.start].encode_utf16().count(),
        end: doc.source[..finding.span.end].encode_utf16().count(),
    }
}

#[cfg(test)]
fn analyze_text(source: &str, format: Format) -> Result<Analysis, String> {
    analyze_text_with_limit(source, format, MAX_INPUT_BYTES)
}

fn analyze_text_with_limit(source: &str, format: Format, max_bytes: usize) -> Result<Analysis, String> {
    if source.len() > max_bytes {
        return Err(format!(
            "This document exceeds the {max_bytes}-byte analysis limit; use the CLI for full manuscripts."
        ));
    }
    let doc = Document::parse("Your sample", source, format);
    let lex = Lexicons::default();
    let mut findings = Vec::new();
    for echo in echoes::echoes(&doc, &lex, 50, 4, &WordSet::default(), false) {
        findings.push(Finding {
            rule: "echo",
            message: format!("\"{}\" repeats {} words later", echo.word, echo.distance),
            span: echo.second,
        });
    }
    findings.extend(cliches::find(&doc, &lex));
    findings.extend(starters::starters(&doc, 3).findings);
    findings.extend(rhythm::rhythm(&doc, 4, 3, 40).findings);
    findings.extend(passive::find(&doc, &lex));
    findings.extend(filter_words::find(&doc, &lex));
    findings.extend(hedges::find(&doc, &lex));
    findings.extend(adverbs::find(&doc, &lex));
    findings.sort_by_key(|f| (f.span.start, f.span.end));
    let truncated = findings.len() > MAX_FINDINGS;
    let findings = findings.iter().take(MAX_FINDINGS).map(|f| view(&doc, f)).collect();
    let counts = Counts::of(&doc);
    Ok(Analysis {
        words: counts.words,
        sentences: counts.sentences,
        paragraphs: counts.paragraphs,
        findings,
        truncated,
    })
}

#[wasm_bindgen]
pub fn analyze(source: &str, format: &str) -> Result<String, JsValue> {
    analyze_with_limit(source, format, MAX_INPUT_BYTES)
}

#[wasm_bindgen]
pub fn analyze_full(source: &str, format: &str) -> Result<String, JsValue> {
    analyze_with_limit(source, format, 1024 * 1024)
}

fn analyze_with_limit(source: &str, format: &str, max_bytes: usize) -> Result<String, JsValue> {
    let format = match format {
        "markdown" => Format::Markdown,
        "fountain" => Format::Fountain,
        "plain" => Format::Plain,
        _ => return Err(JsValue::from_str("Choose Markdown, Fountain, or plain text.")),
    };
    let result = analyze_text_with_limit(source, format, max_bytes).map_err(|message| JsValue::from_str(&message))?;
    serde_json::to_string(&result).map_err(|_| JsValue::from_str("Could not serialize the analysis."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_source_locations_after_unicode() {
        let source = "🌊 The gravel was cold. The gravel was very cold.";
        let result = analyze_text(source, Format::Plain).unwrap();
        let echo = result.findings.iter().find(|f| f.rule == "echo" && f.message.contains("gravel")).unwrap();
        assert_eq!(echo.line, 1);
        assert_eq!(&source["🌊 The gravel was cold. The ".len()..][..6], "gravel");
        assert_eq!(echo.start, 28);
    }

    #[test]
    fn refuses_oversized_input() {
        assert!(analyze_text(&"a".repeat(MAX_INPUT_BYTES - 1), Format::Plain).is_ok());
        assert!(analyze_text(&"a".repeat(MAX_INPUT_BYTES), Format::Plain).is_ok());
        assert!(analyze_text(&"a".repeat(MAX_INPUT_BYTES + 1), Format::Plain).is_err());
    }
}
