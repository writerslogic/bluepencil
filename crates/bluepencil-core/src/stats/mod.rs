pub mod histogram;
pub mod mattr;
pub mod ngram;
pub mod readability;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Summary {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub stdev: f64,
    pub min: usize,
    pub max: usize,
}

impl Summary {
    pub fn of(values: &[usize]) -> Self {
        if values.is_empty() {
            return Self::default();
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        let n = sorted.len();
        let mean = sorted.iter().sum::<usize>() as f64 / n as f64;
        let median =
            if n.is_multiple_of(2) { (sorted[n / 2 - 1] + sorted[n / 2]) as f64 / 2.0 } else { sorted[n / 2] as f64 };
        let var = sorted.iter().map(|&v| (v as f64 - mean).powi(2)).sum::<f64>() / n as f64;
        Self { count: n, mean, median, stdev: var.sqrt(), min: sorted[0], max: sorted[n - 1] }
    }
}

pub fn per_thousand(count: usize, words: usize) -> f64 {
    if words == 0 { 0.0 } else { count as f64 * 1000.0 / words as f64 }
}
