use pretty_assertions::assert_eq;

use termgrid_core::{normalize_spans, spans_plain_text, Span, Style};

#[test]
fn spans_plain_text_concatenates() {
    let spans = vec![
        Span::new("Hello ", Style::plain()),
        Span::new("World", Style::plain()),
    ];
    assert_eq!(spans_plain_text(&spans), "Hello World".to_string());
}

#[test]
fn normalize_spans_drops_empty_and_coalesces() {
    let s1 = Style::plain();
    let s2 = Style {
        bold: true,
        ..Style::plain()
    };

    let spans = vec![
        Span::new("", s1),
        Span::new("A", s1),
        Span::new("B", s1),
        Span::new("", s2),
        Span::new("C", s2),
        Span::new("D", s1),
        Span::new("", s1),
    ];

    let norm = normalize_spans(&spans);

    assert_eq!(
        norm,
        vec![Span::new("AB", s1), Span::new("C", s2), Span::new("D", s1),]
    );
}
