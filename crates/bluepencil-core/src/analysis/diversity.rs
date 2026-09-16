use std::collections::HashSet;

use serde::Serialize;

use crate::stats::mattr::{mattr, ttr};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Diversity {
    pub words: usize,
    pub unique: usize,
    pub ttr: f64,
    pub mattr: f64,
    pub window: usize,
}

pub fn diversity(words: &[&str], window: usize) -> Diversity {
    Diversity {
        words: words.len(),
        unique: words.iter().collect::<HashSet<_>>().len(),
        ttr: ttr(words),
        mattr: mattr(words, window),
        window,
    }
}
