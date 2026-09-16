use crate::commands::report::Metrics;

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

pub fn report(m: &Metrics, files: &[Metrics]) -> String {
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
        .replace("{{overview}}", &overview)
        .replace("{{flags}}", &flag_rows)
        .replace("{{top_words}}", &chips(&m.top_words))
        .replace("{{top_starters}}", &chips(&m.top_starters))
        .replace("{{top_repeats}}", &chips(&m.top_repeats))
        .replace("{{by_file}}", &by_file)
}
