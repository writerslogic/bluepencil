use std::collections::HashSet;
use std::ops::Range;

use bluepencil_core::{Document, Finding, Span};
use serde::Serialize;

use super::SourceIndex;

const MAX_POINTS: usize = 8192;

#[derive(Serialize)]
pub struct ChartPoint {
    from: usize,
    to: usize,
    first_word: usize,
    last_word: usize,
    first_sentence: usize,
    last_sentence: usize,
    first_paragraph: usize,
    last_paragraph: usize,
    section: Option<usize>,
    sentence_length: f64,
    paragraph_length: f64,
    dialogue_share: f64,
    finding_density: f64,
    sentence_variation: f64,
}

#[derive(Serialize)]
pub struct ChartSection {
    id: usize,
    title: String,
    from: usize,
    to: usize,
}

#[derive(Serialize)]
pub struct Chart {
    pub(super) points: Vec<ChartPoint>,
    pub(super) words: usize,
    sections: Vec<ChartSection>,
    resolution: &'static str,
    target: usize,
    from: usize,
    to: usize,
}

struct MetricSentence {
    span: Span,
    words: usize,
    first_word: usize,
    number: usize,
    paragraph: usize,
    paragraph_words: usize,
    dialogue_words: usize,
    findings: usize,
    variation: f64,
    section: Option<usize>,
}

struct Section {
    title: String,
    span: Span,
    sentences: Range<usize>,
}

pub(super) struct ChartData {
    sentences: Vec<MetricSentence>,
    sections: Vec<Section>,
}

impl ChartData {
    pub(super) fn new(doc: &Document, findings: &[Finding]) -> Self {
        let headings = chapter_headings(doc);
        let mut sentences = Vec::new();
        let mut first_word = 1;
        let mut number = 0;
        for (paragraph, block) in doc.paragraphs.iter().enumerate() {
            let paragraph_words = block.word_count();
            for sentence in &block.sentences {
                number += 1;
                let words = sentence.words.len();
                let heading = headings.partition_point(|(_, span)| span.end < sentence.span.end);
                let is_heading = headings.get(heading).is_some_and(|(_, span)| span.contains(sentence.span));
                if words > 0 && !is_heading {
                    let first = findings.partition_point(|finding| finding.span.start < sentence.span.start);
                    let last = findings.partition_point(|finding| finding.span.start < sentence.span.end);
                    sentences.push(MetricSentence {
                        span: sentence.span,
                        words,
                        first_word,
                        number,
                        paragraph,
                        paragraph_words,
                        dialogue_words: sentence.words.iter().filter(|word| doc.in_dialogue(word.span)).count(),
                        findings: last - first,
                        variation: 0.0,
                        section: None,
                    });
                }
                first_word += words;
            }
        }
        let mut sections = Vec::new();
        if headings.len() >= 2 {
            if sentences.first().is_some_and(|sentence| sentence.span.start < headings[0].1.start) {
                let end = sentences.partition_point(|sentence| sentence.span.start < headings[0].1.start);
                sections.push(Section {
                    title: "Opening".into(),
                    span: Span::new(0, headings[0].1.start),
                    sentences: 0..end,
                });
            }
            for (index, (title, span)) in headings.iter().enumerate() {
                let end = headings.get(index + 1).map_or(doc.source.len(), |(_, next)| next.start);
                let first = sentences.partition_point(|sentence| sentence.span.start < span.start);
                let last = sentences.partition_point(|sentence| sentence.span.start < end);
                if first < last {
                    sections.push(Section {
                        title: title.clone(),
                        span: Span::new(span.start, end),
                        sentences: first..last,
                    });
                }
            }
        }
        for (index, section) in sections.iter().enumerate() {
            for sentence in &mut sentences[section.sentences.clone()] {
                sentence.section = Some(index);
            }
        }
        let mut section_start = 0;
        for index in 0..sentences.len() {
            if index > 0 && sentences[index].section != sentences[index - 1].section {
                section_start = index;
            }
            let window = &sentences[index.saturating_sub(4).max(section_start)..=index];
            let mean = window.iter().map(|sentence| sentence.words as f64).sum::<f64>() / window.len() as f64;
            let variance =
                window.iter().map(|sentence| (sentence.words as f64 - mean).powi(2)).sum::<f64>() / window.len() as f64;
            sentences[index].variation = variance.sqrt() * 100.0 / mean;
        }
        Self { sentences, sections }
    }

