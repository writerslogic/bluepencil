use std::collections::{BTreeMap, HashSet};

use bluepencil_core::analysis::{
    adverbs, cliches, counts::Counts, dialogue_tags, echoes, filter_words, hedges, nominalization, passive, pov,
    repetition, rhythm, starters, tense,
};
use bluepencil_core::lexicon::WordSet;
use bluepencil_core::{Document, Finding, Format, Lexicons, Span};
use serde::Serialize;

pub const BROWSER_INPUT_BYTES: usize = 16 * 1024 * 1024;
const LEGACY_FINDINGS: usize = 150;

#[derive(Serialize)]
pub struct FindingView {
    rule: &'static str,
    message: String,
    excerpt: String,
    line: usize,
    column: usize,
    start: usize,
    end: usize,
    priority: u8,
}

#[derive(Serialize)]
pub struct Summary {
    words: usize,
    sentences: usize,
    paragraphs: usize,
    total_findings: usize,
    rule_counts: BTreeMap<&'static str, usize>,
    length: usize,
}

#[derive(Serialize)]
pub struct LegacyAnalysis {
    words: usize,
    sentences: usize,
    paragraphs: usize,
    findings: Vec<FindingView>,
    truncated: bool,
}

#[derive(Serialize)]
pub struct VisibleReview {
    findings: Vec<FindingView>,
    total: usize,
    truncated: bool,
    rule_counts: BTreeMap<&'static str, usize>,
}

#[derive(Serialize)]
pub struct ChartPoint {
    from: usize,
    to: usize,
    first_word: usize,
    last_word: usize,
    sentence_length: f64,
    paragraph_length: f64,
    dialogue_share: f64,
    finding_density: f64,
    sentence_variation: f64,
}

#[derive(Serialize)]
pub struct Chart {
    points: Vec<ChartPoint>,
    interval: usize,
    words: usize,
}

struct MetricSentence {
    span: Span,
    words: usize,
    first_word: usize,
    paragraph: usize,
    paragraph_words: usize,
    dialogue_words: usize,
    findings: usize,
}

pub struct BrowserManuscript {
    doc: Document,
    lex: Lexicons,
    findings: Vec<Finding>,
    counts: Counts,
    stage: usize,
    source_index: SourceIndex,
    max_ends: Vec<usize>,
    metrics: Vec<MetricSentence>,
}

pub fn parse_format(format: &str) -> Result<Format, &'static str> {
    match format {
        "markdown" => Ok(Format::Markdown),
        "fountain" => Ok(Format::Fountain),
        "plain" => Ok(Format::Plain),
        _ => Err("Choose Markdown, Fountain, or plain text."),
    }
}

fn validate_size(bytes: usize, max_bytes: usize) -> Result<(), String> {
    if bytes > max_bytes {
        return Err(format!(
            "This document exceeds the {} MiB analysis limit. Open a smaller document or use the CLI.",
            max_bytes / (1024 * 1024)
        ));
    }
    Ok(())
}

impl BrowserManuscript {
    pub const STAGES: [&'static str; 6] = [
        "Checking repeated words, phrases, and openings",
        "Checking sentence rhythm",
        "Checking phrases and constructions",
        "Checking word habits",
        "Checking dialogue tags",
        "Checking narrative consistency",
    ];

    pub fn new(source: &str, format: Format, max_bytes: usize) -> Result<Self, String> {
        validate_size(source.len(), max_bytes)?;
        Ok(Self {
            doc: Document::parse("Your manuscript", source, format),
            lex: Lexicons::default(),
            findings: Vec::new(),
            counts: Counts::default(),
            stage: 0,
            source_index: SourceIndex::new(source),
            max_ends: Vec::new(),
            metrics: Vec::new(),
        })
    }

    pub fn is_finished(&self) -> bool {
        self.stage == Self::STAGES.len()
    }

