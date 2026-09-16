use super::Parsed;

pub fn parse(source: &str) -> Parsed {
    Parsed { prose: source.to_string(), headings: Vec::new(), breaks: Vec::new(), dialogue: None }
}
