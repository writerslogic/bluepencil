use serde::Serialize;

use crate::document::Document;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct DialogueRatio {
    pub dialogue_words: usize,
    pub narration_words: usize,
    pub ratio: f64,
}

impl DialogueRatio {
    pub fn new(dialogue_words: usize, narration_words: usize) -> Self {
        let total = dialogue_words + narration_words;
        let ratio = if total == 0 { 0.0 } else { dialogue_words as f64 / total as f64 };
        Self { dialogue_words, narration_words, ratio }
    }

    pub fn merge(self, o: Self) -> Self {
        Self::new(self.dialogue_words + o.dialogue_words, self.narration_words + o.narration_words)
    }
}

pub fn ratio(doc: &Document) -> DialogueRatio {
    let d = doc.words().filter(|w| doc.in_dialogue(w.span)).count();
    DialogueRatio::new(d, doc.word_count() - d)
}

pub fn by_section(doc: &Document) -> Vec<(String, DialogueRatio)> {
    let mut out: Vec<(Option<usize>, DialogueRatio)> = Vec::new();
    for p in &doc.paragraphs {
        let d = p.sentences.iter().flat_map(|s| &s.words).filter(|w| doc.in_dialogue(w.span)).count();
        let r = DialogueRatio::new(d, p.word_count() - d);
        match out.last_mut() {
            Some((sec, acc)) if *sec == p.section => *acc = acc.merge(r),
            _ => out.push((p.section, r)),
        }
    }
    out.into_iter().map(|(sec, r)| (doc.section_title(sec).to_string(), r)).collect()
}
