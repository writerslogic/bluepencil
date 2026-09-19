use schemars::JsonSchema;
use serde::Serialize;

use super::echoes::proper_nouns;
use crate::document::{Document, Word};
use crate::lexicon::Lexicons;
use crate::span::Span;

/// How many words on either side of a line of dialogue count as "attached" to it.
const WINDOW: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TagKind {
    /// `said`/`asked` and their inflections.
    Plain,
    /// A more expressive verb standing in for `said` ("exclaimed", "snapped").
    Showy,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Attribution {
    pub dialogue: Span,
    pub tag: Option<Span>,
    pub kind: Option<TagKind>,
    /// Whether the tag is qualified by an adverb ("she said softly").
    pub adverb: bool,
    pub speaker: Option<String>,
}

/// Attributes each line of dialogue to a tag verb and speaker name found within
/// [`WINDOW`] words before or after the quote, in the same paragraph. No POS tagger and
/// no coreference: a speaker is whichever proper noun sits nearest the tag verb, so a run
/// of unattributed dialogue after a single "she said" is reported as unattributed rather
/// than inherited from the previous line, the same tradeoff [`super::passive::find`] makes
/// for lacking a real parser.
pub fn attributions(doc: &Document, lex: &Lexicons) -> Vec<Attribution> {
    let names = proper_nouns(doc);
    let mut out = Vec::new();
    for p in &doc.paragraphs {
        let words: Vec<&Word> = p.sentences.iter().flat_map(|s| &s.words).collect();
        for dlg in &doc.dialogue {
            if p.span.contains(*dlg) {
                out.push(attribution_for(*dlg, &words, lex, &names));
            }
        }
    }
    out
}

fn attribution_for(
    dialogue: Span,
    words: &[&Word],
    lex: &Lexicons,
    names: &std::collections::HashSet<&str>,
) -> Attribution {
    let after: Vec<usize> = (0..words.len()).filter(|&i| words[i].span.start >= dialogue.end).collect();
    let before: Vec<usize> = (0..words.len()).filter(|&i| words[i].span.end <= dialogue.start).rev().collect();

    for candidates in [&after, &before] {
        for (rank, &i) in candidates.iter().take(WINDOW).enumerate() {
            let w = words[i];
            let kind = if lex.dialogue_tags.contains(&w.lower) {
                Some(TagKind::Plain)
            } else if lex.said_bookisms.contains(&w.lower) {
                Some(TagKind::Showy)
            } else {
                None
            };
            let Some(kind) = kind else { continue };

            let adverb = candidates
                .get(rank + 1)
                .map(|&j| words[j])
                .is_some_and(|next| next.lower.ends_with("ly") && !lex.not_adverbs.contains(&next.lower));

            let speaker = candidates
                .iter()
                .take(WINDOW)
                .map(|&j| words[j])
                .find(|w2| names.contains(w2.lower.as_str()))
                .map(|w2| w2.lower.clone());

            return Attribution { dialogue, tag: Some(w.span), kind: Some(kind), adverb, speaker };
        }
    }
    Attribution { dialogue, tag: None, kind: None, adverb: false, speaker: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Format;

    fn doc(text: &str) -> Document {
        Document::parse("test.md".to_string(), text.to_string(), Format::Markdown)
    }

    #[test]
    fn finds_a_plain_tag_after_the_quote() {
        let lex = Lexicons::default();
        let d = doc("\"Hello,\" said Mara.");
        let a = attributions(&d, &lex);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].kind, Some(TagKind::Plain));
        assert_eq!(a[0].speaker.as_deref(), Some("mara"));
        assert!(!a[0].adverb);
    }

    #[test]
    fn finds_a_showy_tag_before_the_quote() {
        let lex = Lexicons::default();
        let d = doc("Elsewhere, Mara laughed. Mara snapped, \"Enough.\"");
        let a = attributions(&d, &lex);
        assert_eq!(a[0].kind, Some(TagKind::Showy));
        assert_eq!(a[0].speaker.as_deref(), Some("mara"));
    }

    #[test]
    fn flags_an_adverb_modified_plain_tag() {
        let lex = Lexicons::default();
        let d = doc("\"Hello,\" she said softly.");
        let a = attributions(&d, &lex);
        assert_eq!(a[0].kind, Some(TagKind::Plain));
        assert!(a[0].adverb);
    }

    #[test]
    fn reports_no_tag_when_none_is_nearby() {
        let lex = Lexicons::default();
        let d = doc("\"Hello.\" The room was silent for a long while afterward.");
        let a = attributions(&d, &lex);
        assert!(a[0].tag.is_none());
        assert!(a[0].speaker.is_none());
    }
}
