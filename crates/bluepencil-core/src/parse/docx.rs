//! Extracts body text from a `.docx` (Office Open XML) archive as synthetic Markdown.
//!
//! A `.docx` is a zip of XML, so it cannot be masked in place the way the other parsers
//! mask their source text (see `Mask` in `parse/mod.rs`): there is no byte-aligned
//! relationship between the archive bytes and the prose they contain. Instead this module
//! reconstructs a plain Markdown-flavored string from `word/document.xml` — each paragraph's
//! run text joined, heading styles turned into `#` prefixes — and callers feed that string
//! through the existing `Format::Markdown` parser. Spans reported for a `.docx` document
//! therefore point into this synthetic text, not into the original file; line/column
//! locations are approximate, not a match for the reader's word processor.
//!
//! Known limitations (v1): tables, images, headers/footers, comments, and tracked changes
//! are skipped entirely; field codes (`w:instrText`) are skipped rather than resolved to
//! their cached display text; list items are emitted as flat `- ` bullets without
//! distinguishing ordered from unordered lists or preserving nesting.

use std::io::{Cursor, Read};

use anyhow::{Context as _, Result};
use quick_xml::Reader;
use quick_xml::events::Event;

/// Caps the decompressed size of `word/document.xml` read from an untrusted archive.
const MAX_DOCUMENT_XML_BYTES: u64 = 64 * 1024 * 1024;

pub fn extract(bytes: &[u8]) -> Result<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).context("not a valid .docx (zip) archive")?;
    let mut entry = archive.by_name("word/document.xml").context("missing word/document.xml in .docx archive")?;
    let mut xml = String::new();
    entry.by_ref().take(MAX_DOCUMENT_XML_BYTES).read_to_string(&mut xml).context("reading word/document.xml")?;
    render(&xml)
}

#[derive(Default)]
struct Paragraph {
    text: String,
    style: Option<String>,
    is_list_item: bool,
}

fn render(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);

    let mut paragraphs: Vec<String> = Vec::new();
    let mut table_depth = 0u32;
    let mut para: Option<Paragraph> = None;
    let mut in_run_text = false;
    let mut skip_text = false;

    loop {
        let event = reader.read_event().context("parsing word/document.xml")?;
        match event {
            Event::Eof => break,
            Event::Start(e) => {
                match e.local_name().as_ref() {
                    "tbl" => table_depth += 1,
                    "t" => in_run_text = true,
                    "instrText" | "delText" => skip_text = true,
                    "p" if table_depth == 0 => para = Some(Paragraph::default()),
                    _ => {}
                }
                apply_paragraph_markup(&e, para.as_mut());
            }
            Event::Empty(e) => apply_paragraph_markup(&e, para.as_mut()),
            Event::Text(t) => {
                if in_run_text
                    && !skip_text
                    && table_depth == 0
                    && let Some(p) = para.as_mut()
                {
                    p.text.push_str(t.as_ref());
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                "tbl" => table_depth = table_depth.saturating_sub(1),
                "p" => {
                    if let Some(p) = para.take() {
                        paragraphs.push(format_paragraph(p));
                    }
                }
                "t" => in_run_text = false,
                "instrText" | "delText" => skip_text = false,
                _ => {}
            },
            _ => {}
        }
    }

    Ok(paragraphs.into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join("\n\n"))
}

/// Handles the paragraph-formatting tags shared between `Event::Start` and `Event::Empty`
/// (real documents emit self-closing `<w:pStyle .../>`, `<w:tab/>`, and `<w:br/>` tags).
fn apply_paragraph_markup(e: &quick_xml::events::BytesStart, para: Option<&mut Paragraph>) {
    let Some(p) = para else { return };
    match e.local_name().as_ref() {
        "pStyle" => {
            if let Some(val) = attr(e, "val") {
                p.style = Some(val);
            }
        }
        "numPr" => p.is_list_item = true,
        "tab" | "br" | "cr" => p.text.push(' '),
        _ => {}
    }
}

fn attr(e: &quick_xml::events::BytesStart, local_key: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == local_key)
        .and_then(|a| a.normalized_value(quick_xml::XmlVersion::Implicit1_0).ok().map(|v| v.into_owned()))
}

fn heading_prefix(style: &str) -> Option<&'static str> {
    let normalized: String =
        style.chars().filter(|c| !c.is_whitespace() && *c != '_' && *c != '-').flat_map(|c| c.to_lowercase()).collect();
    match normalized.as_str() {
        "title" => Some("#"),
        "heading1" => Some("#"),
        "heading2" => Some("##"),
        "heading3" => Some("###"),
        "heading4" => Some("####"),
        "heading5" => Some("#####"),
        "heading6" => Some("######"),
        _ => None,
    }
}

