use anyhow::Result;
use bluepencil_core::analysis::rhythm::rhythm;

use crate::cli::RhythmArgs;
use crate::context::Context;
use crate::output::chart::{bar, sparkline};
use crate::output::human::{file_banner, findings};
use crate::output::{json, locate};

pub fn run(ctx: &Context, args: &RhythmArgs) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let cfg = &ctx.config.rhythm;
    let run = args.run.unwrap_or(cfg.run);
    let tolerance = args.tolerance.unwrap_or(cfg.tolerance);
    let long = args.long.unwrap_or(cfg.long);
    let results: Vec<_> = docs.iter().map(|d| rhythm(d, run, tolerance, long)).collect();

    if ctx.json {
        let out: Vec<_> = docs
            .iter()
            .zip(&results)
            .map(|(d, r)| {
                let f: Vec<_> = r.findings.iter().map(|f| locate(d, f)).collect();
                serde_json::json!({ "file": d.name, "lengths": r.lengths, "summary": r.summary, "variation": r.variation, "findings": f })
            })
            .collect();
        return json::print(&out);
    }

    for (doc, r) in docs.iter().zip(&results) {
        file_banner(&docs, doc);
        let s = r.summary;
        println!("{} sentences, mean {:.1} words, sd {:.1}, variation {:.2}", s.count, s.mean, s.stdev, r.variation);
        if args.bars {
            let max = s.max;
            for (i, (len, sentence)) in r.lengths.iter().zip(doc.sentences()).enumerate() {
                let line = doc.position(sentence.span.start).line;
                println!("{:>5} L{:<6}{:>4} {}", i + 1, line, len, bar(*len, max, 60));
            }
        } else {
            for line in sparkline(&r.lengths, 80) {
                println!("{line}");
            }
        }
        println!();
        let items: Vec<_> = r.findings.iter().map(|f| (doc, f)).collect();
        findings(&items);
    }
    Ok(())
}
