use anyhow::{Context as _, Result, bail};
use bluepencil_core::analysis::counts::Counts;
use bluepencil_core::analysis::dialogue::{self, DialogueRatio};
use bluepencil_core::analysis::diversity::{Diversity, diversity};
use bluepencil_core::analysis::{
    adverbs, cliches, echoes, filter_words, frequency, hedges, nominalization, overuse, passive, readability,
    repetition, rhythm, starters, tics,
};
use bluepencil_core::stats::readability::Readability;
use bluepencil_core::stats::{Summary, per_thousand};
use bluepencil_core::{Document, Finding, Lexicons};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cache::Cache;
use crate::cli::ReportArgs;
use crate::context::Context;
use crate::output::human::{minutes, thousands};
use crate::output::table::{Align, Table};
use crate::output::{html, json};

#[derive(Serialize, JsonSchema)]
pub struct ReportOutput {
    pub total: Metrics,
    pub files: Vec<Metrics>,
    pub revisions: Vec<RevisionItem>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RevisionItem {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub rule: String,
    pub message: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Metrics {
    pub name: String,
    pub counts: Counts,
    pub readability: Readability,
    pub diversity: Diversity,
    pub dialogue: DialogueRatio,
    pub sentence_lengths: Summary,
    pub echoes: usize,
    pub adverbs: usize,
    pub filter: usize,
    pub hedges: usize,
    pub cliches: usize,
    pub tics: usize,
    pub passive: usize,
    pub nominal: usize,
    pub overused: usize,
    pub repeated_starter_runs: usize,
    pub monotonous_runs: usize,
    pub long_sentences: usize,
    pub top_words: Vec<(String, usize)>,
    pub top_repeats: Vec<(String, usize)>,
    pub top_starters: Vec<(String, usize)>,
}

impl Metrics {
    pub fn per_1k(&self, n: usize) -> f64 {
        per_thousand(n, self.counts.words)
    }
}

pub fn measure(ctx: &Context, docs: &[Document], name: &str) -> Metrics {
    let lex: &Lexicons = &ctx.lexicons;
    let cfg = &ctx.config;
    let ignore = ctx.echo_ignore();
    let mut m = Metrics {
        name: name.to_string(),
        counts: Counts::default(),
        readability: Readability::default(),
        diversity: diversity(&[], 50),
        dialogue: DialogueRatio::default(),
        sentence_lengths: Summary::default(),
        echoes: 0,
        adverbs: 0,
        filter: 0,
        hedges: 0,
        cliches: 0,
        tics: 0,
        passive: 0,
        nominal: 0,
        overused: overuse::overused(docs, lex, 3.0, 3).len(),
        repeated_starter_runs: 0,
        monotonous_runs: 0,
        long_sentences: 0,
        top_words: frequency::frequency(docs, lex, false, 3).into_iter().take(15).collect(),
        top_repeats: Vec::new(),
        top_starters: Vec::new(),
    };
    let mut words: Vec<&str> = Vec::new();
    let mut lengths = Vec::new();
    let mut openers = std::collections::HashMap::<String, usize>::new();
    for d in docs {
        m.counts = m.counts.merge(Counts::of(d));
        m.dialogue = m.dialogue.merge(dialogue::ratio(d));
        words.extend(d.words().map(|w| w.lower.as_str()));
        lengths.extend(d.sentences().map(|s| s.words.len()));
        m.echoes +=
            echoes::echoes(d, lex, cfg.echoes.window, cfg.echoes.min_length, &ignore, cfg.echoes.include_names).len();
        m.adverbs += adverbs::find(d, lex).len();
        m.filter += filter_words::find(d, lex).len();
        m.hedges += hedges::find(d, lex).len();
        m.cliches += cliches::find(d, lex).len();
        m.tics += tics::find(d, lex).len();
        m.passive += passive::find(d, lex).len();
        m.nominal += nominalization::find(d, lex).len();
        let st = starters::starters(d, cfg.starters.run);
        m.repeated_starter_runs += st.findings.len();
        for (w, n) in st.sentence {
            *openers.entry(w).or_default() += n;
        }
        let rh = rhythm::rhythm(d, cfg.rhythm.run, cfg.rhythm.tolerance, cfg.rhythm.long);
        m.monotonous_runs += rh.findings.iter().filter(|f| f.rule == "monotony").count();
        m.long_sentences += rh.findings.iter().filter(|f| f.rule == "long-sentence").count();
        m.top_repeats.extend(
            repetition::repeats(d, lex, cfg.repeats.min, cfg.repeats.max, cfg.repeats.count)
                .into_iter()
                .map(|r| (r.phrase, r.count)),
        );
    }
    m.readability = combined_readability(docs);
    m.diversity = diversity(&words, 50);
    m.sentence_lengths = Summary::of(&lengths);
    m.top_repeats.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    m.top_repeats.truncate(10);
    let mut openers: Vec<_> = openers.into_iter().collect();
    openers.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    openers.truncate(8);
    m.top_starters = openers;
    m
}

fn measure_cached(ctx: &Context, cache: Option<&Cache>, doc: &Document, name: &str) -> Metrics {
    // The cache key is content + config only, not `name`, so the same file cached under one
    // name (e.g. a single-file report's "all files") and then looked up under another (its own
    // path, in a later multi-file report) still hits -- but the retrieved value must be
    // relabeled to the name this call actually asked for, or the wrong filename leaks into
    // the "by file" breakdown.
    if let Some(mut m) = cache.and_then(|cache| cache.get(doc, true)) {
        m.name = name.to_string();
        return m;
    }
    let m = measure(ctx, std::slice::from_ref(doc), name);
    if let Some(cache) = cache {
        cache.put(doc, &m);
    }
    m
}

/// Word-weighted average of each document's scores.
fn combined_readability(docs: &[Document]) -> Readability {
    let scores: Vec<Readability> = docs.iter().map(readability::document).collect();
    let words: usize = scores.iter().map(|r| r.words).sum();
    if words == 0 {
        return Readability::default();
    }
    let avg = |f: fn(&Readability) -> f64| scores.iter().map(|r| f(r) * r.words as f64).sum::<f64>() / words as f64;
    Readability {
        flesch_reading_ease: avg(|r| r.flesch_reading_ease),
        flesch_kincaid_grade: avg(|r| r.flesch_kincaid_grade),
        gunning_fog: avg(|r| r.gunning_fog),
        coleman_liau: avg(|r| r.coleman_liau),
        automated_readability: avg(|r| r.automated_readability),
        words,
        sentences: scores.iter().map(|r| r.sentences).sum(),
    }
}

fn revision_item(doc: &Document, finding: &Finding) -> RevisionItem {
    let pos = doc.position(finding.span.start);
    let sentence = doc
        .sentences()
        .find(|s| s.span.start <= finding.span.start && finding.span.start < s.span.end)
        .map_or(finding.span, |s| s.span);
    RevisionItem {
        file: doc.name.clone(),
        line: pos.line,
        column: pos.column,
        rule: finding.rule.to_string(),
        message: finding.message.clone(),
        excerpt: crate::output::excerpt(doc, sentence, 160),
    }
}

fn revisions(ctx: &Context, docs: &[Document]) -> Vec<RevisionItem> {
    let mut ranked = Vec::new();
    let lex = &ctx.lexicons;
    let cfg = &ctx.config;
    let ignore = ctx.echo_ignore();
    for doc in docs {
        let echoes =
            echoes::echoes(doc, lex, cfg.echoes.window, cfg.echoes.min_length, &ignore, cfg.echoes.include_names);
        for echo in echoes {
            let finding = Finding {
                rule: "echo",
                message: format!("\"{}\" repeats {} words later", echo.word, echo.distance),
                span: echo.second,
            };
            ranked.push((100u8.saturating_sub(echo.distance.min(50) as u8), revision_item(doc, &finding)));
        }
        let mut add = |rank: u8, findings: Vec<Finding>| {
            ranked.extend(findings.iter().map(|f| (rank, revision_item(doc, f))));
        };
        add(90, cliches::find(doc, lex));
        add(85, starters::starters(doc, cfg.starters.run).findings);
        add(80, rhythm::rhythm(doc, cfg.rhythm.run, cfg.rhythm.tolerance, cfg.rhythm.long).findings);
        add(75, tics::find(doc, lex));
        add(70, passive::find(doc, lex));
        add(65, nominalization::find(doc, lex));
        add(60, filter_words::find(doc, lex));
        add(55, hedges::find(doc, lex));
        add(40, adverbs::find(doc, lex));
    }
    ranked.sort_by(|(a_rank, a), (b_rank, b)| {
        b_rank
            .cmp(a_rank)
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.column.cmp(&b.column))
    });
    let mut per_rule = std::collections::HashMap::<String, usize>::new();
    ranked
        .into_iter()
        .filter_map(|(_, item)| {
            let count = per_rule.entry(item.rule.clone()).or_default();
            if *count >= 3 {
                return None;
            }
            *count += 1;
            Some(item)
        })
        .take(ctx.limit.min(50))
        .collect()
}

pub fn run(ctx: &Context, args: &ReportArgs) -> Result<()> {
    if args.trend_since.is_some() && args.html.is_none() {
        bail!("--trend-since needs --html to write the chart to");
    }

    let docs = ctx.documents(&args.input)?;
    let cache = (!args.no_cache).then(|| ctx.cache());
    // `overused`, `top_words`, and `diversity` look across all documents together, so only a
    // single-document `total` (docs.len() == 1) is safe to serve from the per-file cache; with
    // more than one document the total must see every document's raw words at once.
    let total = if docs.len() == 1 {
        measure_cached(ctx, cache.as_ref(), &docs[0], "all files")
    } else {
        measure(ctx, &docs, "all files")
    };
    let files: Vec<Metrics> = if docs.len() > 1 {
        docs.iter().map(|d| measure_cached(ctx, cache.as_ref(), d, &d.name)).collect()
    } else {
        Vec::new()
    };
    let revisions = revisions(ctx, &docs);

    if let Some(path) = &args.html {
        let trend = match &args.trend_since {
            Some(since) => super::trend::collect(ctx, &docs, since, args.trend_points)?,
            None => Vec::new(),
        };
        std::fs::write(path, html::report(&total, &files, &revisions, &trend))
            .with_context(|| format!("writing {}", path.display()))?;
        if !ctx.json {
            println!("Wrote {}\n", path.display());
        }
    }
    if ctx.json {
        return json::print(&ReportOutput { total, files, revisions });
    }
    print_human(&total, &files);
    if !revisions.is_empty() {
        println!("\nREVIEW NEXT");
        for item in revisions.iter().take(5) {
            println!("{}:{}:{}  {}  {}", item.file, item.line, item.column, item.rule, item.message);
            println!("    {}", item.excerpt);
        }
    }
    Ok(())
}

fn print_human(m: &Metrics, files: &[Metrics]) {
    let c = &m.counts;
    let r = &m.readability;
    let s = &m.sentence_lengths;
    println!("OVERVIEW");
    println!(
        "  {} words, {} sentences, {} paragraphs",
        thousands(c.words),
        thousands(c.sentences),
        thousands(c.paragraphs)
    );
    println!("  reading time {}, read aloud {}", minutes(c.reading_minutes), minutes(c.speaking_minutes));
    println!("\nSTYLE");
    println!("  sentence length   mean {:.1}, median {:.1}, sd {:.1}, longest {}", s.mean, s.median, s.stdev, s.max);
    println!(
        "  readability       ease {:.1}, grade {:.1}, fog {:.1}",
        r.flesch_reading_ease, r.flesch_kincaid_grade, r.gunning_fog
    );
    println!("  diversity         MATTR {:.3} ({} unique words)", m.diversity.mattr, thousands(m.diversity.unique));
    println!("  dialogue          {:.0}% of words", m.dialogue.ratio * 100.0);

    println!("\nFLAGS (per 1,000 words)");
    let mut t = Table::new(&[("", Align::Left), ("count", Align::Right), ("per 1k", Align::Right)]);
    for (label, n) in [
        ("echoes", m.echoes),
        ("adverbs", m.adverbs),
        ("filter words", m.filter),
        ("hedges", m.hedges),
        ("cliches", m.cliches),
        ("tics", m.tics),
        ("passive", m.passive),
        ("nominal", m.nominal),
        ("overused", m.overused),
        ("repeated openers", m.repeated_starter_runs),
        ("monotonous runs", m.monotonous_runs),
        ("long sentences", m.long_sentences),
    ] {
        t.row(vec![format!("  {label}"), n.to_string(), format!("{:.1}", m.per_1k(n))]);
    }
    t.print();

    let list = |title: &str, rows: &[(String, usize)]| {
        if !rows.is_empty() {
            println!("\n{title}");
            let joined: Vec<String> = rows.iter().map(|(w, n)| format!("{w} ({n})")).collect();
            println!("  {}", joined.join(", "));
        }
    };
    list("TOP WORDS", &m.top_words);
    list("TOP OPENERS", &m.top_starters);
    list("REPEATED PHRASES", &m.top_repeats);

    if !files.is_empty() {
        println!("\nBY FILE");
        let mut t = Table::new(&[
            ("file", Align::Left),
            ("words", Align::Right),
            ("grade", Align::Right),
            ("mattr", Align::Right),
            ("dialogue", Align::Right),
            ("echoes/1k", Align::Right),
            ("adverbs/1k", Align::Right),
        ]);
        for f in files {
            t.row(vec![
                f.name.clone(),
                thousands(f.counts.words),
                format!("{:.1}", f.readability.flesch_kincaid_grade),
                format!("{:.3}", f.diversity.mattr),
                format!("{:.0}%", f.dialogue.ratio * 100.0),
                format!("{:.1}", f.per_1k(f.echoes)),
                format!("{:.1}", f.per_1k(f.adverbs)),
            ]);
        }
        t.print();
    }
}
