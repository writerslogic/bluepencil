use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

const LONG_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), env!("BLUEPENCIL_GIT_HASH"));

#[derive(Parser)]
#[command(name = "bluepencil", version, long_version = LONG_VERSION, about = "Prose analysis for writers", propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[command(flatten)]
    pub global: Global,
}

#[derive(Args)]
pub struct Global {
    /// Print machine-readable JSON
    #[arg(long, global = true)]
    pub json: bool,
    /// Use this config file instead of searching for bluepencil.toml
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,
    /// Treat input as this format regardless of file extension
    #[arg(long = "as", global = true, value_enum, value_name = "FORMAT")]
    pub input_format: Option<InputFormat>,
    /// Show at most this many rows in ranked lists
    #[arg(long, short = 'n', global = true, default_value_t = 25)]
    pub limit: usize,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum InputFormat {
    Plain,
    Markdown,
    Fountain,
}

#[derive(Args, Clone)]
pub struct Input {
    /// Files or glob patterns; `-` reads stdin. Defaults to `project.files` in bluepencil.toml
    pub files: Vec<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Words, characters, sentences, paragraphs, and reading time
    Count(Input),
    /// Heading tree with word counts per section
    Outline(Input),
    /// Sentence and paragraph length distributions
    Histogram(HistogramArgs),
    /// Sentence length flow, monotonous runs, and overlong sentences
    Rhythm(RhythmArgs),
    /// Word frequency across all inputs
    Freq(FreqArgs),
    /// Lexical diversity (TTR and MATTR)
    Unique(UniqueArgs),
    /// Words used exactly once
    Hapax(Input),
    /// Repeated phrases
    Repeats(RepeatsArgs),
    /// The same word reappearing too soon
    Echoes(EchoesArgs),
    /// Sentence and paragraph openers, and runs of the same opener
    Starters(StartersArgs),
    /// Your personal crutch words from bluepencil.toml
    Tics(ListArgs),
    /// Filter words that distance the reader (felt, saw, noticed)
    Filter(ListArgs),
    /// Hedges and intensifiers (just, really, very)
    Hedges(ListArgs),
    /// -ly adverbs
    Adverbs(ListArgs),
    /// Stock phrases and cliches
    Cliches(ListArgs),
    /// Passive voice constructions (be-verb + past participle)
    Passive(ListArgs),
    /// Words used far more than in general English usage
    Overused(OverusedArgs),
    /// Readability scores for each document and section
    Readability(Input),
    /// Dialogue versus narration
    Dialogue(Input),
    /// Everything at a glance
    Report(ReportArgs),
    /// Fail when configured thresholds are exceeded (for CI and hooks)
    Check(CheckArgs),
    /// Write a starter bluepencil.toml
    Init {
        /// Overwrite an existing file
        #[arg(long)]
        force: bool,
    },
    /// Print shell completions
    Completions { shell: Shell },
    /// Word-level diff between two versions of the same text
    Wdiff(WdiffArgs),
    /// Split a manuscript into one file per heading
    Split(SplitArgs),
    /// Concatenate manuscript files into one
    Join(JoinArgs),
}

#[derive(Args)]
pub struct SplitArgs {
    /// The manuscript to split
    pub path: String,
    /// Heading level to split at (1 = #, 2 = ##, ...)
    #[arg(long, default_value_t = 1)]
    pub level: u8,
    /// Directory to write the split files into (defaults to a directory named after the input file)
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args)]
pub struct JoinArgs {
    #[command(flatten)]
    pub input: Input,
    /// Write to this file instead of stdout
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args)]
pub struct WdiffArgs {
    /// The earlier version; `-` reads stdin
    pub old: String,
    /// The later version; `-` reads stdin
    pub new: String,
}

