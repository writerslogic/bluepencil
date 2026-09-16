use serde::Serialize;

use crate::document::Sentence;
use crate::text::syllables;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Readability {
    pub flesch_reading_ease: f64,
    pub flesch_kincaid_grade: f64,
    pub gunning_fog: f64,
    pub coleman_liau: f64,
    pub automated_readability: f64,
    pub words: usize,
    pub sentences: usize,
}

pub fn score<'a>(sentences: impl Iterator<Item = &'a Sentence>, source: &str) -> Readability {
    let (mut words, mut sents, mut syll, mut complex, mut letters) = (0usize, 0usize, 0usize, 0usize, 0usize);
    for s in sentences {
        sents += 1;
        for w in &s.words {
            let text = w.span.slice(source);
            let n = syllables::count(text);
            words += 1;
            syll += n;
            letters += text.chars().filter(|c| c.is_alphanumeric()).count();
            if n >= 3 && !text.contains('-') && !text.starts_with(char::is_uppercase) {
                complex += 1;
            }
        }
    }
    if words == 0 || sents == 0 {
        return Readability::default();
    }
    let (w, s) = (words as f64, sents as f64);
    let wps = w / s;
    let spw = syll as f64 / w;
    let l = letters as f64 / w * 100.0;
    let sp = s / w * 100.0;
    Readability {
        flesch_reading_ease: 206.835 - 1.015 * wps - 84.6 * spw,
        flesch_kincaid_grade: 0.39 * wps + 11.8 * spw - 15.59,
        gunning_fog: 0.4 * (wps + 100.0 * complex as f64 / w),
        coleman_liau: 0.0588 * l - 0.296 * sp - 15.8,
        automated_readability: 4.71 * (letters as f64 / w) + 0.5 * wps - 21.43,
        words,
        sentences: sents,
    }
}