    pub(super) fn chapter_count(&self) -> usize {
        self.sections.len()
    }

    pub(super) fn view(
        &self,
        source: &str,
        source_index: &SourceIndex,
        span: Span,
        scope: &str,
        detail: &str,
    ) -> Result<Chart, &'static str> {
        if !matches!(scope, "whole" | "visible") {
            return Err("Choose the entire manuscript or the visible passage.");
        }
        let (resolution, target) = match (scope, detail) {
            ("whole", "auto") if self.sections.len() >= 2 && self.sections.len() <= MAX_POINTS / 8 => ("chapter", 8),
            ("whole", "auto") => ("overview", 256),
            ("whole", "chapter-4") => ("chapter", 4),
            ("whole", "chapter-8") => ("chapter", 8),
            ("whole", "chapter-16") => ("chapter", 16),
            ("whole", "overview-128") => ("overview", 128),
            ("whole", "overview-256") => ("overview", 256),
            ("whole", "overview-512") => ("overview", 512),
            ("visible", "auto" | "sentence") => ("sentence", 1),
            ("visible", "paragraph") => ("paragraph", 1),
            _ => return Err("Choose a detail setting for this chart view."),
        };
        if resolution == "chapter" && self.sections.len() < 2 {
            return Err("No chapter boundaries were found. Choose an overview detail setting.");
        }
        let first = self.sentences.partition_point(|sentence| sentence.span.end <= span.start);
        let last = self.sentences.partition_point(|sentence| sentence.span.start < span.end);
        let selected = if scope == "whole" { 0..self.sentences.len() } else { first.min(last)..last };
        let mut groups = Vec::new();
        match resolution {
            "chapter" => {
                for section in &self.sections {
                    sample(section.sentences.clone(), target, &mut groups)?;
                }
            }
            "overview" => sample(selected, target, &mut groups)?,
            "sentence" => {
                if selected.len() > MAX_POINTS {
                    return Err(too_detailed());
                }
                groups.extend(selected.map(|index| index..index + 1));
            }
            "paragraph" if !selected.is_empty() => {
                let first_paragraph = self.sentences[selected.start].paragraph;
                let last_paragraph = self.sentences[selected.end - 1].paragraph;
                let mut first = self.sentences.partition_point(|sentence| sentence.paragraph < first_paragraph);
                let end = self.sentences.partition_point(|sentence| sentence.paragraph <= last_paragraph);
                while first < end {
                    let paragraph = self.sentences[first].paragraph;
                    let last =
                        first + self.sentences[first..end].partition_point(|sentence| sentence.paragraph == paragraph);
                    if groups.len() == MAX_POINTS {
                        return Err(too_detailed());
                    }
                    groups.push(first..last);
                    first = last;
                }
            }
            _ => {}
        }
        let points: Vec<_> = groups.iter().map(|group| self.point(source, source_index, group.clone())).collect();
        let words = groups.iter().flat_map(|group| &self.sentences[group.clone()]).map(|sentence| sentence.words).sum();
        let from = if scope == "whole" { 0 } else { points.first().map_or(0, |point| point.from) };
        let to = if scope == "whole" { source_index.length } else { points.last().map_or(from, |point| point.to) };
        let shown_sections: Vec<_> = self
            .sections
            .iter()
            .enumerate()
            .filter(|(_, section)| scope == "whole" || (section.span.end > span.start && section.span.start < span.end))
            .take(MAX_POINTS + 1)
            .collect();
        if shown_sections.len() > MAX_POINTS {
            return Err("This overview has too many chapter boundaries. Zoom to the visible passage.");
        }
        let sections = shown_sections
            .into_iter()
            .map(|(id, section)| ChartSection {
                id,
                title: section.title.chars().take(120).collect(),
                from: source_index.position(source, section.span.start).utf16,
                to: source_index.position(source, section.span.end).utf16,
            })
            .collect();
        Ok(Chart { points, words, sections, resolution, target, from, to })
    }

    fn point(&self, source: &str, index: &SourceIndex, group: Range<usize>) -> ChartPoint {
        let bucket = &self.sentences[group];
        let first = &bucket[0];
        let last = &bucket[bucket.len() - 1];
        let mut paragraphs = HashSet::new();
        let (mut words, mut paragraph_words, mut dialogue, mut findings) = (0, 0, 0, 0);
        let mut variation = 0.0;
        for sentence in bucket {
            words += sentence.words;
            if paragraphs.insert(sentence.paragraph) {
                paragraph_words += sentence.paragraph_words;
            }
            dialogue += sentence.dialogue_words;
            findings += sentence.findings;
            variation += sentence.variation;
        }
        ChartPoint {
            from: index.position(source, first.span.start).utf16,
            to: index.position(source, last.span.end).utf16,
            first_word: first.first_word,
            last_word: last.first_word + last.words - 1,
            first_sentence: first.number,
            last_sentence: last.number,
            first_paragraph: first.paragraph + 1,
            last_paragraph: last.paragraph + 1,
            section: first.section,
            sentence_length: words as f64 / bucket.len() as f64,
            paragraph_length: paragraph_words as f64 / paragraphs.len() as f64,
            dialogue_share: dialogue as f64 * 100.0 / words as f64,
            finding_density: findings as f64 * 1000.0 / words as f64,
            sentence_variation: variation / bucket.len() as f64,
        }
    }
}

