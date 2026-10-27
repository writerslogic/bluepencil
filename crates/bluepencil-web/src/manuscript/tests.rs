use super::*;

fn analyzed(source: &str, format: Format) -> BrowserManuscript {
    let mut manuscript = BrowserManuscript::new(source, format, BROWSER_INPUT_BYTES).unwrap();
    while !manuscript.is_finished() {
        manuscript.advance();
    }
    manuscript
}

fn has(manuscript: &BrowserManuscript, rule: &str) -> bool {
    manuscript.findings.iter().any(|finding| finding.rule == rule)
}

fn for_rule(manuscript: &BrowserManuscript, rule: &str) -> VisibleReview {
    manuscript.visible(0, manuscript.source_index.length, &serde_json::to_string(&[rule]).unwrap()).unwrap()
}

fn selected_text(source: &str, finding: &FindingView) -> String {
    let utf16: Vec<_> = source.encode_utf16().collect();
    String::from_utf16(&utf16[finding.start..finding.end]).unwrap()
}

#[test]
fn browser_size_limit_accepts_the_boundary_and_rejects_one_byte_more() {
    for size in [BROWSER_INPUT_BYTES - 1, BROWSER_INPUT_BYTES] {
        assert!(validate_size(size, BROWSER_INPUT_BYTES).is_ok());
    }
    assert!(validate_size(BROWSER_INPUT_BYTES + 1, BROWSER_INPUT_BYTES).is_err());
    assert!(BrowserManuscript::new("🌊", Format::Plain, 3).is_err());
    assert!(parse_format("html").is_err());
}

#[test]
fn results_require_all_stages_and_advancing_again_is_idempotent() {
    let mut manuscript = BrowserManuscript::new("She moved very slowly.", Format::Plain, BROWSER_INPUT_BYTES).unwrap();
    assert!(manuscript.summary().is_err());
    assert!(manuscript.visible(0, 0, "[]").is_err());
    for _ in BrowserManuscript::STAGES {
        manuscript.advance();
    }
    let total = manuscript.summary().unwrap().total_findings;
    manuscript.advance();
    assert_eq!(manuscript.summary().unwrap().total_findings, total);
}

#[test]
fn analyzes_a_manuscript_beyond_the_old_limit_and_finds_its_ending() {
    let body = "The lantern stood beside the window. She crossed the silent room.\n\n".repeat(17_000);
    let source = format!("# Arrival\n\n{body}# Departure\n\nShe had to make a decision before dawn.");
    assert!(source.len() > 1_000_000);
    let manuscript = analyzed(&source, Format::Markdown);
    assert_eq!(manuscript.counts.words, 17_000 * 11 + 8);
    let ending = for_rule(&manuscript, "nominal");
    assert_eq!(ending.total, 1);
    assert_eq!(selected_text(&source, &ending.findings[0]), "make a decision");
    assert!(ending.findings[0].start > 1_000_000);
    let chart = manuscript.chart(0, source.len(), "whole", "auto").unwrap();
    assert_eq!(chart.points.len(), 9);
    assert_eq!(chart.words, manuscript.counts.words);
}

