const EIGHTHS: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
const SPARKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Horizontal bar scaled so `max` fills `width` cells.
pub fn bar(value: usize, max: usize, width: usize) -> String {
    if max == 0 || value == 0 {
        return String::new();
    }
    let eighths = (value * width * 8).div_ceil(max);
    let mut s = "█".repeat(eighths / 8);
    if !eighths.is_multiple_of(8) {
        s.push(EIGHTHS[eighths % 8 - 1]);
    }
    s
}

/// Sparkline wrapped to `width` characters per line.
pub fn sparkline(values: &[usize], width: usize) -> Vec<String> {
    let max = values.iter().copied().max().unwrap_or(0).max(1);
    let line: Vec<char> = values.iter().map(|&v| SPARKS[(v * 7).div_ceil(max).min(7)]).collect();
    line.chunks(width.max(1)).map(|c| c.iter().collect()).collect()
}