    pub fn next_stage(&self) -> &'static str {
        Self::STAGES.get(self.stage).copied().unwrap_or("Analysis complete")
    }

    pub fn advance(&mut self) {
        match self.stage {
            0 => self.repetition(),
            1 => {
                let findings = rhythm::rhythm(&self.doc, 4, 3, 40).findings;
                self.findings.extend(findings.into_iter().filter(|finding| {
                    finding.rule != "monotony"
                        || (!overlaps_dialogue(&self.doc, finding.span) && !crosses_heading(&self.doc, finding.span))
                }));
            }
            2 => {
                self.findings.extend(cliches::find(&self.doc, &self.lex));
                self.findings.extend(nominalization::find(&self.doc, &self.lex));
                self.findings.extend(passive::find(&self.doc, &self.lex));
            }
            3 => {
                self.findings.extend(
                    filter_words::find(&self.doc, &self.lex).into_iter().filter(|finding| {
                        !self.doc.in_dialogue(finding.span) && !sensory_description(&self.doc, finding)
                    }),
                );
                self.findings.extend(hedges::find(&self.doc, &self.lex));
                self.findings.extend(adverbs::find(&self.doc, &self.lex));
            }
            4 => self.dialogue(),
            5 => self.narrative(),
            _ => return,
        }
        self.stage += 1;
        if self.is_finished() {
            self.finish();
        }
    }

    fn repetition(&mut self) {
        for echo in echoes::echoes(&self.doc, &self.lex, 50, 4, &WordSet::default(), false) {
            if !crosses_heading(&self.doc, Span::new(echo.first.start, echo.second.end)) {
                self.findings.push(Finding {
                    rule: "echo",
                    message: format!("\"{}\" repeats {} words later", echo.word, echo.distance),
                    span: echo.second,
                });
            }
        }
        for repeat in repetition::repeats(&self.doc, &self.lex, 4, 6, 3) {
            for span in repeat.spans {
                self.findings.push(Finding {
                    rule: "repeated-phrase",
                    message: format!("\"{}\" appears {} times", repeat.phrase, repeat.count),
                    span,
                });
            }
        }
        self.findings.extend(starters::starters(&self.doc, 3).findings.into_iter().filter(|finding| {
            !overlaps_dialogue(&self.doc, finding.span) && !crosses_heading(&self.doc, finding.span)
        }));
    }

    fn dialogue(&mut self) {
        if self.doc.format == Format::Fountain {
            return;
        }
        for attribution in dialogue_tags::attributions(&self.doc, &self.lex) {
            let Some(span) = attribution.tag else { continue };
            if attribution.kind == Some(dialogue_tags::TagKind::Showy) {
                self.findings.push(Finding {
                    rule: "dialogue-tag",
                    message: format!("\"{}\" is an expressive dialogue tag", self.doc.text(span)),
                    span,
                });
            }
            if attribution.adverb {
                self.findings.push(Finding {
                    rule: "adverbial-tag",
                    message: format!("\"{}\" is modified by an adverb", self.doc.text(span)),
                    span,
                });
            }
        }
    }

    fn narrative(&mut self) {
        let tense = tense::tense(&self.doc, 3);
        let total = tense.past_sentences + tense.present_sentences;
        if total >= 12 && tense.past_sentences.max(tense.present_sentences) * 5 >= total * 4 {
            self.findings
                .extend(tense.findings.into_iter().filter(|finding| !crosses_heading(&self.doc, finding.span)));
        }
        let pov = pov::pov(&self.doc, 3);
        let total = pov.first_sentences + pov.second_sentences + pov.third_sentences;
        let dominant = pov.first_sentences.max(pov.second_sentences).max(pov.third_sentences);
        if total >= 12 && dominant * 5 >= total * 4 {
            self.findings.extend(pov.findings.into_iter().filter(|finding| !crosses_heading(&self.doc, finding.span)));
        }
    }

    fn finish(&mut self) {
        self.findings.sort_by(|a, b| (a.span, a.rule).cmp(&(b.span, b.rule)));
        self.findings.dedup_by(|a, b| a.span == b.span && a.rule == b.rule);
        self.counts = Counts::of(&self.doc);
        let mut max_end = 0;
        for finding in &self.findings {
            max_end = max_end.max(finding.span.end);
            self.max_ends.push(max_end);
        }
        let mut first_word = 1;
        for (index, paragraph) in self.doc.paragraphs.iter().enumerate() {
            let paragraph_words = paragraph.word_count();
            for sentence in &paragraph.sentences {
                if sentence.words.is_empty() {
                    continue;
                }
                let first = self.findings.partition_point(|finding| finding.span.start < sentence.span.start);
                let last = self.findings.partition_point(|finding| finding.span.start < sentence.span.end);
                self.metrics.push(MetricSentence {
                    span: sentence.span,
                    words: sentence.words.len(),
                    first_word,
                    paragraph: index,
                    paragraph_words,
                    dialogue_words: sentence.words.iter().filter(|word| self.doc.in_dialogue(word.span)).count(),
                    findings: last - first,
                });
                first_word += sentence.words.len();
            }
        }
    }

    pub fn summary(&self) -> Result<Summary, &'static str> {
        if !self.is_finished() {
            return Err("Analysis is still running.");
        }
        Ok(Summary {
            words: self.counts.words,
            sentences: self.counts.sentences,
            paragraphs: self.counts.paragraphs,
            total_findings: self.findings.len(),
            rule_counts: rule_counts(self.findings.iter()),
            length: self.source_index.length,
        })
    }

    fn byte_range(&self, from: usize, to: usize) -> Result<Span, &'static str> {
        if !self.is_finished() {
            return Err("Analysis is still running.");
        }
        if from > to || to > self.source_index.length {
            return Err("Choose a passage within this manuscript.");
        }
        Ok(Span::new(self.source_index.byte(&self.doc.source, from), self.source_index.byte(&self.doc.source, to)))
    }

    pub fn visible(&self, from: usize, to: usize, enabled: &str) -> Result<VisibleReview, &'static str> {
        let span = self.byte_range(from, to)?;
        if enabled.len() > 1024 {
            return Err("Choose available finding types.");
        }
        let enabled: Vec<String> = serde_json::from_str(enabled).map_err(|_| "Choose available finding types.")?;
        if enabled.iter().any(|rule| priority(rule) == 0) {
            return Err("Choose available finding types.");
        }
        let enabled: HashSet<_> = enabled.iter().map(String::as_str).collect();
        let first = self.max_ends.partition_point(|end| *end <= span.start);
        let last = self.findings.partition_point(|finding| finding.span.start < span.end);
        let matches: Vec<_> =
            self.findings[first.min(last)..last].iter().filter(|finding| finding.span.end > span.start).collect();
        let rule_counts = rule_counts(matches.iter().copied());
        let selected: Vec<_> = matches.into_iter().filter(|finding| enabled.contains(finding.rule)).collect();
        let total = selected.len();
        let findings = views(&self.doc, &self.source_index, &selected[..total.min(1000)]);
        Ok(VisibleReview { findings, total, truncated: total > 1000, rule_counts })
    }

    pub fn chart(&self, from: usize, to: usize, interval: usize) -> Result<Chart, &'static str> {
        let span = self.byte_range(from, to)?;
        if ![0, 25, 50, 250, 1000].contains(&interval) {
            return Err("Choose an available chart interval.");
        }
        let first = self.metrics.partition_point(|sentence| sentence.span.end <= span.start);
        let last = self.metrics.partition_point(|sentence| sentence.span.start < span.end);
        let sentences = &self.metrics[first.min(last)..last];
        let words: usize = sentences.iter().map(|sentence| sentence.words).sum();
        let interval = if interval == 0 { words.div_ceil(100).div_ceil(25) * 25 } else { interval };
        let interval = interval.max(words.div_ceil(250).div_ceil(25) * 25).max(25);
        let mut points = Vec::new();
        let mut start = 0;
        while start < sentences.len() {
            let mut end = start;
            let mut count = 0;
            while end < sentences.len() && count < interval {
                count += sentences[end].words;
                end += 1;
            }
            let bucket = &sentences[start..end];
            let mut paragraphs = HashSet::new();
            let mut paragraph_words = 0;
            let mut dialogue = 0;
            let mut findings = 0;
            let mut squares = 0.0;
            for sentence in bucket {
                if paragraphs.insert(sentence.paragraph) {
                    paragraph_words += sentence.paragraph_words;
                }
                dialogue += sentence.dialogue_words;
                findings += sentence.findings;
                squares += (sentence.words as f64).powi(2);
            }
            let mean = count as f64 / bucket.len() as f64;
            let variance = (squares / bucket.len() as f64 - mean * mean).max(0.0);
            points.push(ChartPoint {
                from: self.source_index.position(&self.doc.source, bucket[0].span.start).utf16,
                to: self.source_index.position(&self.doc.source, bucket[bucket.len() - 1].span.end).utf16,
                first_word: bucket[0].first_word,
                last_word: bucket[bucket.len() - 1].first_word + bucket[bucket.len() - 1].words.saturating_sub(1),
                sentence_length: mean,
                paragraph_length: paragraph_words as f64 / paragraphs.len() as f64,
                dialogue_share: if count == 0 { 0.0 } else { dialogue as f64 * 100.0 / count as f64 },
                finding_density: if count == 0 { 0.0 } else { findings as f64 * 1000.0 / count as f64 },
                sentence_variation: if mean == 0.0 { 0.0 } else { variance.sqrt() * 100.0 / mean },
            });
            start = end;
        }
        Ok(Chart { points, interval, words })
    }

    pub fn legacy_analysis(&self) -> LegacyAnalysis {
        let selected: Vec<_> = self.findings.iter().take(LEGACY_FINDINGS).collect();
        LegacyAnalysis {
            words: self.counts.words,
            sentences: self.counts.sentences,
            paragraphs: self.counts.paragraphs,
            findings: views(&self.doc, &self.source_index, &selected),
            truncated: self.findings.len() > LEGACY_FINDINGS,
        }
    }
}

