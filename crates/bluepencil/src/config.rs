use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use serde::Deserialize;

pub const FILE_NAME: &str = "bluepencil.toml";
pub const TEMPLATE: &str = include_str!("../../../bluepencil.example.toml");

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub project: Project,
    pub echoes: Echoes,
    pub repeats: Repeats,
    pub rhythm: Rhythm,
    pub starters: Starters,
    pub tics: Tics,
    pub lexicon: Lexicon,
    pub check: Check,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Project {
    pub files: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Echoes {
    pub window: usize,
    pub min_length: usize,
    pub include_names: bool,
    pub ignore: Vec<String>,
}

impl Default for Echoes {
    fn default() -> Self {
        Self { window: 50, min_length: 4, include_names: false, ignore: Vec::new() }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Repeats {
    pub min: usize,
    pub max: usize,
    pub count: usize,
}

impl Default for Repeats {
    fn default() -> Self {
        Self { min: 3, max: 6, count: 2 }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rhythm {
    pub run: usize,
    pub tolerance: usize,
    pub long: usize,
}

impl Default for Rhythm {
    fn default() -> Self {
        Self { run: 4, tolerance: 3, long: 40 }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Starters {
    pub run: usize,
}

impl Default for Starters {
    fn default() -> Self {
        Self { run: 3 }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Tics {
    pub words: Vec<String>,
}

/// Additions to and removals from the built-in word lists.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Lexicon {
    pub stopwords: Vec<String>,
    pub filter: Vec<String>,
    pub hedges: Vec<String>,
    pub cliches: Vec<String>,
    pub not_adverbs: Vec<String>,
    /// Removed from every list.
    pub ignore: Vec<String>,
    /// Extra list files, keyed by list name.
    pub files: std::collections::BTreeMap<String, Vec<PathBuf>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Check {
    pub max_echoes_per_1k: Option<f64>,
    pub max_adverbs_per_1k: Option<f64>,
    pub max_filter_per_1k: Option<f64>,
    pub max_hedges_per_1k: Option<f64>,
    pub max_tics_per_1k: Option<f64>,
    pub max_passive_per_1k: Option<f64>,
    pub max_cliches: Option<usize>,
    pub max_repeated_starter_runs: Option<usize>,
    pub max_monotonous_runs: Option<usize>,
    pub max_sentence_words: Option<usize>,
    pub min_mattr: Option<f64>,
    pub max_grade: Option<f64>,
    pub min_dialogue_ratio: Option<f64>,
    pub max_dialogue_ratio: Option<f64>,
}

/// Loads the explicit config or the nearest `bluepencil.toml` above the working directory.
pub fn load(explicit: Option<&Path>) -> Result<(Config, PathBuf)> {
    let cwd = std::env::current_dir()?;
    let path = match explicit {
        Some(p) => Some(p.to_path_buf()),
        None => cwd.ancestors().map(|d| d.join(FILE_NAME)).find(|p| p.is_file()),
    };
    let Some(path) = path else {
        return Ok((Config::default(), cwd));
    };
    let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let config = toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    let base = path.parent().map_or(cwd, Path::to_path_buf);
    Ok((config, base))
}
