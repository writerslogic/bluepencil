pub mod embedded;
pub mod user;

use std::collections::{HashMap, HashSet};

use crate::document::Document;
use crate::span::Span;
use crate::text::normalize::fold;

#[derive(Debug, Clone, Default)]
pub struct WordSet(HashSet<String>);

impl WordSet {
    pub fn from_list<'a>(items: impl IntoIterator<Item = &'a str>) -> Self {
        Self(items.into_iter().map(fold).collect())
    }

    pub fn contains(&self, word: &str) -> bool {
        self.0.contains(word)
    }

    pub fn extend<'a>(&mut self, items: impl IntoIterator<Item = &'a str>) {
        self.0.extend(items.into_iter().map(fold));
    }

    pub fn remove_all<'a>(&mut self, items: impl IntoIterator<Item = &'a str>) {
        for i in items {
            self.0.remove(&fold(i));
        }
    }
}

/// Multi-word phrases matched against the word stream of each sentence.
#[derive(Debug, Clone, Default)]
pub struct PhraseSet {
    by_first: HashMap<String, Vec<Vec<String>>>,
}

#[derive(Debug, Clone)]
pub struct PhraseMatch {
    pub phrase: String,
    pub span: Span,
}

impl PhraseSet {
    pub fn from_list<'a>(items: impl IntoIterator<Item = &'a str>) -> Self {
        let mut set = Self::default();
        set.extend(items);
        set
    }

    pub fn extend<'a>(&mut self, items: impl IntoIterator<Item = &'a str>) {
        for item in items {
            let words: Vec<String> = fold(item).split_whitespace().map(str::to_string).collect();
            if let Some(first) = words.first().cloned() {
                let list = self.by_first.entry(first).or_default();
                list.push(words);
                list.sort_by_key(|w| std::cmp::Reverse(w.len()));
            }
        }
    }

    pub fn remove_all<'a>(&mut self, items: impl IntoIterator<Item = &'a str>) {
        for item in items {
            let words: Vec<String> = fold(item).split_whitespace().map(str::to_string).collect();
            for list in self.by_first.values_mut() {
                list.retain(|p| *p != words);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.by_first.values().all(Vec::is_empty)
    }

    pub fn find(&self, doc: &Document) -> Vec<PhraseMatch> {
        let mut out = Vec::new();
        for sentence in doc.sentences() {
            let words = &sentence.words;
            let mut i = 0;
            while i < words.len() {
                let hit = self.by_first.get(&words[i].lower).and_then(|cands| {
                    cands
                        .iter()
                        .find(|p| p.len() <= words.len() - i && p.iter().zip(&words[i..]).all(|(a, b)| *a == b.lower))
                });
                match hit {
                    Some(p) => {
                        let span = Span::new(words[i].span.start, words[i + p.len() - 1].span.end);
                        out.push(PhraseMatch { phrase: p.join(" "), span });
                        i += p.len();
                    }
                    None => i += 1,
                }
            }
        }
        out
    }
}

/// Built-in word lists merged with the user's additions and removals.
#[derive(Debug, Clone)]
pub struct Lexicons {
    pub stopwords: WordSet,
    pub not_adverbs: WordSet,
    pub filter: PhraseSet,
    pub hedges: PhraseSet,
    pub cliches: PhraseSet,
    pub tics: PhraseSet,
    pub irregular_participles: WordSet,
}

impl Default for Lexicons {
    fn default() -> Self {
        use embedded::*;
        Self {
            stopwords: WordSet::from_list(lines(STOPWORDS)),
            not_adverbs: WordSet::from_list(lines(NOT_ADVERBS)),
            filter: PhraseSet::from_list(lines(FILTER_WORDS)),
            hedges: PhraseSet::from_list(lines(HEDGES)),
            cliches: PhraseSet::from_list(lines(CLICHES)),
            tics: PhraseSet::default(),
            irregular_participles: WordSet::from_list(lines(IRREGULAR_PARTICIPLES)),
        }
    }
}
