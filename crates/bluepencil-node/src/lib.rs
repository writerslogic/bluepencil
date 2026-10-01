//! Node.js bindings for `bluepencil-core`, built with napi-rs and published to
//! npm as `bluepencil-node`.
//!
//! Every function takes the prose as a string (not a path) plus an optional
//! options object, parses it once, and returns plain objects whose locations
//! are 1-based `line`/`column` pairs with a short excerpt, so a caller never
//! needs the byte offsets `bluepencil-core` works in. The CLI's config file is
//! not consulted: defaults match `bluepencil init`, and each option can be
//! overridden per call.

use bluepencil_core::analysis::dialogue_tags::{self, TagKind};
use bluepencil_core::analysis::{
    adverbs, arc, cliches, counts, dialogue, diversity, echoes, filter_words, frequency, hedges, nominalization,
    overuse, passive, readability, repetition, rhythm, starters, tics,
};
use bluepencil_core::lexicon::WordSet;
use bluepencil_core::stats::{Summary, per_thousand};
use bluepencil_core::{Document, Finding, Format, Lexicons, Span};
use napi_derive::napi;

const EXCERPT_CHARS: usize = 80;

// ---------------------------------------------------------------------------
// Shared input/output shapes
// ---------------------------------------------------------------------------

/// Parses the `format` option: `"markdown"` (default), `"plain"`, or
/// `"fountain"`. Markup is blanked out of the prose so headings and screenplay
/// elements never count as sentences.
fn format_of(f: Option<String>) -> napi::Result<Format> {
    match f.as_deref() {
        None | Some("markdown") => Ok(Format::Markdown),
        Some("plain") => Ok(Format::Plain),
        Some("fountain") => Ok(Format::Fountain),
        Some(other) => Err(napi::Error::from_reason(format!(
            "unknown format {other:?}: expected \"markdown\", \"plain\", or \"fountain\""
        ))),
    }
}

/// Where something was found. `offset`/`length` are byte positions in the
/// text as given; `line`/`column` are 1-based.
#[napi(object)]
pub struct Location {
    pub line: u32,
    pub column: u32,
    pub offset: u32,
    pub length: u32,
    /// Single-line excerpt of the span, middle-truncated when long.
    pub excerpt: String,
}

/// A located observation a writer may want to act on.
#[napi(object)]
pub struct StyleFinding {
    /// `adverb`, `filter`, `hedge`, `cliche`, `tic`, `passive`, `nominal`,
    /// `monotony`, `long-sentence`, or `repeated-starter`.
    pub rule: String,
    pub message: String,
    pub location: Location,
    pub in_dialogue: bool,
}

#[napi(object)]
pub struct WordCount {
    pub word: String,
    pub count: u32,
}

#[napi(object)]
pub struct Distribution {
    pub count: u32,
    pub mean: f64,
    pub median: f64,
    pub stdev: f64,
    pub min: u32,
    pub max: u32,
}

#[napi(object)]
pub struct Readability {
    pub flesch_reading_ease: f64,
    pub flesch_kincaid_grade: f64,
    pub gunning_fog: f64,
    pub coleman_liau: f64,
    pub automated_readability: f64,
    pub words: u32,
    pub sentences: u32,
}

#[napi(object)]
pub struct SectionReadability {
    pub title: String,
    pub scores: Readability,
}

#[napi(object)]
pub struct DialogueRatio {
    pub dialogue_words: u32,
    pub narration_words: u32,
    /// Share of words inside quotation marks, 0..1.
    pub ratio: f64,
}

#[napi(object)]
pub struct Counts {
    pub words: u32,
    pub characters: u32,
    pub characters_no_spaces: u32,
    pub sentences: u32,
    pub paragraphs: u32,
    pub reading_minutes: f64,
    pub speaking_minutes: f64,
}

#[napi(object)]
pub struct Diversity {
    pub words: u32,
    pub unique: u32,
    /// Type-token ratio over the whole text.
    pub ttr: f64,
    /// Moving-average type-token ratio, comparable across lengths.
    pub mattr: f64,
    pub window: u32,
}

