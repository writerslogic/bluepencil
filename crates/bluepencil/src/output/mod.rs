pub mod chart;
pub mod html;
pub mod human;
pub mod json;
pub mod table;

use bluepencil_core::{Document, Finding, Span};
use serde::Serialize;

#[derive(Serialize)]
pub struct Located {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub rule: &'static str,
    pub message: String,
    pub text: String,
}

pub fn locate(doc: &Document, f: &Finding) -> Located {
    let start = doc.position(f.span.start);
    let end = doc.position(f.span.end);
    Located {
        file: doc.name.clone(),
        line: start.line,
        column: start.column,
        end_line: end.line,
        end_column: end.column,
        rule: f.rule,
        message: f.message.clone(),
        text: doc.text(f.span).to_string(),
    }
}

pub fn location(doc: &Document, span: Span) -> String {
    let p = doc.position(span.start);
    format!("{}:{}:{}", doc.name, p.line, p.column)
}

/// Single-line excerpt of `span`, truncated in the middle when long.
pub fn excerpt(doc: &Document, span: Span, max: usize) -> String {
    let flat = span.slice(&doc.prose).split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = flat.chars().collect();
    if chars.len() <= max {
        return flat;
    }
    let half = max.saturating_sub(3) / 2;
    let head: String = chars[..half].iter().collect();
    let tail: String = chars[chars.len() - half..].iter().collect();
    format!("{head} … {tail}")
}
