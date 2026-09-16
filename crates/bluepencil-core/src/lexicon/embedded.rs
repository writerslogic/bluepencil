pub const STOPWORDS: &str = include_str!("../../data/stopwords.txt");
pub const FILTER_WORDS: &str = include_str!("../../data/filter_words.txt");
pub const HEDGES: &str = include_str!("../../data/hedges.txt");
pub const CLICHES: &str = include_str!("../../data/cliches.txt");
pub const DIALOGUE_TAGS: &str = include_str!("../../data/dialogue_tags.txt");
pub const SAID_BOOKISMS: &str = include_str!("../../data/said_bookisms.txt");
pub const NOMINALIZATIONS: &str = include_str!("../../data/nominalizations.tsv");
pub const ABBREVIATIONS: &str = include_str!("../../data/abbreviations.txt");
pub const IRREGULAR_PARTICIPLES: &str = include_str!("../../data/irregular_participles.txt");
pub const SYLLABLE_EXCEPTIONS: &str = include_str!("../../data/syllable_exceptions.tsv");
pub const NOT_ADVERBS: &str = include_str!("../../data/not_adverbs.txt");

/// Non-empty, non-comment lines of an embedded list.
pub fn lines(src: &'static str) -> impl Iterator<Item = &'static str> {
    src.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#'))
}
