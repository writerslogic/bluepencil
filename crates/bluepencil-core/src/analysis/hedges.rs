use super::{Finding, phrase_findings};
use crate::document::Document;
use crate::lexicon::Lexicons;

pub fn find(doc: &Document, lex: &Lexicons) -> Vec<Finding> {
    phrase_findings(doc, &lex.hedges, "hedge")
}
