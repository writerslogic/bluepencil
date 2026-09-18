use std::collections::HashMap;

use serde::Serialize;

use super::echoes::proper_nouns;
use crate::document::Document;

#[derive(Debug, Clone, Serialize)]
pub struct NameVariant {
    pub spelling: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct NameCluster {
    pub variants: Vec<NameVariant>,
}

/// Proper nouns across `docs` likely to be the same name spelled inconsistently: a single
/// edit apart for a name up to 7 letters ("Sara"/"Sarah"), two for anything longer, since a
/// longer name has more room for an edit that's still recognizably the same word. Names of 3
/// letters or fewer never cluster: at that length a single edit turns one real name into
/// another real name too often to be useful ("Jon"/"Ron").
///
/// `ignore` names are dropped before clustering, for a pair of genuinely distinct characters
/// this happens to flag.
pub fn name_variants(docs: &[Document], ignore: &std::collections::HashSet<String>) -> Vec<NameCluster> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for doc in docs {
        let names = proper_nouns(doc);
        for w in doc.words() {
            if names.contains(w.lower.as_str()) && !ignore.contains(&w.lower) {
                *counts.entry(w.lower.clone()).or_default() += 1;
            }
        }
    }

    let mut names: Vec<String> = counts.keys().cloned().collect();
    names.sort();

    let mut clusters: Vec<Vec<String>> = Vec::new();
    for name in names {
        if name.chars().count() <= 3 {
            clusters.push(vec![name]);
            continue;
        }
        let threshold = if name.chars().count() > 7 { 2 } else { 1 };
        // `all`, not `any`: matching only the closest existing member lets a bridge name chain
        // together spellings that aren't themselves close ("lana"/"lano"/"lena" would otherwise
        // merge into one cluster through "lano", even though "lana" and "lena" are 2 edits apart).
        match clusters.iter_mut().find(|c| {
            c.iter().all(|existing| existing.chars().count() > 3 && edit_distance(existing, &name) <= threshold)
        }) {
            Some(cluster) => cluster.push(name),
            None => clusters.push(vec![name]),
        }
    }

    let mut out: Vec<NameCluster> = clusters
        .into_iter()
        .filter(|c| c.len() > 1)
        .map(|c| {
            let mut variants: Vec<NameVariant> =
                c.into_iter().map(|spelling| NameVariant { count: counts[&spelling], spelling }).collect();
            variants.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.spelling.cmp(&b.spelling)));
            NameCluster { variants }
        })
        .collect();
    // Highest total usage first, so a 200-vs-1 spelling drift outranks a one-off coincidence
    // between two rare names.
    out.sort_by(|a, b| {
        let total = |c: &NameCluster| c.variants.iter().map(|v| v.count).sum::<usize>();
        total(b).cmp(&total(a)).then_with(|| a.variants[0].spelling.cmp(&b.variants[0].spelling))
    });
    out
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut curr = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            curr[j] = if a[i - 1] == b[j - 1] { prev[j - 1] } else { 1 + prev[j - 1].min(prev[j]).min(curr[j - 1]) };
        }
        prev = curr;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Format;

    fn doc(text: &str) -> Document {
        Document::parse("test.md".to_string(), text.to_string(), Format::Markdown)
    }

    fn names(docs: &[Document]) -> Vec<Vec<String>> {
        let mut clusters: Vec<Vec<String>> = name_variants(docs, &std::collections::HashSet::new())
            .into_iter()
            .map(|c| c.variants.into_iter().map(|v| v.spelling).collect())
            .collect();
        clusters.sort();
        clusters
    }

    #[test]
    fn flags_a_one_letter_spelling_drift() {
        let docs = [doc("I saw Sara today. I saw Sara again. I met Sarah once. I met Sarah twice.")];
        assert_eq!(names(&docs), vec![vec!["sara".to_string(), "sarah".to_string()]]);
    }

    #[test]
    fn does_not_flag_unrelated_names() {
        let docs = [doc("I met Elaine and Marcus. I saw Elaine leave. I saw Marcus stay.")];
        assert!(names(&docs).is_empty());
    }

    #[test]
    fn short_names_never_cluster() {
        let docs = [doc("I called Jon and Ron. I saw Jon leave. I saw Ron stay.")];
        assert!(names(&docs).is_empty());
    }

    #[test]
    fn ignore_list_suppresses_a_cluster() {
        let docs = [doc("I saw Sara today. I saw Sara again. I met Sarah once. I met Sarah twice.")];
        let ignore: std::collections::HashSet<String> = ["sara".to_string()].into_iter().collect();
        assert!(name_variants(&docs, &ignore).is_empty());
    }

    #[test]
    fn clusters_across_multiple_documents() {
        let docs = [doc("I saw Sara today. I saw Sara again."), doc("I met Sarah once. I met Sarah twice.")];
        assert_eq!(names(&docs), vec![vec!["sara".to_string(), "sarah".to_string()]]);
    }

    #[test]
    fn does_not_chain_through_a_bridge_name() {
        // "lana"/"lano" and "lano"/"lena" are each 1 edit apart, but "lana"/"lena" is 2: without
        // requiring a new name to match every existing cluster member (not just the one it's
        // closest to), "lano" would bridge "lana" and "lena" into one incorrect cluster.
        let docs = [doc("I saw Lana and Lano and Lena today. I saw Lana again. I saw Lano again. I saw Lena again.")];
        assert_eq!(names(&docs), vec![vec!["lana".to_string(), "lano".to_string()]]);
    }

    #[test]
    fn ranks_the_most_used_cluster_first() {
        let mut text = String::new();
        for _ in 0..20 {
            text.push_str("I saw Marcus today. ");
        }
        text.push_str("I saw Marcos once. I saw Elana today. I saw Elena once.");
        let docs = [doc(&text)];
        let clusters = name_variants(&docs, &std::collections::HashSet::new());
        assert_eq!(clusters.len(), 2);
        let total: usize = clusters[0].variants.iter().map(|v| v.count).sum();
        assert!(clusters[0].variants.iter().any(|v| v.spelling == "marcus"), "expected marcus cluster first");
        assert!(total > 20, "expected the high-usage cluster ranked first: {clusters:?}");
    }
}
