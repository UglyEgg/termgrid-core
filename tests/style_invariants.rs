use termgrid_core::Style;

#[test]
fn style_overlay_identities_hold() {
    let plain = Style::plain();
    let a = Style {
        fg: Some(3),
        bg: Some(4),
        dim: true,
        bold: true,
        italic: true,
        underline: true,
        blink: true,
        inverse: true,
        strike: true,
    };

    // Identity: overlaying plain changes nothing.
    assert_eq!(a.overlay(plain), a);
    assert_eq!(plain.overlay(a), a);
    assert_eq!(plain.overlay(plain), plain);
}

#[test]
fn style_overlay_boolean_flags_are_or() {
    let base = Style {
        fg: Some(1),
        bg: None,
        dim: false,
        bold: true,
        italic: false,
        underline: true,
        blink: false,
        inverse: false,
        strike: false,
    };
    let top = Style {
        fg: None,
        bg: Some(2),
        dim: true,
        bold: false,
        italic: true,
        underline: false,
        blink: true,
        inverse: true,
        strike: true,
    };

    let out = base.overlay(top);
    assert_eq!(out.fg, Some(1));
    assert_eq!(out.bg, Some(2));
    assert!(out.dim);
    assert!(out.bold);
    assert!(out.italic);
    assert!(out.underline);
    assert!(out.blink);
    assert!(out.inverse);
    assert!(out.strike);
}

#[test]
fn style_serialization_plain_is_empty_object() {
    let v = serde_json::to_value(Style::plain()).expect("serialize Style");
    assert_eq!(v, serde_json::json!({}));
}

#[test]
fn style_serialization_non_plain_emits_fields() {
    let s = Style {
        dim: true,
        underline: true,
        ..Style::plain()
    };
    let v = serde_json::to_value(s).expect("serialize Style");
    assert_eq!(v, serde_json::json!({"dim": true, "underline": true}));
}
