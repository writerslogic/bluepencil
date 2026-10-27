use super::*;
use bluepencil_core::{Format, Lexicons, analysis::filter_words};

fn view(source: &str, scope: &str, detail: &str) -> Chart {
    let doc = Document::parse("test.md", source, Format::Markdown);
    let index = SourceIndex::new(source);
    ChartData::new(&doc, &[]).view(source, &index, Span::new(0, source.len()), scope, detail).unwrap()
}

#[test]
fn sentence_and_paragraph_detail_represent_real_text_units() {
    let source = "Rain fell. The heavy rain fell again.\n\n\"Run,\" she said.";
    let sentences = view(source, "visible", "sentence");
    assert_eq!(sentences.points.len(), 3);
    assert_eq!(sentences.points.iter().map(|point| point.sentence_length).collect::<Vec<_>>(), [2.0, 5.0, 3.0]);
    assert_eq!(sentences.points[1].paragraph_length, 7.0);
    assert!((sentences.points[2].dialogue_share - 100.0 / 3.0).abs() < 0.001);
    assert!(sentences.points[1].sentence_variation > 0.0);
    let paragraphs = view(source, "visible", "paragraph");
    assert_eq!(paragraphs.points.len(), 2);
    assert_eq!(paragraphs.points[0].sentence_length, 3.5);
    assert_eq!(paragraphs.points[0].paragraph_length, 7.0);
    assert_eq!(paragraphs.points[0].first_sentence, 1);
    assert_eq!(paragraphs.points[0].last_sentence, 2);
    assert_eq!(paragraphs.words, sentences.words);
}

#[test]
fn zooming_preserves_paragraph_values_and_absolute_word_positions() {
    let source = "She moved home. He watched rain.\n\n\"I am here,\" she said.";
    let doc = Document::parse("test.md", source, Format::Markdown);
    let index = SourceIndex::new(source);
    let data = ChartData::new(&doc, &filter_words::find(&doc, &Lexicons::default()));
    let all = data.view(source, &index, Span::new(0, source.len()), "visible", "paragraph").unwrap();
    assert_eq!(all.points[0].sentence_length, 3.0);
    assert_eq!(all.points[0].paragraph_length, 6.0);
    assert!((all.points[0].finding_density - 1000.0 / 6.0).abs() < 0.001);
    let inside = source.find("watched").unwrap();
    let paragraph = data.view(source, &index, Span::new(inside, inside + 7), "visible", "paragraph").unwrap();
    assert_eq!(serde_json::to_value(&paragraph.points[0]).unwrap(), serde_json::to_value(&all.points[0]).unwrap());
    let last =
        data.view(source, &index, Span::new(source.find('"').unwrap(), source.len()), "visible", "sentence").unwrap();
    assert_eq!(last.words, 5);
    assert_eq!(last.points[0].first_word, 7);
    assert_eq!(last.points[0].last_word, 11);
    assert_eq!(last.points[0].first_sentence, 3);
}

#[test]
fn overview_detail_allocates_samples_within_each_chapter() {
    let chapter = "The rain fell. ".repeat(32);
    let source = format!("# The Book\n\n## Chapter One\n\n{chapter}\n\n## Chapter Two\n\n{chapter}");
    for target in [4, 8, 16] {
        let chart = view(&source, "whole", &format!("chapter-{target}"));
        assert_eq!(chart.points.len(), target * 2);
        assert_eq!(chart.sections.len(), 2);
        assert_eq!(chart.sections[0].title, "Chapter One");
        assert!(chart.points[..target].iter().all(|point| point.section == Some(0) && point.last_sentence <= 32));
        assert!(chart.points[target..].iter().all(|point| point.section == Some(1) && point.first_sentence >= 33));
        assert_eq!(chart.points.last().unwrap().last_sentence, 64);
        assert_eq!(chart.words, 192);
    }
    assert_eq!(view(&source, "whole", "auto").points.len(), 16);
}

#[test]
fn pasted_chapter_labels_are_detected_but_prose_and_code_are_not_boundaries() {
    let body = "The rain fell. ".repeat(24);
    let source = format!(
        "Chapter Twenty-One\n\n{body}\n\nChapter one was a long and difficult journey.\n\n```\n\nChapter 99\n\n```\n\nChapter XXII\n\n{body}"
    );
    let chart = view(&source, "whole", "auto");
    assert_eq!(chart.sections.len(), 2);
    assert_eq!(chart.sections[0].title, "Chapter Twenty-One");
    assert_eq!(chart.sections[1].title, "Chapter XXII");
    assert_eq!(chart.points.len(), 16);
    assert_eq!(chart.points[0].sentence_length, 3.0);
    assert!(chapter_label("Chapter 1: A beginning"));
    assert!(!chapter_label("Chapter one was a difficult time."));
}

#[test]
fn unstructured_manuscripts_get_a_detailed_overview_without_clamping_explicit_choices() {
    let source = "Rain fell. ".repeat(1000);
    for target in [128, 256, 512] {
        let chart = view(&source, "whole", &format!("overview-{target}"));
        assert_eq!(chart.points.len(), target);
        assert_eq!(chart.target, target);
        assert_eq!(chart.words, 2000);
        assert_eq!(chart.points.last().unwrap().last_word, 2000);
    }
    assert_eq!(view(&source, "whole", "auto").points.len(), 256);
}

#[test]
fn detailed_views_accept_the_point_limit_and_reject_one_more_explicitly() {
    for count in [MAX_POINTS - 1, MAX_POINTS, MAX_POINTS + 1] {
        let source = "Go. ".repeat(count);
        let doc = Document::parse("test.md", &source, Format::Plain);
        let index = SourceIndex::new(&source);
        let data = ChartData::new(&doc, &[]);
        let result = data.view(&source, &index, Span::new(0, source.len()), "visible", "sentence");
        if count > MAX_POINTS {
            assert_eq!(result.err(), Some(too_detailed()));
        } else {
            assert_eq!(result.unwrap().points.len(), count);
        }
    }
}

#[test]
fn sample_ranges_cover_the_input_without_overflow_or_overlap() {
    let mut groups = Vec::new();
    sample(0..usize::MAX, 512, &mut groups).unwrap();
    assert_eq!(groups.len(), 512);
    assert_eq!(groups[0].start, 0);
    assert_eq!(groups.last().unwrap().end, usize::MAX);
    assert!(groups.windows(2).all(|pair| pair[0].end == pair[1].start));
}
