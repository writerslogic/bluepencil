use std::collections::HashMap;

/// Moving-average type-token ratio. Unlike raw TTR it stays comparable across text lengths.
pub fn mattr<S: AsRef<str>>(words: &[S], window: usize) -> f64 {
    let n = words.len();
    if n == 0 {
        return 0.0;
    }
    if n <= window || window == 0 {
        return ttr(words);
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for w in &words[..window] {
        *counts.entry(w.as_ref()).or_default() += 1;
    }
    let mut total = counts.len() as f64;
    for i in window..n {
        let out = words[i - window].as_ref();
        if let Some(c) = counts.get_mut(out) {
            *c -= 1;
            if *c == 0 {
                counts.remove(out);
            }
        }
        *counts.entry(words[i].as_ref()).or_default() += 1;
        total += counts.len() as f64;
    }
    total / ((n - window + 1) as f64 * window as f64)
}

pub fn ttr<S: AsRef<str>>(words: &[S]) -> f64 {
    if words.is_empty() {
        return 0.0;
    }
    let unique: std::collections::HashSet<&str> = words.iter().map(AsRef::as_ref).collect();
    unique.len() as f64 / words.len() as f64
}