fn rule_counts<'a>(findings: impl Iterator<Item = &'a Finding>) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for finding in findings {
        *counts.entry(finding.rule).or_default() += 1;
    }
    counts
}

fn crosses_heading(doc: &Document, span: Span) -> bool {
    let next = doc.headings.partition_point(|heading| heading.span.start <= span.start);
    doc.headings.get(next).is_some_and(|heading| heading.span.start < span.end)
}

fn overlaps_dialogue(doc: &Document, span: Span) -> bool {
    let next = doc.dialogue.partition_point(|dialogue| dialogue.end <= span.start);
    doc.dialogue.get(next).is_some_and(|dialogue| dialogue.start < span.end)
}

fn sensory_description(doc: &Document, finding: &Finding) -> bool {
    if !matches!(finding.message.as_str(), "smell" | "smells" | "smelled" | "smelt" | "taste" | "tastes" | "tasted") {
        return false;
    }
    let next = doc.prose[finding.span.end..].split_whitespace().next().unwrap_or_default();
    next.eq_ignore_ascii_case("of") || next.eq_ignore_ascii_case("like")
}

fn priority(rule: &str) -> u8 {
    match rule {
        "repeated-phrase" => 100,
        "repeated-starter" => 95,
        "echo" => 90,
        "nominal" => 80,
        "tense-drift" | "pov-drift" => 75,
        "dialogue-tag" => 65,
        "adverbial-tag" => 60,
        "cliche" => 55,
        "long-sentence" => 50,
        "monotony" => 45,
        "passive" => 35,
        "filter" => 30,
        "hedge" => 20,
        "adverb" => 10,
        _ => 0,
    }
}

