use crate::commands::report::Metrics;
use crate::commands::trend::TrendPoint;

const TEMPLATE: &str = include_str!("../../templates/report.html");
const CSS: &str = include_str!("../../templates/report.css");

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn stat(label: &str, value: String) -> String {
    format!(
        "<div class=\"stat\"><span class=\"value\">{}</span><span class=\"label\">{}</span></div>",
        esc(&value),
        esc(label)
    )
}

fn chips(rows: &[(String, usize)]) -> String {
    if rows.is_empty() {
        return "<p class=\"muted\">None</p>".into();
    }
    let items: Vec<String> = rows.iter().map(|(w, n)| format!("<li>{} <b>{n}</b></li>", esc(w))).collect();
    format!("<ul class=\"chips\">{}</ul>", items.join(""))
}

/// Maps `values` onto an SVG polyline's `points` attribute, `width`x`height` with `pad`
/// margin on every side, normalized against `values`' own min/max (a flat line at mid-height
/// when every value is equal, rather than dividing by zero).
fn polyline(values: &[f64], width: f64, height: f64, pad: f64) -> String {
    let (min, max) = values.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let span = (max - min).max(f64::EPSILON);
    let step = if values.len() > 1 { (width - 2.0 * pad) / (values.len() - 1) as f64 } else { 0.0 };
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = pad + step * i as f64;
            let y = if max > min { height - pad - (v - min) / span * (height - 2.0 * pad) } else { height / 2.0 };
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn trend_chart(points: &[TrendPoint]) -> String {
    if points.len() < 2 {
        return String::new();
    }
    const W: f64 = 680.0;
    const H: f64 = 140.0;
    const PAD: f64 = 8.0;

    let words: Vec<f64> = points.iter().map(|p| p.words as f64).collect();
    let words_line = polyline(&words, W, H, PAD);
    let words_svg = format!(
        "<svg viewBox=\"0 0 {W} {H}\" preserveAspectRatio=\"none\" role=\"img\" aria-label=\"Word count over time\"><polyline points=\"{words_line}\" fill=\"none\" style=\"stroke:var(--accent)\" stroke-width=\"2\"/></svg>"
    );

    let series = [
        ("Echoes", "var(--accent)", points.iter().map(|p| p.echoes_per_1k).collect::<Vec<_>>()),
        ("Adverbs", "#c2652b", points.iter().map(|p| p.adverbs_per_1k).collect::<Vec<_>>()),
        ("Cliches", "#8a5fc9", points.iter().map(|p| p.cliches_per_1k).collect::<Vec<_>>()),
    ];
    let flag_lines: String = series
        .iter()
        .map(|(_, color, values)| {
            let line = polyline(values, W, H, PAD);
            format!("<polyline points=\"{line}\" fill=\"none\" style=\"stroke:{color}\" stroke-width=\"2\"/>")
        })
        .collect();
    let flags_svg = format!(
        "<svg viewBox=\"0 0 {W} {H}\" preserveAspectRatio=\"none\" role=\"img\" aria-label=\"Style flags per 1,000 words over time\">{flag_lines}</svg>"
    );
    let legend: String = series
        .iter()
        .map(|(label, color, _)| format!("<li><span class=\"swatch\" style=\"background:{color}\"></span>{label}</li>"))
        .collect();

    let first = esc(&points[0].label);
    let last = esc(&points[points.len() - 1].label);
    format!(
        "<section><h2>Trend</h2>\
        <div class=\"trend\"><p class=\"muted\">Word count, {first} to {last}</p>{words_svg}</div>\
        <div class=\"trend\"><p class=\"muted\">Style flags per 1,000 words</p>{flags_svg}<ul class=\"legend\">{legend}</ul></div>\
        </section>"
    )
}

pub fn report(m: &Metrics, files: &[Metrics], trend: &[TrendPoint]) -> String {
    let c = &m.counts;
    let r = &m.readability;
    let overview = [
        stat("words", c.words.to_string()),
        stat("sentences", c.sentences.to_string()),
        stat("paragraphs", c.paragraphs.to_string()),
        stat("minutes to read", format!("{:.0}", c.reading_minutes)),
        stat("reading ease", format!("{:.1}", r.flesch_reading_ease)),
        stat("grade level", format!("{:.1}", r.flesch_kincaid_grade)),
        stat("MATTR", format!("{:.3}", m.diversity.mattr)),
        stat("dialogue", format!("{:.0}%", m.dialogue.ratio * 100.0)),
    ]
    .join("");

    let flags: Vec<(&str, usize)> = vec![
        ("Echoes", m.echoes),
        ("Adverbs", m.adverbs),
        ("Filter words", m.filter),
        ("Hedges", m.hedges),
        ("Cliches", m.cliches),
        ("Tics", m.tics),
        ("Repeated openers", m.repeated_starter_runs),
        ("Monotonous runs", m.monotonous_runs),
        ("Long sentences", m.long_sentences),
    ];
    let max = flags.iter().map(|(_, n)| m.per_1k(*n)).fold(0.0, f64::max).max(1.0);
    let flag_rows: String = flags
        .iter()
        .map(|(label, n)| {
            let rate = m.per_1k(*n);
            format!(
                "<tr><td>{label}</td><td class=\"num\">{n}</td><td class=\"num\">{rate:.1}</td><td><div class=\"bar\" style=\"width:{:.0}%\"></div></td></tr>",
                rate / max * 100.0
            )
        })
        .collect();

    let file_rows: String = files
        .iter()
        .map(|f| {
            format!(
                "<tr><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{:.1}</td><td class=\"num\">{:.3}</td><td class=\"num\">{:.0}%</td><td class=\"num\">{:.1}</td><td class=\"num\">{:.1}</td></tr>",
                esc(&f.name),
                f.counts.words,
                f.readability.flesch_kincaid_grade,
                f.diversity.mattr,
                f.dialogue.ratio * 100.0,
                f.per_1k(f.echoes),
                f.per_1k(f.adverbs)
            )
        })
        .collect();
    let by_file = if files.is_empty() {
        String::new()
    } else {
        format!(
            "<section><h2>By file</h2><div class=\"scroll\"><table><thead><tr><th>File</th><th>Words</th><th>Grade</th><th>MATTR</th><th>Dialogue</th><th>Echoes/1k</th><th>Adverbs/1k</th></tr></thead><tbody>{file_rows}</tbody></table></div></section>"
        )
    };

    TEMPLATE
        .replace("{{css}}", CSS)
        .replace("{{version}}", env!("CARGO_PKG_VERSION"))
        .replace("{{trend}}", &trend_chart(trend))
        .replace("{{overview}}", &overview)
        .replace("{{flags}}", &flag_rows)
        .replace("{{top_words}}", &chips(&m.top_words))
        .replace("{{top_starters}}", &chips(&m.top_starters))
        .replace("{{top_repeats}}", &chips(&m.top_repeats))
        .replace("{{by_file}}", &by_file)
}