#[test]
fn every_finding_remains_accessible_as_the_viewport_moves() {
    let line = "She moved very slowly.\n\n";
    let source = line.repeat(260);
    let manuscript = analyzed(&source, Format::Plain);
    let windows: Vec<_> = (0..3)
        .map(|window| {
            let from = window * 100 * line.len();
            let to = ((window + 1) * 100 * line.len()).min(source.len());
            manuscript.visible(from, to, r#"["hedge"]"#).unwrap()
        })
        .collect();
    assert_eq!(windows.iter().map(|window| window.findings.len()).collect::<Vec<_>>(), [100, 100, 60]);
    let starts: HashSet<_> =
        windows.iter().flat_map(|window| window.findings.iter().map(|finding| finding.start)).collect();
    assert_eq!(starts.len(), 260);
    assert_eq!(windows[2].findings.last().unwrap().start, source.rfind("very").unwrap());
    assert!(manuscript.legacy_analysis().truncated);
    assert_eq!(manuscript.legacy_analysis().findings.len(), LEGACY_FINDINGS);
}

#[test]
fn viewport_counts_and_unicode_locations_refer_to_the_original_source() {
    let source = "# Café 🌊\n\n🌊 She moved very slowly.\n\n# Départ\n\nÉlodie moved very carefully.";
    let manuscript = analyzed(source, Format::Markdown);
    let chapter = source[..source.find("# Départ").unwrap()].encode_utf16().count();
    let first = manuscript.visible(0, chapter, r#"["hedge"]"#).unwrap();
    let second = manuscript.visible(chapter, source.encode_utf16().count(), r#"["hedge"]"#).unwrap();
    assert_eq!(first.total, 1);
    assert_eq!(second.total, 1);
    assert_eq!(first.rule_counts["hedge"], 1);
    assert_eq!(first.findings[0].line, 3);
    assert_eq!(first.findings[0].column, 13);
    assert_eq!(second.findings[0].line, 7);
    assert_eq!(selected_text(source, &first.findings[0]), "very");
    assert_eq!(selected_text(source, &second.findings[0]), "very");
}

#[test]
fn returns_safe_empty_results() {
    let manuscript = analyzed("", Format::Plain);
    let review = manuscript.visible(0, 0, "[]").unwrap();
    assert!(review.findings.is_empty());
    assert_eq!(review.total, 0);
    assert!(!review.truncated);
    assert_eq!(manuscript.counts.words, 0);
}

#[test]
fn adds_phrase_nominalization_and_dialogue_findings() {
    let source = "She kept a hand on the latch.\n\nHe kept a hand on the latch.\n\n\
                  Mara kept a hand on the latch.\n\nShe had to make a decision.\n\n\
                  \"Wait,\" she exclaimed.\n\n\"Stop,\" she said softly.";
    let manuscript = analyzed(source, Format::Plain);
    for rule in ["repeated-phrase", "nominal", "dialogue-tag", "adverbial-tag"] {
        assert!(has(&manuscript, rule), "missing {rule}");
    }
    let plain = analyzed("\"Wait,\" she said.\n\n\"Stop,\" he asked.", Format::Plain);
    assert!(!has(&plain, "dialogue-tag"));
    assert!(!has(&plain, "adverbial-tag"));
}

#[test]
fn repeated_phrases_need_three_occurrences() {
    for count in [2, 3, 4] {
        let manuscript = analyzed(&"She kept a hand on the latch.\n\n".repeat(count), Format::Plain);
        assert_eq!(has(&manuscript, "repeated-phrase"), count >= 3);
    }
}

#[test]
fn suppresses_sensory_descriptions_and_conversational_filter_words() {
    let source = "The lobby smelled of wet wool. The soup tasted like smoke.\n\n\
                  \"I think I know what you mean,\" she said.\n\nShe noticed the open door. She smelled smoke.";
    let manuscript = analyzed(source, Format::Plain);
    let filters = for_rule(&manuscript, "filter");
    assert_eq!(filters.total, 2);
    assert_eq!(selected_text(source, &filters.findings[0]), "noticed");
    assert_eq!(selected_text(source, &filters.findings[1]), "smelled");
}

#[test]
fn dialogue_and_section_boundaries_do_not_create_narration_patterns() {
    let dialogue =
        analyzed("\"I came home.\"\n\n\"I saw him.\"\n\n\"I left then.\"\n\n\"I went back.\"", Format::Plain);
    assert!(!has(&dialogue, "monotony"));
    assert!(!has(&dialogue, "repeated-starter"));
    let sections = analyzed(
        "# One\nShe touched gravel.\n\n# Two\nShe crossed gravel.\n\n# Three\nShe carried gravel.",
        Format::Markdown,
    );
    assert!(!has(&sections, "echo"));
    assert!(!has(&sections, "repeated-starter"));
    let narration = analyzed("She came home. She saw him. She left then. She went back.", Format::Plain);
    assert!(has(&narration, "monotony"));
    assert!(has(&narration, "repeated-starter"));
}

#[test]
fn fountain_cues_are_not_prose_tags() {
    let manuscript = analyzed("INT. ROOM - NIGHT\n\nMARA\nShe said softly that he exclaimed.\n", Format::Fountain);
    assert!(!has(&manuscript, "dialogue-tag"));
    assert!(!has(&manuscript, "adverbial-tag"));
    assert!(!has(&manuscript, "filter"));
}

#[test]
fn narrative_shifts_require_a_strong_baseline_and_ignore_dialogue() {
    for past in [11, 12, 13] {
        let source = format!("{}She is ready. She has the letter. She is alone.", "She walked home. ".repeat(past));
        assert_eq!(has(&analyzed(&source, Format::Plain), "tense-drift"), past >= 12);
    }
    let baseline = "She walked home. Her keys jingled in her pocket. She opened the door. ".repeat(8);
    let pov = analyzed(&format!("{baseline}I carried my bag. I opened my letter. I folded my coat."), Format::Plain);
    assert!(has(&pov, "pov-drift"));
    let quoted = analyzed(&format!("{baseline}\"I am ready. I have the letter. I am alone.\""), Format::Plain);
    assert!(!has(&quoted, "pov-drift"));
    assert!(!has(&quoted, "tense-drift"));
}

#[test]
fn long_sentence_excerpts_stay_bounded_and_include_the_finding() {
    let source = format!("{}🌊 very cold.", "stone ".repeat(10_000));
    let manuscript = analyzed(&source, Format::Plain);
    let review = for_rule(&manuscript, "hedge");
    let finding = &review.findings[0];
    assert!(finding.excerpt.chars().count() <= 242);
    assert!(finding.excerpt.contains("very"));
    assert_eq!(selected_text(&source, finding), "very");
}

#[test]
fn markdown_fences_keep_their_context_across_large_passages() {
    let source =
        format!("# Sample\n\n```text\n{}\n```\n\nShe moved very slowly.", "She moved very slowly.\n".repeat(6_000));
    let manuscript = analyzed(&source, Format::Markdown);
    assert_eq!(manuscript.counts.words, 4);
    assert_eq!(for_rule(&manuscript, "hedge").total, 1);
}

#[test]
fn viewport_queries_include_overlapping_patterns_and_only_enabled_word_findings() {
    let source = "She moved very slowly.\n\n".repeat(260);
    let manuscript = analyzed(&source, Format::Plain);
    let from = source.rfind("She").unwrap();
    let visible = manuscript.visible(from, source.len(), r#"["hedge","repeated-starter"]"#).unwrap();
    assert_eq!(visible.rule_counts["hedge"], 1);
    assert_eq!(visible.findings.iter().filter(|finding| finding.rule == "hedge").count(), 1);
    assert!(visible.findings.iter().any(|finding| finding.rule == "repeated-starter" && finding.start < from));
    assert!(visible.findings.iter().all(|finding| finding.rule != "adverb"));
    assert!(manuscript.visible(from, source.len(), "[]").unwrap().findings.is_empty());
    assert!(manuscript.visible(source.len(), from, "[]").is_err());
    assert!(manuscript.visible(0, source.len() + 1, "[]").is_err());
    assert!(manuscript.visible(0, source.len(), r#"["unknown"]"#).is_err());
    assert!(manuscript.visible(0, source.len(), "null").is_err());
}

#[test]
fn sparse_source_positions_round_trip_at_unicode_checkpoint_boundaries() {
    let source = format!("{}🌊\n{}é\n{}", "a".repeat(1022), "b".repeat(1021), "石".repeat(400));
    let index = SourceIndex::new(&source);
    for byte in source.char_indices().map(|(byte, _)| byte).chain([source.len()]) {
        let position = index.position(&source, byte);
        assert_eq!(position.utf16, source[..byte].encode_utf16().count());
        assert_eq!(index.byte(&source, position.utf16), byte);
    }
    assert_eq!(index.byte(&source, 1023), 1022);
}

#[test]
fn chart_scope_and_detail_validate_their_inputs() {
    let source = "She moved home. He watched rain.\n\n\"I am here,\" she said.";
    let manuscript = analyzed(source, Format::Plain);
    let zoomed = manuscript.chart(source.find('"').unwrap(), source.len(), "visible", "sentence").unwrap();
    assert_eq!(zoomed.words, 5);
    assert!(manuscript.chart(0, source.len(), "whole", "sentence").is_err());
    assert!(manuscript.chart(0, source.len(), "visible", "chapter-8").is_err());
    assert!(manuscript.chart(0, source.len(), "visible", "25").is_err());
    assert!(manuscript.chart(0, source.len() + 1, "visible", "sentence").is_err());
    for source in ["", "!!!", "# Heading"] {
        assert!(analyzed(source, Format::Markdown).chart(0, source.len(), "whole", "auto").unwrap().points.is_empty());
    }
}