fn u(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn parse(text: String, format: Option<String>) -> napi::Result<Document> {
    Ok(Document::parse("<text>", text, format_of(format)?))
}

fn excerpt(doc: &Document, span: Span) -> String {
    let flat = span.slice(&doc.prose).split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = flat.chars().collect();
    if chars.len() <= EXCERPT_CHARS {
        return flat;
    }
    let half = (EXCERPT_CHARS - 3) / 2;
    let head: String = chars[..half].iter().collect();
    let tail: String = chars[chars.len() - half..].iter().collect();
    format!("{head}...{tail}")
}

fn location(doc: &Document, span: Span) -> Location {
    let p = doc.position(span.start);
    Location {
        line: u(p.line),
        column: u(p.column),
        offset: u(span.start),
        length: u(span.len()),
        excerpt: excerpt(doc, span),
    }
}

fn finding(doc: &Document, f: &Finding) -> StyleFinding {
    StyleFinding {
        rule: f.rule.to_string(),
        message: f.message.clone(),
        location: location(doc, f.span),
        in_dialogue: doc.in_dialogue(f.span),
    }
}

fn distribution(s: &Summary) -> Distribution {
    Distribution { count: u(s.count), mean: s.mean, median: s.median, stdev: s.stdev, min: u(s.min), max: u(s.max) }
}

fn readability_of(r: &bluepencil_core::stats::readability::Readability) -> Readability {
    Readability {
        flesch_reading_ease: r.flesch_reading_ease,
        flesch_kincaid_grade: r.flesch_kincaid_grade,
        gunning_fog: r.gunning_fog,
        coleman_liau: r.coleman_liau,
        automated_readability: r.automated_readability,
        words: u(r.words),
        sentences: u(r.sentences),
    }
}

fn dialogue_ratio(d: &dialogue::DialogueRatio) -> DialogueRatio {
    DialogueRatio { dialogue_words: u(d.dialogue_words), narration_words: u(d.narration_words), ratio: d.ratio }
}

fn word_counts(items: Vec<(String, usize)>) -> Vec<WordCount> {
    items.into_iter().map(|(word, count)| WordCount { word, count: u(count) }).collect()
}

// ---------------------------------------------------------------------------
// Style checks: every located, lexicon-driven rule in one pass
// ---------------------------------------------------------------------------

#[napi(object)]
pub struct StyleOptions {
    /// `"markdown"` (default), `"plain"`, or `"fountain"`.
    pub format: Option<String>,
    /// Extra words or phrases to flag as personal tics (`rule: "tic"`).
    pub tics: Option<Vec<String>>,
}

#[napi(object)]
pub struct StyleCounts {
    pub adverbs: u32,
    pub filter_words: u32,
    pub hedges: u32,
    pub cliches: u32,
    pub tics: u32,
    pub passive: u32,
    pub nominalizations: u32,
}

#[napi(object)]
pub struct StyleRates {
    pub adverbs: f64,
    pub filter_words: f64,
    pub hedges: f64,
    pub cliches: f64,
    pub tics: f64,
    pub passive: f64,
    pub nominalizations: f64,
}

#[napi(object)]
pub struct StyleReport {
    pub words: u32,
    pub counts: StyleCounts,
    /// Each count per 1,000 words, so documents of different lengths compare.
    pub per_thousand: StyleRates,
    /// Every finding in document order.
    pub findings: Vec<StyleFinding>,
}

fn lexicons(tics: Option<Vec<String>>) -> Lexicons {
    let mut lex = Lexicons::default();
    if let Some(words) = tics {
        lex.tics.extend(words.iter().map(String::as_str));
    }
    lex
}

/// Flag adverbs, filter words, hedges and intensifiers, clichés, personal
/// tics, passive constructions, and nominalizations, each with its location.
#[napi]
pub fn check_style(text: String, options: Option<StyleOptions>) -> napi::Result<StyleReport> {
    let options = options.unwrap_or(StyleOptions { format: None, tics: None });
    let doc = parse(text, options.format)?;
    let lex = lexicons(options.tics);
    let groups: [Vec<Finding>; 7] = [
        adverbs::find(&doc, &lex),
        filter_words::find(&doc, &lex),
        hedges::find(&doc, &lex),
        cliches::find(&doc, &lex),
        tics::find(&doc, &lex),
        passive::find(&doc, &lex),
        nominalization::find(&doc, &lex),
    ];
    let words = doc.word_count();
    let n = |i: usize| u(groups[i].len());
    let rate = |i: usize| per_thousand(groups[i].len(), words);
    let counts = StyleCounts {
        adverbs: n(0),
        filter_words: n(1),
        hedges: n(2),
        cliches: n(3),
        tics: n(4),
        passive: n(5),
        nominalizations: n(6),
    };
    let per_thousand = StyleRates {
        adverbs: rate(0),
        filter_words: rate(1),
        hedges: rate(2),
        cliches: rate(3),
        tics: rate(4),
        passive: rate(5),
        nominalizations: rate(6),
    };
    let mut all: Vec<&Finding> = groups.iter().flatten().collect();
    all.sort_by_key(|f| (f.span.start, f.span.end));
    let findings = all.into_iter().map(|f| finding(&doc, f)).collect();
    Ok(StyleReport { words: u(words), counts, per_thousand, findings })
}

// ---------------------------------------------------------------------------
// Echoes and repeats
// ---------------------------------------------------------------------------

#[napi(object)]
pub struct EchoOptions {
    /// `"markdown"` (default), `"plain"`, or `"fountain"`.
    pub format: Option<String>,
    /// Max words between two uses for them to count as an echo. Default 50.
    pub window: Option<u32>,
    /// Shortest word (in characters) to consider. Default 4.
    pub min_length: Option<u32>,
    /// Count capitalized names too. Default false.
    pub include_names: Option<bool>,
    /// Words never to report.
    pub ignore: Option<Vec<String>>,
}

#[napi(object)]
pub struct Echo {
    pub word: String,
    pub first: Location,
    pub second: Location,
    /// Words between the two uses.
    pub distance: u32,
}

/// The same word reappearing within a window of words.
#[napi]
pub fn find_echoes(text: String, options: Option<EchoOptions>) -> napi::Result<Vec<Echo>> {
    let o = options.unwrap_or(EchoOptions {
        format: None,
        window: None,
        min_length: None,
        include_names: None,
        ignore: None,
    });
    let doc = parse(text, o.format)?;
    let lex = Lexicons::default();
    let ignore = WordSet::from_list(o.ignore.as_deref().unwrap_or_default().iter().map(String::as_str));
    Ok(echoes::echoes(
        &doc,
        &lex,
        o.window.unwrap_or(50) as usize,
        o.min_length.unwrap_or(4) as usize,
        &ignore,
        o.include_names.unwrap_or(false),
    )
    .into_iter()
    .map(|e| Echo {
        word: e.word,
        first: location(&doc, e.first),
        second: location(&doc, e.second),
        distance: u(e.distance),
    })
    .collect())
}

#[napi(object)]
pub struct RepeatOptions {
    /// `"markdown"` (default), `"plain"`, or `"fountain"`.
    pub format: Option<String>,
    /// Shortest phrase length in words. Default 3.
    pub min_words: Option<u32>,
    /// Longest phrase length in words. Default 6.
    pub max_words: Option<u32>,
    /// Fewest occurrences to report. Default 2.
    pub min_count: Option<u32>,
}

#[napi(object)]
pub struct Repeat {
    pub phrase: String,
    pub words: u32,
    pub count: u32,
    pub locations: Vec<Location>,
}

/// Multi-word phrases that recur anywhere in the text, most frequent first.
#[napi]
pub fn find_repeats(text: String, options: Option<RepeatOptions>) -> napi::Result<Vec<Repeat>> {
    let o = options.unwrap_or(RepeatOptions { format: None, min_words: None, max_words: None, min_count: None });
    let doc = parse(text, o.format)?;
    let lex = Lexicons::default();
    Ok(repetition::repeats(
        &doc,
        &lex,
        o.min_words.unwrap_or(3) as usize,
        o.max_words.unwrap_or(6) as usize,
        o.min_count.unwrap_or(2) as usize,
    )
    .into_iter()
    .map(|r| Repeat {
        phrase: r.phrase,
        words: u(r.words),
        count: u(r.count),
        locations: r.spans.iter().map(|s| location(&doc, *s)).collect(),
    })
    .collect())
}

// ---------------------------------------------------------------------------
// Rhythm and sentence openers
// ---------------------------------------------------------------------------

#[napi(object)]
pub struct RhythmOptions {
    /// `"markdown"` (default), `"plain"`, or `"fountain"`.
    pub format: Option<String>,
    /// Consecutive sentences of similar length that count as monotonous. Default 4.
    pub run: Option<u32>,
    /// Word-count difference still considered "similar". Default 3.
    pub tolerance: Option<u32>,
    /// Sentence length (words) flagged as overlong. Default 40.
    pub long: Option<u32>,
}

#[napi(object)]
pub struct RhythmReport {
    /// Word count of every sentence, in order.
    pub lengths: Vec<u32>,
    pub summary: Distribution,
    /// Stdev over mean; low values read as monotonous.
    pub variation: f64,
    /// `monotony` runs and `long-sentence` findings.
    pub findings: Vec<StyleFinding>,
}

/// Sentence-length flow: monotonous runs and overlong sentences.
#[napi]
pub fn analyze_rhythm(text: String, options: Option<RhythmOptions>) -> napi::Result<RhythmReport> {
    let o = options.unwrap_or(RhythmOptions { format: None, run: None, tolerance: None, long: None });
    let doc = parse(text, o.format)?;
    let r = rhythm::rhythm(
        &doc,
        o.run.unwrap_or(4) as usize,
        o.tolerance.unwrap_or(3) as usize,
        o.long.unwrap_or(40) as usize,
    );
    Ok(RhythmReport {
        lengths: r.lengths.iter().map(|&n| u(n)).collect(),
        summary: distribution(&r.summary),
        variation: r.variation,
        findings: r.findings.iter().map(|f| finding(&doc, f)).collect(),
    })
}

#[napi(object)]
pub struct StarterOptions {
    /// `"markdown"` (default), `"plain"`, or `"fountain"`.
    pub format: Option<String>,
    /// Consecutive sentences opening with the same word to flag. Default 3.
    pub run: Option<u32>,
}

#[napi(object)]
pub struct StarterReport {
    /// Most common sentence-opening words.
    pub sentence: Vec<WordCount>,
    /// Most common paragraph-opening words.
    pub paragraph: Vec<WordCount>,
    /// `repeated-starter` runs.
    pub findings: Vec<StyleFinding>,
}

/// How sentences and paragraphs open, and runs that open the same way.
#[napi]
pub fn sentence_starters(text: String, options: Option<StarterOptions>) -> napi::Result<StarterReport> {
    let o = options.unwrap_or(StarterOptions { format: None, run: None });
    let doc = parse(text, o.format)?;
    let s = starters::starters(&doc, o.run.unwrap_or(3) as usize);
    Ok(StarterReport {
        sentence: word_counts(s.sentence),
        paragraph: word_counts(s.paragraph),
        findings: s.findings.iter().map(|f| finding(&doc, f)).collect(),
    })
}

// ---------------------------------------------------------------------------
// Dialogue
// ---------------------------------------------------------------------------

#[napi(object)]
pub struct FormatOptions {
    /// `"markdown"` (default), `"plain"`, or `"fountain"`.
    pub format: Option<String>,
}

#[napi(object)]
pub struct Attribution {
    pub dialogue: Location,
    pub tag: Option<Location>,
    /// `plain` (said/asked), `showy` (exclaimed, snapped), or null when untagged.
    pub kind: Option<String>,
    /// Whether the tag carries an adverb ("she said softly").
    pub adverb: bool,
    pub speaker: Option<String>,
}

#[napi(object)]
pub struct DialogueReport {
    pub ratio: DialogueRatio,
    pub by_section: Vec<SectionDialogue>,
    pub attributions: Vec<Attribution>,
    pub plain_tags: u32,
    pub showy_tags: u32,
    pub adverb_tags: u32,
    pub untagged: u32,
}

#[napi(object)]
pub struct SectionDialogue {
    pub section: String,
    pub ratio: DialogueRatio,
}

/// Dialogue versus narration, overall and per section, plus every quoted line
/// with its tag classified.
#[napi]
pub fn analyze_dialogue(text: String, options: Option<FormatOptions>) -> napi::Result<DialogueReport> {
    let doc = parse(text, options.and_then(|o| o.format))?;
    let lex = Lexicons::default();
    let attributions: Vec<Attribution> = dialogue_tags::attributions(&doc, &lex)
        .into_iter()
        .map(|a| Attribution {
            dialogue: location(&doc, a.dialogue),
            tag: a.tag.map(|s| location(&doc, s)),
            kind: a.kind.map(|k| match k {
                TagKind::Plain => "plain".to_string(),
                TagKind::Showy => "showy".to_string(),
            }),
            adverb: a.adverb,
            speaker: a.speaker,
        })
        .collect();
    let plain_tags = u(attributions.iter().filter(|a| a.kind.as_deref() == Some("plain")).count());
    let showy_tags = u(attributions.iter().filter(|a| a.kind.as_deref() == Some("showy")).count());
    let adverb_tags = u(attributions.iter().filter(|a| a.adverb).count());
    let untagged = u(attributions.iter().filter(|a| a.tag.is_none()).count());
    Ok(DialogueReport {
        ratio: dialogue_ratio(&dialogue::ratio(&doc)),
        by_section: dialogue::by_section(&doc)
            .into_iter()
            .map(|(section, r)| SectionDialogue { section, ratio: dialogue_ratio(&r) })
            .collect(),
        attributions,
        plain_tags,
        showy_tags,
        adverb_tags,
        untagged,
    })
}

// ---------------------------------------------------------------------------
// Arc and the at-a-glance report
// ---------------------------------------------------------------------------

#[napi(object)]
pub struct SectionArc {
    pub section: String,
    pub words: u32,
    pub dialogue_ratio: f64,
    pub sentence_words_mean: f64,
    pub adverbs: u32,
    pub adverbs_per_1k: f64,
    pub passive: u32,
    pub passive_per_1k: f64,
}

/// Pacing profile per section (heading): length, dialogue share, sentence
/// rhythm, and style density.
#[napi]
pub fn section_arc(text: String, options: Option<FormatOptions>) -> napi::Result<Vec<SectionArc>> {
    let doc = parse(text, options.and_then(|o| o.format))?;
    let lex = Lexicons::default();
    Ok(arc::arc(&doc, &lex)
        .into_iter()
        .map(|s| SectionArc {
            section: s.section,
            words: u(s.words),
            dialogue_ratio: s.dialogue_ratio,
            sentence_words_mean: s.sentence_words_mean,
            adverbs: u(s.adverbs),
            adverbs_per_1k: s.adverbs_per_1k,
            passive: u(s.passive),
            passive_per_1k: s.passive_per_1k,
        })
        .collect())
}

#[napi(object)]
pub struct Overused {
    pub word: String,
    pub count: u32,
    pub document_per_million: f64,
    pub baseline_per_million: f64,
    /// Document rate over baseline English rate.
    pub ratio: f64,
}

#[napi(object)]
pub struct ReportOptions {
    /// `"markdown"` (default), `"plain"`, or `"fountain"`.
    pub format: Option<String>,
    pub tics: Option<Vec<String>>,
    /// How many top words, repeats, and overused words to include. Default 15.
    pub top: Option<u32>,
}

#[napi(object)]
pub struct Report {
    pub counts: Counts,
    pub readability: Readability,
    pub readability_by_section: Vec<SectionReadability>,
    pub diversity: Diversity,
    pub dialogue: DialogueRatio,
    pub sentence_lengths: Distribution,
    pub style: StyleCounts,
    pub style_per_thousand: StyleRates,
    pub echoes: u32,
    pub echoes_per_thousand: f64,
    pub repeated_starter_runs: u32,
    pub monotonous_runs: u32,
    pub long_sentences: u32,
    pub top_words: Vec<WordCount>,
    pub top_repeats: Vec<WordCount>,
    pub top_starters: Vec<WordCount>,
    pub overused: Vec<Overused>,
}

/// Everything at a glance for one text, matching `bluepencil report` with
/// default settings.
#[napi]
pub fn report(text: String, options: Option<ReportOptions>) -> napi::Result<Report> {
    let o = options.unwrap_or(ReportOptions { format: None, tics: None, top: None });
    let top = o.top.unwrap_or(15) as usize;
    let doc = parse(text, o.format)?;
    let lex = lexicons(o.tics);
    let words = doc.word_count();
    let rate = |n: usize| per_thousand(n, words);

    let style_counts = [
        adverbs::find(&doc, &lex).len(),
        filter_words::find(&doc, &lex).len(),
        hedges::find(&doc, &lex).len(),
        cliches::find(&doc, &lex).len(),
        tics::find(&doc, &lex).len(),
        passive::find(&doc, &lex).len(),
        nominalization::find(&doc, &lex).len(),
    ];
    let style = StyleCounts {
        adverbs: u(style_counts[0]),
        filter_words: u(style_counts[1]),
        hedges: u(style_counts[2]),
        cliches: u(style_counts[3]),
        tics: u(style_counts[4]),
        passive: u(style_counts[5]),
        nominalizations: u(style_counts[6]),
    };
    let style_per_thousand = StyleRates {
        adverbs: rate(style_counts[0]),
        filter_words: rate(style_counts[1]),
        hedges: rate(style_counts[2]),
        cliches: rate(style_counts[3]),
        tics: rate(style_counts[4]),
        passive: rate(style_counts[5]),
        nominalizations: rate(style_counts[6]),
    };

    let echo_count = echoes::echoes(&doc, &lex, 50, 4, &WordSet::from_list([]), false).len();
    let st = starters::starters(&doc, 3);
    let rh = rhythm::rhythm(&doc, 4, 3, 40);
    let lowered: Vec<&str> = doc.words().map(|w| w.lower.as_str()).collect();
    let lengths: Vec<usize> = doc.sentences().map(|s| s.words.len()).collect();
    let docs = std::slice::from_ref(&doc);
    let c = counts::Counts::of(&doc);
    let dv = diversity::diversity(&lowered, 50);

    Ok(Report {
        counts: Counts {
            words: u(c.words),
            characters: u(c.characters),
            characters_no_spaces: u(c.characters_no_spaces),
            sentences: u(c.sentences),
            paragraphs: u(c.paragraphs),
            reading_minutes: c.reading_minutes,
            speaking_minutes: c.speaking_minutes,
        },
        readability: readability_of(&readability::document(&doc)),
        readability_by_section: readability::sections(&doc)
            .into_iter()
            .map(|s| SectionReadability { title: s.title, scores: readability_of(&s.scores) })
            .collect(),
        diversity: Diversity {
            words: u(dv.words),
            unique: u(dv.unique),
            ttr: dv.ttr,
            mattr: dv.mattr,
            window: u(dv.window),
        },
        dialogue: dialogue_ratio(&dialogue::ratio(&doc)),
        sentence_lengths: distribution(&Summary::of(&lengths)),
        style,
        style_per_thousand,
        echoes: u(echo_count),
        echoes_per_thousand: rate(echo_count),
        repeated_starter_runs: u(st.findings.len()),
        monotonous_runs: u(rh.findings.iter().filter(|f| f.rule == "monotony").count()),
        long_sentences: u(rh.findings.iter().filter(|f| f.rule == "long-sentence").count()),
        top_words: word_counts(frequency::frequency(docs, &lex, false, 3).into_iter().take(top).collect()),
        top_repeats: word_counts(
            repetition::repeats(&doc, &lex, 3, 6, 2).into_iter().map(|r| (r.phrase, r.count)).take(top).collect(),
        ),
        top_starters: word_counts(st.sentence.into_iter().take(top).collect()),
        overused: overuse::overused(docs, &lex, 3.0, 3)
            .into_iter()
            .take(top)
            .map(|o| Overused {
                word: o.word,
                count: u(o.count),
                document_per_million: o.document_per_million,
                baseline_per_million: o.baseline_per_million,
                ratio: o.ratio,
            })
            .collect(),
    })
}