fn format_paragraph(p: Paragraph) -> String {
    let text = p.text.trim_end_matches(['\r', '\n']);
    if text.is_empty() {
        return String::new();
    }
    if let Some(prefix) = p.style.as_deref().and_then(heading_prefix) {
        return format!("{prefix} {text}");
    }
    if p.is_list_item {
        return format!("- {text}");
    }
    // A body paragraph that happens to start with `#` would otherwise be mistaken for an
    // ATX heading once this synthetic text is re-parsed as Markdown.
    if let Some(rest) = text.strip_prefix('#') {
        return format!("\\#{rest}");
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::*;

    fn build_docx(document_xml: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buf));
            let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("word/document.xml", options).unwrap();
            zip.write_all(document_xml.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        buf
    }

    const NS: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main""#;

    fn wrap_body(body: &str) -> String {
        format!(r#"<?xml version="1.0" encoding="UTF-8"?><w:document {NS}><w:body>{body}</w:body></w:document>"#)
    }

    #[test]
    fn extracts_plain_paragraphs_separated_by_blank_lines() {
        let xml = wrap_body(
            r#"<w:p><w:r><w:t>First paragraph.</w:t></w:r></w:p>
               <w:p><w:r><w:t>Second paragraph.</w:t></w:r></w:p>"#,
        );
        let bytes = build_docx(&xml);
        let text = extract(&bytes).unwrap();
        assert_eq!(text, "First paragraph.\n\nSecond paragraph.");
    }

    #[test]
    fn heading1_style_becomes_a_single_hash() {
        let xml = wrap_body(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Chapter One</w:t></w:r></w:p>
               <w:p><w:r><w:t>Body text.</w:t></w:r></w:p>"#,
        );
        let bytes = build_docx(&xml);
        let text = extract(&bytes).unwrap();
        assert_eq!(text, "# Chapter One\n\nBody text.");
    }

    #[test]
    fn heading_levels_nest_by_number() {
        let xml = wrap_body(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>One</w:t></w:r></w:p>
               <w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Two</w:t></w:r></w:p>
               <w:p><w:pPr><w:pStyle w:val="Heading3"/></w:pPr><w:r><w:t>Three</w:t></w:r></w:p>"#,
        );
        let bytes = build_docx(&xml);
        let text = extract(&bytes).unwrap();
        assert_eq!(text, "# One\n\n## Two\n\n### Three");
    }

    #[test]
    fn multiple_runs_in_one_paragraph_are_concatenated() {
        let xml = wrap_body(r#"<w:p><w:r><w:t>Hello </w:t></w:r><w:r><w:t>world.</w:t></w:r></w:p>"#);
        let bytes = build_docx(&xml);
        let text = extract(&bytes).unwrap();
        assert_eq!(text, "Hello world.");
    }

    #[test]
    fn table_paragraphs_are_skipped() {
        let xml = wrap_body(
            r#"<w:p><w:r><w:t>Before.</w:t></w:r></w:p>
               <w:tbl><w:tr><w:tc><w:p><w:r><w:t>cell text</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
               <w:p><w:r><w:t>After.</w:t></w:r></w:p>"#,
        );
        let bytes = build_docx(&xml);
        let text = extract(&bytes).unwrap();
        assert_eq!(text, "Before.\n\nAfter.");
    }

    #[test]
    fn numbered_paragraph_becomes_a_flat_bullet() {
        let xml = wrap_body(
            r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>Item one</w:t></w:r></w:p>"#,
        );
        let bytes = build_docx(&xml);
        let text = extract(&bytes).unwrap();
        assert_eq!(text, "- Item one");
    }

    #[test]
    fn leading_hash_in_body_text_is_escaped() {
        let xml = wrap_body(r#"<w:p><w:r><w:t>#1 bestseller</w:t></w:r></w:p>"#);
        let bytes = build_docx(&xml);
        let text = extract(&bytes).unwrap();
        assert_eq!(text, "\\#1 bestseller");
    }

    #[test]
    fn instr_text_field_codes_are_skipped() {
        let xml = wrap_body(
            r#"<w:p><w:r><w:instrText>HYPERLINK "https://example.com"</w:instrText></w:r><w:r><w:t>visible</w:t></w:r></w:p>"#,
        );
        let bytes = build_docx(&xml);
        let text = extract(&bytes).unwrap();
        assert_eq!(text, "visible");
    }

    #[test]
    fn missing_document_xml_errors() {
        let mut buf = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buf));
            zip.start_file("word/other.xml", SimpleFileOptions::default()).unwrap();
            zip.write_all(b"<x/>").unwrap();
            zip.finish().unwrap();
        }
        assert!(extract(&buf).is_err());
    }

    #[test]
    fn not_a_zip_errors() {
        assert!(extract(b"not a zip file").is_err());
    }
}
