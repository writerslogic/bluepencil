use anyhow::Result;
use bluepencil_core::analysis::rhythm::rhythm;
use bluepencil_core::stats::Summary;
use schemars::JsonSchema;
use serde::Serialize;

use crate::cli::RhythmArgs;
use crate::context::Context;
use crate::output::chart::{bar, sparkline};
use crate::output::human::{file_banner, findings};
use crate::output::{Located, json, locate};

#[derive(Serialize, JsonSchema)]
pub struct RhythmOutput {
    pub file: String,
    pub lengths: Vec<usize>,
    pub summary: Summary,
    pub variation: f64,
    pub findings: Vec<Located>,
}

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
                let findings: Vec<_> = r.findings.iter().map(|f| locate(d, f)).collect();
                RhythmOutput {
                    file: d.name.clone(),
                    lengths: r.lengths.clone(),
                    summary: r.summary,
                    variation: r.variation,
                    findings,
                }
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
