use pretty_assertions::assert_eq;

use termgrid_core::{wrap_spans_wordwise, GlyphRegistry, RenderProfile, Span, Style, WrapOpts};

fn load_example_registry() -> GlyphRegistry {
    let json = include_str!("../testdata/profile_example.json");
    let profile: RenderProfile = serde_json::from_str(json).expect("profile json");
    GlyphRegistry::new(profile)
}

fn lines_to_plain(lines: &[Vec<Span>]) -> Vec<String> {
    lines
        .iter()
        .map(|ls| ls.iter().map(|s| s.text.clone()).collect::<String>())
        .collect()
}

#[test]
fn wrap_simple_words_default_opts() {
    let reg = load_example_registry();
    let spans = vec![Span::new("Hello world", Style::plain())];
    let lines = wrap_spans_wordwise(&reg, &spans, 5, &WrapOpts::default());
    assert_eq!(
        lines_to_plain(&lines),
        vec!["Hello".to_string(), "world".to_string()]
    );
}

#[test]
fn wrap_preserves_styles_across_lines() {
    let reg = load_example_registry();
    let bold = Style {
        bold: true,
        ..Style::plain()
    };
    let spans = vec![
        Span::new("Hello ", Style::plain()),
        Span::new("world", bold),
    ];
    let lines = wrap_spans_wordwise(&reg, &spans, 5, &WrapOpts::default());
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0][0].text, "Hello");
    assert_eq!(lines[0][0].style, Style::plain());
    assert_eq!(lines[1][0].text, "world");
    assert_eq!(lines[1][0].style, bold);
}

#[test]
fn wrap_hard_breaks_long_token_by_grapheme() {
    let reg = load_example_registry();
    // profile_example.json defines 🙂 as width 2
    let spans = vec![Span::new("A🙂B", Style::plain())];
    let lines = wrap_spans_wordwise(&reg, &spans, 2, &WrapOpts::default());
    assert_eq!(
        lines_to_plain(&lines),
        vec!["A".to_string(), "🙂".to_string(), "B".to_string()]
    );
}

#[test]
fn wrap_preserve_spaces_keeps_indentation() {
    let reg = load_example_registry();
    let opts = WrapOpts {
        preserve_spaces: true,
        ..Default::default()
    };
    let spans = vec![Span::new("    indented", Style::plain())];
    let lines = wrap_spans_wordwise(&reg, &spans, 6, &opts);
    // first visual line keeps the leading spaces
    assert_eq!(lines_to_plain(&lines)[0], "    in".to_string());
}

#[test]
fn wrap_newlines_force_hard_breaks() {
    let reg = load_example_registry();
    let spans = vec![Span::new("a\nb", Style::plain())];
    let lines = wrap_spans_wordwise(&reg, &spans, 10, &WrapOpts::default());
    assert_eq!(
        lines_to_plain(&lines),
        vec!["a".to_string(), "b".to_string()]
    );
}

#[test]
fn wrap_continuation_prefix_only_on_wrapped_lines() {
    let reg = load_example_registry();
    let opts = WrapOpts {
        continuation_prefix: Some(vec![Span::new("↳ ", Style::plain())]),
        ..Default::default()
    };

    // First break is due to wrapping; second break is due to explicit newline.
    let spans = vec![Span::new("one two three\nfour five", Style::plain())];
    let lines = wrap_spans_wordwise(&reg, &spans, 7, &opts);

    // width=7, so "one two" fits (7 incl space). "three" wraps and gets prefix.
    // After explicit newline, the next line is not a continuation.
    assert_eq!(
        lines_to_plain(&lines),
        vec![
            "one two".to_string(),
            "↳ three".to_string(),
            "four".to_string(),
            "↳ five".to_string()
        ]
    );
}
