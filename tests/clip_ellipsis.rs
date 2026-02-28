use pretty_assertions::assert_eq;

use termgrid_core::{
    clip_to_cells_spans, clip_to_cells_text, ellipsis_to_cells_spans, ellipsis_to_cells_text,
    GlyphRegistry, RenderProfile, Span, Style,
};

fn load_example_registry() -> GlyphRegistry {
    let json = include_str!("../testdata/profile_example.json");
    let profile: RenderProfile = serde_json::from_str(json).expect("profile json");
    GlyphRegistry::new(profile)
}

#[test]
fn clip_to_cells_text_does_not_split_emoji() {
    let reg = load_example_registry();

    let (s, clipped) = clip_to_cells_text(&reg, "A🙂B", 1);
    assert_eq!(s, "A".to_string());
    assert_eq!(clipped, true);

    let (s2, clipped2) = clip_to_cells_text(&reg, "A🙂B", 3);
    // A (1) + 🙂 (2) fits exactly.
    assert_eq!(s2, "A🙂".to_string());
    assert_eq!(clipped2, true);
}

#[test]
fn clip_to_cells_text_does_not_split_combining_grapheme() {
    let reg = load_example_registry();

    let combining = "e\u{301}X"; // "e" + combining acute accent, then X
    let (s, clipped) = clip_to_cells_text(&reg, combining, 1);
    assert_eq!(s, "e\u{301}".to_string());
    assert_eq!(clipped, true);
}

#[test]
fn clip_to_cells_spans_preserves_styles_and_coalesces() {
    let reg = load_example_registry();

    let s_plain = Style::plain();
    let s_bold = Style {
        bold: true,
        ..Style::plain()
    };

    let spans = vec![
        Span::new("A", s_plain),
        Span::new("🙂", s_bold),
        Span::new("B", s_plain),
    ];

    let (out, clipped) = clip_to_cells_spans(&reg, &spans, 3);
    // A (1) + 🙂 (2) = 3
    assert_eq!(out, vec![Span::new("A", s_plain), Span::new("🙂", s_bold)]);
    assert_eq!(clipped, true);
}

#[test]
fn ellipsis_to_cells_text_respects_width() {
    let reg = load_example_registry();

    assert_eq!(
        ellipsis_to_cells_text(&reg, "Hello", 10, "…"),
        "Hello".to_string()
    );
    assert_eq!(
        ellipsis_to_cells_text(&reg, "Hello", 3, "…"),
        "He…".to_string()
    );

    // Ellipsis itself longer than width: should be clipped to fit.
    assert_eq!(
        ellipsis_to_cells_text(&reg, "Hello", 1, ".."),
        ".".to_string()
    );
}

#[test]
fn ellipsis_to_cells_spans_appends_ellipsis_span() {
    let reg = load_example_registry();

    let s_plain = Style::plain();
    let s_dim = Style {
        underline: true,
        ..Style::plain()
    };

    let spans = vec![Span::new("Hello", s_plain)];
    let ell = Span::new("…", s_dim);

    let out = ellipsis_to_cells_spans(&reg, &spans, 3, &ell);
    assert_eq!(out, vec![Span::new("He", s_plain), Span::new("…", s_dim)]);
}
