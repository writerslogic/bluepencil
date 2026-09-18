use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use bluepencil_core::Document;
use serde::{Deserialize, Serialize};

use crate::commands::report::Metrics;
use crate::config::Config;

const DIR_NAME: &str = ".bluepencil-cache";

#[derive(Serialize, Deserialize)]
struct Entry {
    key: u64,
    metrics: Metrics,
}

/// Best-effort on-disk cache of per-file `Metrics`, keyed on file content plus the config
/// values that feed into their computation. Any I/O failure just disables caching for the
/// run rather than failing `report`, since it is a pure performance optimization.
pub struct Cache {
    dir: Option<PathBuf>,
    config_hash: u64,
}

impl Cache {
    pub fn open(base: &Path, config: &Config) -> Self {
        let dir = base.join(DIR_NAME);
        let ready = std::fs::create_dir_all(&dir).is_ok();
        if ready {
            ensure_gitignored(base);
        }
        Self { dir: ready.then_some(dir), config_hash: config_hash(base, config) }
    }

    pub fn get(&self, doc: &Document, enabled: bool) -> Option<Metrics> {
        let dir = self.dir.as_ref()?;
        if !enabled {
            return None;
        }
        let text = std::fs::read_to_string(path_for(dir, &doc.name)).ok()?;
        let entry: Entry = serde_json::from_str(&text).ok()?;
        (entry.key == self.key_for(doc)).then_some(entry.metrics)
    }

    pub fn put(&self, doc: &Document, metrics: &Metrics) {
        let Some(dir) = &self.dir else { return };
        let entry = Entry { key: self.key_for(doc), metrics: metrics.clone() };
        if let Ok(text) = serde_json::to_string(&entry) {
            let _ = std::fs::write(path_for(dir, &doc.name), text);
        }
    }

    fn key_for(&self, doc: &Document) -> u64 {
        let mut hasher = DefaultHasher::new();
        doc.source.hash(&mut hasher);
        self.config_hash.hash(&mut hasher);
        hasher.finish()
    }
}

/// Hashed by path rather than a sanitized path string, so nested files with the same
/// basename never collide.
fn path_for(dir: &Path, name: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    dir.join(format!("{:016x}.json", hasher.finish()))
}

fn config_hash(base: &Path, config: &Config) -> u64 {
    let mut hasher = DefaultHasher::new();
    format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
        config.echoes, config.repeats, config.rhythm, config.starters, config.tics, config.lexicon
    )
    .hash(&mut hasher);
    for paths in config.lexicon.files.values() {
        for path in paths {
            if let Ok(bytes) = std::fs::read(base.join(path)) {
                bytes.hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

fn ensure_gitignored(base: &Path) {
    let path = base.join(".gitignore");
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            if !text.lines().any(|l| l.trim() == DIR_NAME) {
                let mut updated = text;
                if !updated.is_empty() && !updated.ends_with('\n') {
                    updated.push('\n');
                }
                updated.push_str(DIR_NAME);
                updated.push('\n');
                let _ = std::fs::write(&path, updated);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let _ = std::fs::write(&path, format!("{DIR_NAME}\n"));
        }
        Err(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use bluepencil_core::Format;
    use bluepencil_core::analysis::diversity::diversity;

    use super::*;

    fn sample_metrics() -> Metrics {
        Metrics {
            name: "one.md".to_string(),
            counts: Default::default(),
            readability: Default::default(),
            diversity: diversity(&[], 50),
            dialogue: Default::default(),
            sentence_lengths: Default::default(),
            echoes: 0,
            adverbs: 0,
            filter: 0,
            hedges: 0,
            cliches: 0,
            tics: 0,
            passive: 0,
            overused: 0,
            repeated_starter_runs: 0,
            monotonous_runs: 0,
            long_sentences: 0,
            top_words: Vec::new(),
            top_repeats: Vec::new(),
            top_starters: Vec::new(),
        }
    }

    #[test]
    fn cache_hits_on_unchanged_content_and_misses_on_change() {
        let dir = std::env::temp_dir().join(format!("bluepencil-cache-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cache = Cache::open(&dir, &Config::default());
        let doc = Document::parse("one.md", "One two three.", Format::Markdown);
        let metrics = sample_metrics();

        assert!(cache.get(&doc, true).is_none());
        cache.put(&doc, &metrics);
        let hit = cache.get(&doc, true).expect("fresh entry should hit");
        assert_eq!(hit.name, metrics.name);

        let changed = Document::parse("one.md", "Different words entirely now.", Format::Markdown);
        assert!(cache.get(&changed, true).is_none(), "content change must invalidate");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn path_for_is_stable_and_distinguishes_names() {
        let dir = PathBuf::from("/tmp/whatever");
        let a = path_for(&dir, "chapters/one.md");
        let b = path_for(&dir, "chapters/one.md");
        let c = path_for(&dir, "other/one.md");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
