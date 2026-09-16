use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Bucket {
    pub label: String,
    pub low: usize,
    pub high: Option<usize>,
    pub count: usize,
}

/// Buckets of `width`, with everything at or above `cap` gathered into a final open bucket.
pub fn buckets(values: &[usize], width: usize, cap: usize) -> Vec<Bucket> {
    let width = width.max(1);
    let mut out: Vec<Bucket> = (0..cap.div_ceil(width))
        .map(|i| {
            let low = i * width;
            let high = (low + width).min(cap);
            let label = if width == 1 { format!("{low}") } else { format!("{low}-{}", high - 1) };
            Bucket { label, low, high: Some(high), count: 0 }
        })
        .collect();
    out.push(Bucket { label: format!("{cap}+"), low: cap, high: None, count: 0 });
    for &v in values {
        let i = if v >= cap { out.len() - 1 } else { v / width };
        out[i].count += 1;
    }
    while out.first().is_some_and(|b| b.count == 0) {
        out.remove(0);
    }
    while out.len() > 1 && out.last().is_some_and(|b| b.count == 0) {
        out.pop();
    }
    out
}