#[derive(Clone, Copy)]
struct SourcePosition {
    byte: usize,
    utf16: usize,
    line: usize,
    column: usize,
}

impl SourcePosition {
    fn advance(&mut self, character: char) {
        self.byte += character.len_utf8();
        self.utf16 += character.len_utf16();
        if character == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}

struct SourceIndex {
    checkpoints: Vec<SourcePosition>,
    length: usize,
}

impl SourceIndex {
    fn new(source: &str) -> Self {
        let mut position = SourcePosition { byte: 0, utf16: 0, line: 1, column: 1 };
        let mut checkpoints = vec![position];
        for character in source.chars() {
            position.advance(character);
            if position.byte - checkpoints.last().unwrap().byte >= 1024 {
                checkpoints.push(position);
            }
        }
        Self { checkpoints, length: position.utf16 }
    }

    fn position(&self, source: &str, byte: usize) -> SourcePosition {
        let index = self.checkpoints.partition_point(|position| position.byte <= byte) - 1;
        let mut position = self.checkpoints[index];
        for character in source[position.byte..byte].chars() {
            position.advance(character);
        }
        position
    }

    fn byte(&self, source: &str, utf16: usize) -> usize {
        let index = self.checkpoints.partition_point(|position| position.utf16 <= utf16) - 1;
        let mut position = self.checkpoints[index];
        for character in source[position.byte..].chars() {
            if position.utf16 + character.len_utf16() > utf16 {
                break;
            }
            position.advance(character);
        }
        position.byte
    }
}

fn views(doc: &Document, source_index: &SourceIndex, findings: &[&Finding]) -> Vec<FindingView> {
    let sentences: Vec<_> = doc.sentences().collect();
    findings
        .iter()
        .map(|finding| {
            let start = source_index.position(&doc.source, finding.span.start);
            let end = source_index.position(&doc.source, finding.span.end);
            let index = sentences.partition_point(|sentence| sentence.span.end <= finding.span.start);
            let context = sentences
                .get(index)
                .filter(|sentence| sentence.span.start <= finding.span.start)
                .map_or(finding.span, |sentence| sentence.span);
            FindingView {
                rule: finding.rule,
                message: finding.message.clone(),
                excerpt: excerpt(&doc.source, context, finding.span.start),
                line: start.line,
                column: start.column,
                start: start.utf16,
                end: end.utf16,
                priority: priority(finding.rule),
            }
        })
        .collect()
}

fn excerpt(source: &str, context: Span, at: usize) -> String {
    let mut start = context.start.max(at.saturating_sub(100));
    while !source.is_char_boundary(start) {
        start += 1;
    }
    let text: String = source[start..context.end].chars().take(240).collect();
    let end = start + text.len();
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("{}{text}{}", if start > context.start { "…" } else { "" }, if end < context.end { "…" } else { "" })
}

#[cfg(test)]
mod tests;
