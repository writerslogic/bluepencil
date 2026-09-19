use super::Finding;
use crate::document::Document;
use crate::lexicon::Lexicons;

/// A weak-verb-plus-noun phrase where a single stronger verb would do ("make a decision"
/// rather than "decide"), matched against `data/nominalizations.tsv`.
pub fn find(doc: &Document, lex: &Lexicons) -> Vec<Finding> {
    lex.nominalizations
        .find(doc)
        .into_iter()
        .map(|m| {
            let message = match lex.nominalizations.suggestion(&m.phrase) {
                Some(verb) => format!("{} -> {verb}", m.phrase),
                None => m.phrase,
            };
            Finding { rule: "nominal", message, span: m.span }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::find;
    use crate::{Document, Format, Lexicons};

    #[test]
    fn flags_a_nominalization_with_its_suggested_verb() {
        let lex = Lexicons::default();
        let doc =
            Document::parse("test.md".to_string(), "She had to make a decision by noon.".to_string(), Format::Markdown);
        let findings = find(&doc, &lex);
        assert!(findings.iter().any(|f| f.message == "make a decision -> decide"));
    }

    #[test]
    fn does_not_flag_unrelated_prose() {
        let lex = Lexicons::default();
        let doc = Document::parse("test.md".to_string(), "She decided by noon.".to_string(), Format::Markdown);
        assert!(find(&doc, &lex).is_empty());
    }
}
