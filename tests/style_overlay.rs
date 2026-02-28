use termgrid_core::Style;

#[test]
fn style_overlay_fg_bg_precedence_and_flags_or() {
    let base = Style {
        fg: Some(1),
        bg: Some(2),
        dim: false,
        bold: true,
        italic: false,
        underline: false,
        blink: false,
        inverse: false,
        strike: false,
    };

    let top = Style {
        fg: None,
        bg: Some(9),
        dim: false,
        bold: false,
        italic: false,
        underline: true,
        blink: false,
        inverse: false,
        strike: false,
    };

    let out = base.overlay(top);
    assert_eq!(out.fg, Some(1));
    assert_eq!(out.bg, Some(9));
    assert!(out.bold);
    assert!(out.underline);
    assert!(!out.inverse);
}

#[test]
fn style_overlay_top_fg_wins() {
    let base = Style {
        fg: Some(3),
        bg: None,
        dim: false,
        bold: false,
        italic: false,
        underline: false,
        blink: false,
        inverse: true,
        strike: false,
    };

    let top = Style {
        fg: Some(7),
        bg: None,
        dim: false,
        bold: true,
        italic: false,
        underline: false,
        blink: false,
        inverse: false,
        strike: false,
    };

    let out = base.overlay(top);
    assert_eq!(out.fg, Some(7));
    assert_eq!(out.bg, None);
    assert!(out.bold);
    assert!(!out.underline);
    assert!(out.inverse);
}
