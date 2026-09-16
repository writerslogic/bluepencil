use serde::Serialize;
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Insert,
    Delete,
    Equal,
}

#[derive(Debug, Clone, Serialize)]
pub struct WordChange {
    pub tag: ChangeKind,
    pub word: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WordDiff {
    pub changes: Vec<WordChange>,
    pub added: usize,
    pub removed: usize,
}

/// Word-level diff between two texts, tokenizing on whitespace so punctuation stays
/// attached to the word it follows -- the same convention the classic Unix `wdiff`
/// tool uses, and the one a writer comparing two drafts actually wants to see.
pub fn word_diff(old: &str, new: &str) -> WordDiff {
    let old_tokens: Vec<&str> = old.split_whitespace().collect();
    let new_tokens: Vec<&str> = new.split_whitespace().collect();
    let diff = TextDiff::from_slices(&old_tokens, &new_tokens);

    let mut changes = Vec::new();
    let mut added = 0;
    let mut removed = 0;
    for change in diff.iter_all_changes() {
        let tag = match change.tag() {
            ChangeTag::Delete => {
                removed += 1;
                ChangeKind::Delete
            }
            ChangeTag::Insert => {
                added += 1;
                ChangeKind::Insert
            }
            ChangeTag::Equal => ChangeKind::Equal,
        };
        changes.push(WordChange { tag, word: change.value().to_string() });
    }
    WordDiff { changes, added, removed }
}
