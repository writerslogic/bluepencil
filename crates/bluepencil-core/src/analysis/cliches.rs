use super::{Finding, phrase_findings};
use crate::document::Document;
use crate::lexicon::Lexicons;

pub fn find(doc: &Document, lex: &Lexicons) -> Vec<Finding> {
    phrase_findings(doc, &lex.cliches, "cliche")
}

#[cfg(test)]
mod tests {
    use super::find;
    use crate::{Document, Format, Lexicons};

    #[test]
    fn flags_the_british_spelling_the_same_as_the_american_one() {
        let lex = Lexicons::default();
        let doc = Document::parse(
            "test.md".to_string(),
            "Every fiber of her being screamed. Every fibre of his being ached.".to_string(),
            Format::Markdown,
        );
        let findings = find(&doc, &lex);
        assert!(findings.iter().any(|f| f.message == "every fiber of her being"));
        assert!(findings.iter().any(|f| f.message == "every fibre of his being"));
    }
}
