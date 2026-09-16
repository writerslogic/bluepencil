use anyhow::{Context as _, Result};
use bluepencil_core::analysis::counts::Counts;
use bluepencil_core::analysis::dialogue::{self, DialogueRatio};
use bluepencil_core::analysis::diversity::{Diversity, diversity};
use bluepencil_core::analysis::{
    adverbs, cliches, echoes, filter_words, frequency, hedges, readability, repetition, rhythm, starters, tics,
};
use bluepencil_core::stats::readability::Readability;
use bluepencil_core::stats::{Summary, per_thousand};
use bluepencil_core::{Document, Lexicons};
use serde::Serialize;

use crate::cli::ReportArgs;
use crate::context::Context;
use crate::output::human::{minutes, thousands};
use crate::output::table::{Align, Table};
use crate::output::{html, json};

#[derive(Debug, Clone, Serialize)]
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

pub fn run(ctx: &Context, args: &ReportArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let total = measure(ctx, &docs, "all files");
    let files: Vec<Metrics> = if docs.len() > 1 {
        docs.iter().map(|d| measure(ctx, std::slice::from_ref(d), &d.name)).collect()
    } else {
        Vec::new()
    };

    if let Some(path) = &args.html {
        std::fs::write(path, html::report(&total, &files)).with_context(|| format!("writing {}", path.display()))?;
        if !ctx.json {
            println!("Wrote {}\n", path.display());
        }
    }
    if ctx.json {
        return json::print(&serde_json::json!({ "total": total, "files": files }));
    }
    print_human(&total, &files);
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
