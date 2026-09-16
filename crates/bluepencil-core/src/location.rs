use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(text.bytes().enumerate().filter(|(_, b)| *b == b'\n').map(|(i, _)| i + 1));
        Self { starts }
    }

    /// One-based line and column (columns count characters, not bytes).
    pub fn position(&self, text: &str, offset: usize) -> Position {
        let line = self.starts.partition_point(|&s| s <= offset) - 1;
        let column = text[self.starts[line]..offset].chars().count() + 1;
        Position { line: line + 1, column }
    }
}
