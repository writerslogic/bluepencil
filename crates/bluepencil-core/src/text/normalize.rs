/// Lowercases and maps typographic apostrophes to ASCII so `don’t` and `don't` match.
pub fn fold(word: &str) -> String {
    word.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            '\u{2019}' | '\u{2018}' | '\u{02BC}' => '\'',
            '\u{2010}' | '\u{2011}' => '-',
            c => c,
        })
        .collect()
}
