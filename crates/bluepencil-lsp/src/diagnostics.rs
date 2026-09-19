//! Maps `bluepencil-core` analysis output to LSP diagnostics.

use bluepencil_core::analysis::echoes::Echo;
use bluepencil_core::{Document, Finding, Span};
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Position, Range, Url,
};

const SOURCE: &str = "bluepencil";

/// Converts a byte-offset `Span` into an LSP `Range`.
///
/// `bluepencil_core::Position` is 1-indexed for both line and column; LSP
/// positions are 0-indexed for both, so each component is shifted down by one.
/// `bluepencil_core`'s column counts Unicode scalar values, while LSP's
/// `character` counts UTF-16 code units; these agree for BMP text (including
/// curly quotes and em-dashes) but diverge on astral characters (e.g. emoji),
/// which is not handled here.
fn range(doc: &Document, span: Span) -> Range {
    let start = doc.position(span.start);
    let end = doc.position(span.end);
    Range {
        start: Position { line: (start.line - 1) as u32, character: (start.column - 1) as u32 },
        end: Position { line: (end.line - 1) as u32, character: (end.column - 1) as u32 },
    }
}

/// Maps one `Finding` (adverbs, clichés, hedges, filter words, passive voice, ...)
/// to a diagnostic. These are style suggestions, not errors, so severity is
/// `INFORMATION` and the rule name is carried in `code` for client-side filtering.
pub fn finding_to_diagnostic(doc: &Document, finding: &Finding) -> Diagnostic {
    Diagnostic {
        range: range(doc, finding.span),
        severity: Some(DiagnosticSeverity::INFORMATION),
        code: Some(NumberOrString::String(finding.rule.to_string())),
        code_description: None,
        source: Some(SOURCE.to_string()),
        message: finding.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

pub fn findings_to_diagnostics(doc: &Document, findings: &[Finding]) -> Vec<Diagnostic> {
    findings.iter().map(|f| finding_to_diagnostic(doc, f)).collect()
}

/// Maps one echo (a content word reappearing within a window of its previous
/// use) to a diagnostic on the *second* occurrence, with the first occurrence
/// linked as related information so the client can jump to it.
pub fn echo_to_diagnostic(doc: &Document, uri: &Url, echo: &Echo) -> Diagnostic {
    let first_range = range(doc, echo.first);
    Diagnostic {
        range: range(doc, echo.second),
        severity: Some(DiagnosticSeverity::INFORMATION),
        code: Some(NumberOrString::String("echo".to_string())),
        code_description: None,
        source: Some(SOURCE.to_string()),
        message: format!("echo: \"{}\" repeated within {} words", echo.word, echo.distance),
        related_information: Some(vec![DiagnosticRelatedInformation {
            location: Location { uri: uri.clone(), range: first_range },
            message: format!("previous use of \"{}\"", echo.word),
        }]),
        tags: None,
        data: None,
    }
}

pub fn echoes_to_diagnostics(doc: &Document, uri: &Url, echoes: &[Echo]) -> Vec<Diagnostic> {
    echoes.iter().map(|e| echo_to_diagnostic(doc, uri, e)).collect()
}

#[cfg(test)]
mod tests {
    use bluepencil_core::Format;

    use super::*;

    #[test]
    fn maps_finding_span_to_zero_indexed_range() {
        let source = "First line.\nSecond line has adverbly words.\n";
        let doc = Document::parse("test", source, Format::Plain);

        // "adverbly" is on the second line (1-indexed line 2, 1-indexed
        // column 17): "Second line has " is 16 bytes into line 2, so the
        // word starts at column 17, giving 0-indexed LSP character 16.
        let start = source.find("adverbly").unwrap();
        let span = Span::new(start, start + "adverbly".len());
        let finding = Finding { rule: "adverb", message: "adverbly".to_string(), span };

        let diagnostic = finding_to_diagnostic(&doc, &finding);

        assert_eq!(diagnostic.range.start, Position { line: 1, character: 16 });
        assert_eq!(diagnostic.range.end, Position { line: 1, character: 24 });
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::INFORMATION));
        assert_eq!(diagnostic.source, Some(SOURCE.to_string()));
        assert_eq!(diagnostic.code, Some(NumberOrString::String("adverb".to_string())));
        assert_eq!(diagnostic.message, "adverbly");
    }
}
