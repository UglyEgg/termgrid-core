use pretty_assertions::assert_eq;

use termgrid_core::{apply_highlight, spans_plain_text, Span, Style};

fn style_fg(fg: u8) -> Style {
    Style {
        fg: Some(fg),
        bg: None,
        dim: false,
        bold: false,
        italic: false,
        underline: false,
        blink: false,
        inverse: false,
        strike: false,
    }
}

#[test]
fn apply_highlight_targets_grapheme_ranges_and_preserves_text() {
    let s1 = style_fg(2);
    let s2 = style_fg(3);

    let spans = vec![Span::new("a🙂", s1), Span::new("b", s2)];

    // Graphemes: ["a" (0), "🙂" (1), "b" (2)]
    let hi = Style {
        fg: None,
        bg: Some(1),
        dim: false,
        bold: true,
        italic: false,
        underline: false,
        blink: false,
        inverse: false,
        strike: false,
    };

    let out = apply_highlight(&spans, &[(1, 2)], hi);
    assert_eq!(spans_plain_text(&out), "a🙂b");

    assert_eq!(out.len(), 3);

    assert_eq!(out[0].text, "a");
    assert_eq!(out[0].style, s1);

    assert_eq!(out[1].text, "🙂");
    assert_eq!(out[1].style.fg, Some(2));
    assert_eq!(out[1].style.bg, Some(1));
    assert_eq!(out[1].style.bold, true);

    assert_eq!(out[2].text, "b");
    assert_eq!(out[2].style, s2);
}

#[test]
fn apply_highlight_can_cross_span_boundaries_and_merges_ranges() {
    let s1 = style_fg(5);
    let s2 = style_fg(6);

    let spans = vec![Span::new("a", s1), Span::new("🙂b", s2)];

    // Highlight "🙂b" using overlapping ranges that should merge.
    // Graphemes: ["a" (0), "🙂" (1), "b" (2)]
    let hi = Style {
        fg: None,
        bg: Some(7),
        dim: false,
        bold: false,
        italic: false,
        underline: true,
        blink: false,
        inverse: false,
        strike: false,
    };

    let out = apply_highlight(&spans, &[(1, 3), (2, 3)], hi);
    assert_eq!(spans_plain_text(&out), "a🙂b");

    // Expected: "a" unhighlighted, "🙂b" highlighted with s2 overlaid.
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].text, "a");
    assert_eq!(out[0].style, s1);

    assert_eq!(out[1].text, "🙂b");
    assert_eq!(out[1].style.fg, Some(6));
    assert_eq!(out[1].style.bg, Some(7));
    assert_eq!(out[1].style.underline, true);
}
