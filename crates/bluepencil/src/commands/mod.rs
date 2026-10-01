mod adverbs;
mod arc;
mod cast;
mod check;
mod cliches;
mod completions;
mod config;
mod continuity;
mod count;
mod dialogue;
mod echoes;
mod facts;
mod filter;
mod freq;
mod hapax;
mod hedges;
mod histogram;
mod init;
mod join;
mod nominal;
mod outline;
mod overused;
mod passive;
mod pov;
mod progress;
mod readability;
mod repeats;
pub mod report;
mod rhythm;
mod scenes;
mod schema;
mod speakers;
mod split;
mod starters;
mod tags;
mod tense;
mod tics;
pub mod trend;
mod unique;
mod voice;
mod wdiff;

use anyhow::Result;
use bluepencil_core::analysis::tally;
use bluepencil_core::{Document, Finding, Lexicons};

use crate::cli::{Cli, Command, ListArgs};
use crate::context::Context;
use crate::exit::Status;
use crate::explain;
use crate::output::{self, human};

pub fn run(cli: Cli) -> Result<Status> {
    match cli.command {
        Command::Init { force, genre } => return init::run(force, genre),
        Command::Completions { shell } => return completions::run(shell),
        Command::Schema { command } => return schema::run(&command),
        _ => {}
    }
    let ctx = Context::new(&cli.global)?;
    if ctx.explain && !supports_explain(&cli.command) {
        anyhow::bail!(
            "--explain is not supported for this command yet (it works with the word-list commands and `echoes`)"
        );
    }
    match cli.command {
        Command::Count(a) => count::run(&ctx, &a),
        Command::Outline(a) => outline::run(&ctx, &a),
        Command::Histogram(a) => histogram::run(&ctx, &a),
        Command::Rhythm(a) => rhythm::run(&ctx, &a),
        Command::Freq(a) => freq::run(&ctx, &a),
        Command::Unique(a) => unique::run(&ctx, &a),
        Command::Hapax(a) => hapax::run(&ctx, &a),
        Command::Repeats(a) => repeats::run(&ctx, &a),
        Command::Echoes(a) => echoes::run(&ctx, &a),
        Command::Starters(a) => starters::run(&ctx, &a),
        Command::Tics(a) => tics::run(&ctx, &a),
        Command::Filter(a) => filter::run(&ctx, &a),
        Command::Hedges(a) => hedges::run(&ctx, &a),
        Command::Adverbs(a) => adverbs::run(&ctx, &a),
        Command::Cliches(a) => cliches::run(&ctx, &a),
        Command::Passive(a) => passive::run(&ctx, &a),
        Command::Overused(a) => overused::run(&ctx, &a),
        Command::Wdiff(a) => wdiff::run(&ctx, &a),
        Command::Split(a) => split::run(&ctx, &a),
        Command::Join(a) => join::run(&ctx, &a),
        Command::Progress(a) => progress::run(&ctx, &a),
        Command::Readability(a) => readability::run(&ctx, &a),
        Command::Dialogue(a) => dialogue::run(&ctx, &a),
        Command::Continuity(a) => continuity::run(&ctx, &a),
        Command::Cast(a) => cast::run(&ctx, &a),
        Command::Nominal(a) => nominal::run(&ctx, &a),
        Command::Tags(a) => tags::run(&ctx, &a),
        Command::Speakers(a) => speakers::run(&ctx, &a),
        Command::Tense(a) => tense::run(&ctx, &a),
        Command::Pov(a) => pov::run(&ctx, &a),
        Command::Arc(a) => arc::run(&ctx, &a),
        Command::Facts(a) => facts::run(&ctx, &a),
        Command::Voice(a) => voice::run(&ctx, &a),
        Command::Scenes(a) => scenes::run(&ctx, &a),
        Command::Config => config::run(&ctx),
        Command::Report(a) => report::run(&ctx, &a),
        Command::Check(a) => return check::run(&ctx, &a),
        Command::Init { .. } | Command::Completions { .. } | Command::Schema { .. } => unreachable!(),
    }?;
    Ok(Status::Ok)
}

fn supports_explain(command: &Command) -> bool {
    matches!(
        command,
        Command::Tics(_)
            | Command::Filter(_)
            | Command::Hedges(_)
            | Command::Adverbs(_)
            | Command::Cliches(_)
            | Command::Passive(_)
            | Command::Nominal(_)
            | Command::Echoes(_)
    )
}

type Finder = fn(&Document, &Lexicons) -> Vec<Finding>;

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct ListOutput {
    pub total: usize,
    pub per_1k_words: f64,
    pub summary: Vec<(String, usize)>,
    pub findings: Vec<output::Located>,
}

/// Shared driver for commands that locate words or phrases from a list.
pub(crate) fn list_command(ctx: &Context, args: &ListArgs, find: Finder, label: &str) -> Result<()> {
    let docs = ctx.documents(&args.input)?;
    let per_doc: Vec<Vec<Finding>> = docs.iter().map(|d| find(d, &ctx.lexicons)).collect();
    let total_words: usize = docs.iter().map(Document::word_count).sum();
    let all: Vec<Finding> = per_doc.iter().flatten().cloned().collect();
    let items: Vec<(&Document, &Finding)> =
        docs.iter().zip(&per_doc).flat_map(|(d, fs)| fs.iter().map(move |f| (d, f))).collect();
    let notes = if ctx.explain && !args.summary {
        let targets: Vec<_> = items
            .iter()
            .map(|(d, f)| explain::Target { doc: d, span: f.span, rule: f.rule, message: &f.message })
            .collect();
        explain::annotate(ctx, &targets)?
    } else {
        Vec::new()
    };

    if ctx.json {
        let mut findings: Vec<_> = items.iter().map(|(d, f)| output::locate(d, f)).collect();
        for (located, note) in findings.iter_mut().zip(notes) {
            located.note = note;
        }
        return output::json::print(&ListOutput {
            total: all.len(),
            per_1k_words: bluepencil_core::stats::per_thousand(all.len(), total_words),
            summary: tally(&all),
            findings,
        });
    }

    if args.summary {
        human::ranked(&tally(&all), label, ctx.limit, Some(total_words));
    } else if notes.is_empty() {
        human::findings(&items);
    } else {
        human::findings_with_notes(&items, &notes);
    }
    println!(
        "\n{} found, {:.1} per 1,000 words",
        all.len(),
        bluepencil_core::stats::per_thousand(all.len(), total_words)
    );
    Ok(())
}
