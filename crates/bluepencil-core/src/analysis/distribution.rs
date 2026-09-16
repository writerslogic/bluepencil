use serde::Serialize;

use crate::document::Document;
use crate::stats::Summary;
use crate::stats::histogram::{Bucket, buckets};

#[derive(Debug, Clone, Serialize)]
pub struct Distribution {
    pub summary: Summary,
    pub buckets: Vec<Bucket>,
}

pub fn sentence_lengths(doc: &Document) -> Vec<usize> {
    doc.sentences().map(|s| s.words.len()).collect()
}

pub fn paragraph_lengths(doc: &Document) -> Vec<usize> {
    doc.paragraphs.iter().map(|p| p.word_count()).collect()
}

pub fn paragraph_sentence_counts(doc: &Document) -> Vec<usize> {
    doc.paragraphs.iter().map(|p| p.sentences.len()).collect()
}

pub fn distribution(values: &[usize], width: usize, cap: usize) -> Distribution {
    Distribution { summary: Summary::of(values), buckets: buckets(values, width, cap) }
}