fn too_detailed() -> &'static str {
    "This view exceeds 8,192 samples. Choose a coarser overview or a smaller visible passage."
}

fn sample(range: Range<usize>, target: usize, groups: &mut Vec<Range<usize>>) -> Result<(), &'static str> {
    let count = range.len().min(target);
    if groups.len() + count > MAX_POINTS {
        return Err(too_detailed());
    }
    if count == 0 {
        return Ok(());
    }
    let (size, extra) = (range.len() / count, range.len() % count);
    let mut start = range.start;
    for index in 0..count {
        let end = start + size + usize::from(index < extra);
        groups.push(start..end);
        start = end;
    }
    Ok(())
}

fn chapter_headings(doc: &Document) -> Vec<(String, Span)> {
    let mut explicit: Vec<_> = doc
        .headings
        .iter()
        .filter(|heading| chapter_label(&heading.title))
        .map(|heading| (heading.title.clone(), heading.span))
        .collect();
    let mut lines = doc.source.split_inclusive('\n').peekable();
    let mut start = 0;
    let mut previous_blank = true;
    while let Some(line) = lines.next() {
        let title = line.trim();
        let end = start + line.len();
        if previous_blank
            && lines.peek().is_none_or(|next| next.trim().is_empty())
            && chapter_label(title)
            && !doc.prose[start..end].trim().is_empty()
        {
            explicit.push((title.to_string(), Span::new(start, end)));
        }
        previous_blank = title.is_empty();
        start = end;
    }
    explicit.sort_by_key(|(_, span)| span.start);
    explicit.dedup_by_key(|(_, span)| span.start);
    if explicit.len() >= 2 {
        return explicit;
    }
    let level = (1..=6).find(|level| doc.headings.iter().filter(|heading| heading.level == *level).count() >= 2);
    level.map_or_else(Vec::new, |level| {
        doc.headings
            .iter()
            .filter(|heading| heading.level == level)
            .map(|heading| (heading.title.clone(), heading.span))
            .collect()
    })
}

fn chapter_label(title: &str) -> bool {
    if title.len() > 120 {
        return false;
    }
    let lower = title.to_ascii_lowercase();
    if matches!(lower.as_str(), "prologue" | "epilogue") {
        return true;
    }
    let Some(rest) = lower.strip_prefix("chapter ") else { return false };
    let token = rest.split_whitespace().next().unwrap_or_default().trim_end_matches(['.', ':']);
    let number = !token.is_empty()
        && (token.bytes().all(|byte| byte.is_ascii_digit())
            || token.bytes().all(|byte| b"ivxlcdm".contains(&byte))
            || token.split('-').all(|word| {
                matches!(
                    word,
                    "one"
                        | "two"
                        | "three"
                        | "four"
                        | "five"
                        | "six"
                        | "seven"
                        | "eight"
                        | "nine"
                        | "ten"
                        | "eleven"
                        | "twelve"
                        | "thirteen"
                        | "fourteen"
                        | "fifteen"
                        | "sixteen"
                        | "seventeen"
                        | "eighteen"
                        | "nineteen"
                        | "twenty"
                        | "thirty"
                        | "forty"
                        | "fifty"
                        | "sixty"
                        | "seventy"
                        | "eighty"
                        | "ninety"
                        | "hundred"
                )
            }));
    let remainder = rest[token.len()..].trim();
    number
        && (remainder.is_empty()
            || remainder == "."
            || remainder.starts_with([':', '-', '—'])
            || !(title.ends_with('.') || title.ends_with('?') || title.ends_with('!')))
}

#[cfg(test)]
mod tests;
