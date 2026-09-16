#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

pub struct Table {
    headers: Vec<String>,
    align: Vec<Align>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(headers: &[(&str, Align)]) -> Self {
        Self {
            headers: headers.iter().map(|(h, _)| h.to_string()).collect(),
            align: headers.iter().map(|(_, a)| *a).collect(),
            rows: Vec::new(),
        }
    }

    pub fn row(&mut self, cells: Vec<String>) {
        self.rows.push(cells);
    }

    pub fn print(&self) {
        let width = |s: &str| s.chars().count();
        let mut widths: Vec<usize> = self.headers.iter().map(|h| width(h)).collect();
        for row in &self.rows {
            for (i, c) in row.iter().enumerate() {
                widths[i] = widths[i].max(width(c));
            }
        }
        let render = |cells: &[String]| {
            let last = cells.len().saturating_sub(1);
            cells
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let pad = " ".repeat(widths[i] - width(c));
                    match (self.align[i], i == last) {
                        (Align::Right, _) => format!("{pad}{c}"),
                        (Align::Left, true) => c.clone(),
                        (Align::Left, false) => format!("{c}{pad}"),
                    }
                })
                .collect::<Vec<_>>()
                .join("  ")
        };
        println!("{}", render(&self.headers));
        println!("{}", render(&widths.iter().map(|w| "─".repeat(*w)).collect::<Vec<_>>()));
        for row in &self.rows {
            println!("{}", render(row));
        }
    }
}