#[derive(Args)]
pub struct HistogramArgs {
    #[command(flatten)]
    pub input: Input,
    /// What to measure
    #[arg(long, value_enum, default_value_t = HistogramKind::Sentences)]
    pub of: HistogramKind,
    /// Bucket width
    #[arg(long, default_value_t = 5)]
    pub width: usize,
    /// Values at or above this share the last bucket
    #[arg(long, default_value_t = 50)]
    pub cap: usize,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum HistogramKind {
    /// Words per sentence
    Sentences,
    /// Words per paragraph
    Paragraphs,
    /// Sentences per paragraph
    ParagraphSentences,
}

#[derive(Args)]
pub struct RhythmArgs {
    #[command(flatten)]
    pub input: Input,
    /// Flag runs of at least this many similar-length sentences
    #[arg(long)]
    pub run: Option<usize>,
    /// Maximum spread in words for a run to count as monotonous
    #[arg(long)]
    pub tolerance: Option<usize>,
    /// Flag sentences longer than this
    #[arg(long)]
    pub long: Option<usize>,
    /// Show every sentence length as a bar
    #[arg(long)]
    pub bars: bool,
}

#[derive(Args)]
pub struct FreqArgs {
    #[command(flatten)]
    pub input: Input,
    /// Include common function words
    #[arg(long)]
    pub all: bool,
    /// Ignore words shorter than this
    #[arg(long, default_value_t = 1)]
    pub min_length: usize,
}

#[derive(Args)]
pub struct CheckArgs {
    #[command(flatten)]
    pub input: Input,
    /// Only evaluate line-scoped rules (echoes, adverbs, filter, hedges, tics, passive,
    /// cliches, repeated openers, monotonous runs, long sentences) against lines changed
    /// since this git ref; whole-document rules (overused, grade, mattr, dialogue ratio)
    /// are skipped rather than evaluated against the whole file under a diff-scoped name.
    #[arg(long, value_name = "GIT_REF")]
    pub since: Option<String>,
}

#[derive(Args)]
pub struct OverusedArgs {
    #[command(flatten)]
    pub input: Input,
    /// Flag a word once it's used at least this many times more than baseline
    #[arg(long, default_value_t = 3.0)]
    pub min_ratio: f64,
    /// Ignore words seen fewer than this many times
    #[arg(long, default_value_t = 3)]
    pub min_count: usize,
}

#[derive(Args)]
pub struct UniqueArgs {
    #[command(flatten)]
    pub input: Input,
    /// MATTR window size in words
    #[arg(long, default_value_t = 50)]
    pub window: usize,
}

#[derive(Args)]
pub struct RepeatsArgs {
    #[command(flatten)]
    pub input: Input,
    /// Shortest phrase length in words
    #[arg(long)]
    pub min: Option<usize>,
    /// Longest phrase length in words
    #[arg(long)]
    pub max: Option<usize>,
    /// Minimum occurrences to report
    #[arg(long)]
    pub count: Option<usize>,
    /// List every location
    #[arg(long, short)]
    pub locations: bool,
}

#[derive(Args)]
pub struct EchoesArgs {
    #[command(flatten)]
    pub input: Input,
    /// Words between uses that still count as an echo
    #[arg(long, short)]
    pub window: Option<usize>,
    /// Ignore words shorter than this
    #[arg(long)]
    pub min_length: Option<usize>,
    /// Include names and other always-capitalized words
    #[arg(long)]
    pub names: bool,
    /// Show a summary by word instead of each location
    #[arg(long)]
    pub summary: bool,
}

#[derive(Args)]
pub struct StartersArgs {
    #[command(flatten)]
    pub input: Input,
    /// Flag runs of at least this many sentences with the same opener
    #[arg(long)]
    pub run: Option<usize>,
}

#[derive(Args)]
pub struct ListArgs {
    #[command(flatten)]
    pub input: Input,
    /// Show a summary by word instead of each location
    #[arg(long)]
    pub summary: bool,
}

#[derive(Args)]
pub struct ReportArgs {
    #[command(flatten)]
    pub input: Input,
    /// Also write a standalone HTML report
    #[arg(long, value_name = "PATH")]
    pub html: Option<PathBuf>,
}
