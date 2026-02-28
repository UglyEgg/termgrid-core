use pretty_assertions::assert_eq;

use termgrid_core::{
    measure_cells_spans, measure_cells_text, GlyphRegistry, RenderProfile, Span, Style,
};

fn load_example_registry() -> GlyphRegistry {
    let json = include_str!("../testdata/profile_example.json");
    let profile: RenderProfile = serde_json::from_str(json).expect("profile json");
    GlyphRegistry::new(profile)
}

#[test]
fn measure_cells_text_ascii() {
    let reg = load_example_registry();
    assert_eq!(measure_cells_text(&reg, "abc"), 3);
}

#[test]
fn measure_cells_text_emoji_width_policy() {
    let reg = load_example_registry();
    // profile_example.json defines 🙂 as width 2
    assert_eq!(measure_cells_text(&reg, "🙂"), 2);
    assert_eq!(measure_cells_text(&reg, "A🙂B"), 1 + 2 + 1);
}

#[test]
fn measure_cells_spans_mixed_styles() {
    let reg = load_example_registry();

    let spans = vec![
        Span::new("A", Style::plain()),
        Span::new(
            "🙂",
            Style {
                bold: true,
                ..Style::plain()
            },
        ),
        Span::new("B", Style::plain()),
    ];

    assert_eq!(measure_cells_spans(&reg, &spans), 4);
}
