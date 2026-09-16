use anyhow::Result;
use bluepencil_core::analysis::distribution::{self, distribution};

use crate::cli::{HistogramArgs, HistogramKind};
use crate::context::Context;
use crate::output::chart::bar;
use crate::output::json;

pub fn run(ctx: &Context, args: &HistogramArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let values: Vec<usize> = docs
        .iter()
        .flat_map(|d| match args.of {
            HistogramKind::Sentences => distribution::sentence_lengths(d),
            HistogramKind::Paragraphs => distribution::paragraph_lengths(d),
            HistogramKind::ParagraphSentences => distribution::paragraph_sentence_counts(d),
        })
        .collect();
    let (width, cap) = match args.of {
        HistogramKind::ParagraphSentences if args.width == 5 && args.cap == 50 => (1, 12),
        _ => (args.width, args.cap),
    };
    let dist = distribution(&values, width, cap);
    if ctx.json {
        return json::print(&dist);
    }
    let unit = match args.of {
        HistogramKind::Sentences => "words per sentence",
        HistogramKind::Paragraphs => "words per paragraph",
        HistogramKind::ParagraphSentences => "sentences per paragraph",
    };
    let s = dist.summary;
    println!(
        "{unit}: n={} mean={:.1} median={:.1} sd={:.1} min={} max={}\n",
        s.count, s.mean, s.median, s.stdev, s.min, s.max
    );
    let max = dist.buckets.iter().map(|b| b.count).max().unwrap_or(0);
    let label_w = dist.buckets.iter().map(|b| b.label.len()).max().unwrap_or(0);
    for b in &dist.buckets {
        println!("{:>label_w$}  {:>6}  {}", b.label, b.count, bar(b.count, max, 50));
    }
    Ok(())
}
