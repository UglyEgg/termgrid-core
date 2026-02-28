use pretty_assertions::assert_eq;
use serde_json::json;

use termgrid_core::WrapOpts;
use termgrid_core::{BoxCharset, Frame, RenderOp, RenderProfile, Style, TruncateMode};

fn read_json_value(rel_path: &str) -> serde_json::Value {
    // IMPORTANT:
    // These fixtures are part of the crate source tree and the stable roundtrip
    // test asserts byte-for-byte equality against them. We intentionally anchor
    // fixture loading to the *compile-time* manifest dir of the test binary.
    //
    // Rationale: some runners may set a runtime CARGO_MANIFEST_DIR that differs
    // from the compile-time path (e.g. when executing a prebuilt test binary
    // from another checkout). That can make the same test compare against a
    // different fixture set and produce confusing diffs.
    //
    // If you are running prebuilt test binaries across checkouts, remove the
    // shared target dir / delete `target/` so tests rebuild with the intended
    // fixture root.
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join(rel_path);
    let s = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("failed to read {}: {e}", path.display());
    });
    serde_json::from_str(&s).unwrap_or_else(|e| {
        panic!("invalid json in {}: {e}", path.display());
    })
}

#[test]
fn style_default_serializes_to_empty_object() {
    let v = serde_json::to_value(Style::default()).expect("serialize Style");
    assert_eq!(v, json!({}));
}

#[test]
fn style_flags_are_representable_in_wire_v1() {
    // Wire v1 must be able to express the "classic" terminal flags we rely on.
    // Key names are part of the wire contract.
    let s = Style {
        fg: None,
        bg: None,
        dim: false,
        bold: true,
        italic: true,
        underline: true,
        blink: true,
        inverse: true,
        strike: true,
    };
    let v = serde_json::to_value(s).expect("serialize Style");
    assert_eq!(
        v,
        json!({
            "bold": true,
            "italic": true,
            "underline": true,
            "blink": true,
            "reverse": true,
            "strikethrough": true
        })
    );
}

#[test]
fn plain_style_is_omitted_on_ops() {
    let op = RenderOp::Put {
        x: 0,
        y: 0,
        text: "Hi".to_string(),
        style: Style::plain(),
    };
    let v = serde_json::to_value(op).expect("serialize RenderOp");
    assert_eq!(v, json!({"op":"put","x":0,"y":0,"text":"Hi"}));
}

#[test]
fn default_charset_is_omitted_on_box_op() {
    let op = RenderOp::Box {
        x: 0,
        y: 0,
        w: 5,
        h: 3,
        style: Style::plain(),
        charset: BoxCharset::UnicodeSingle,
    };
    let v = serde_json::to_value(op).expect("serialize RenderOp");
    assert_eq!(v, json!({"op":"box","x":0,"y":0,"w":5,"h":3}));
}

#[test]
fn default_truncate_mode_is_omitted_on_label_op() {
    let op = RenderOp::Label {
        x: 0,
        y: 0,
        w: 10,
        text: "Hello".to_string(),
        style: Style::plain(),
        truncate: TruncateMode::Clip,
    };
    let v = serde_json::to_value(op).expect("serialize RenderOp");
    assert_eq!(v, json!({"op":"label","x":0,"y":0,"w":10,"text":"Hello"}));
}

#[test]
fn default_truncate_mode_is_omitted_on_label_styled_op() {
    let op = RenderOp::LabelStyled {
        x: 0,
        y: 0,
        w: 10,
        spans: vec![termgrid_core::Span::new("Hi", Style::plain())],
        truncate: TruncateMode::Clip,
    };
    let v = serde_json::to_value(op).expect("serialize RenderOp");
    assert_eq!(
        v,
        json!({"op":"label_styled","x":0,"y":0,"w":10,"spans":[{"text":"Hi"}]})
    );
}

#[test]
fn wrap_defaults_are_canonicalized_on_text_block_styled_op() {
    let op = RenderOp::TextBlockStyled {
        x: 0,
        y: 0,
        w: 10,
        h: u16::MAX,
        spans: vec![termgrid_core::Span::new("Hi", Style::plain())],
        wrap: WrapOpts::default(),
    };

    let v = serde_json::to_value(op).expect("serialize RenderOp");

    // Canonical v1 omits wrap when it is all-default.
    assert_eq!(
        v.get("op").and_then(|x| x.as_str()),
        Some("text_block_styled")
    );
    assert_eq!(v.get("x").and_then(|x| x.as_u64()), Some(0));
    assert_eq!(v.get("y").and_then(|x| x.as_u64()), Some(0));
    assert_eq!(v.get("w").and_then(|x| x.as_u64()), Some(10));
    assert_eq!(v.get("h").and_then(|x| x.as_u64()), Some(65535));
    assert!(
        v.get("wrap").is_none(),
        "wrap should be omitted when default"
    );
}

#[test]
fn frame_v1_fixtures_roundtrip_stably() {
    let fixtures = [
        "testdata/wire/frame_clear.json",
        "testdata/wire/frame_clear_line.json",
        "testdata/wire/frame_clear_eol.json",
        "testdata/wire/frame_clear_bol.json",
        "testdata/wire/frame_clear_eos.json",
        "testdata/wire/frame_clear_rect.json",
        "testdata/wire/frame_put_plain.json",
        "testdata/wire/frame_put_with_style.json",
        "testdata/wire/frame_label_clip.json",
        "testdata/wire/frame_label_ellipsis.json",
        "testdata/wire/frame_label_styled_clip.json",
        "testdata/wire/frame_text_block.json",
        "testdata/wire/frame_text_block_styled.json",
        "testdata/wire/frame_text_block_styled_prefix.json",
        "testdata/wire/frame_text_block_styled_max_lines.json",
        "testdata/wire/frame_blit.json",
        "testdata/wire/frame_fill_rect_bg.json",
        "testdata/wire/frame_fill_rect_hline.json",
        "testdata/wire/frame_fill_rect_vline.json",
        "testdata/wire/frame_box_default.json",
        "testdata/wire/frame_box_ascii.json",
    ];

    for rel in fixtures {
        let expected = read_json_value(rel);
        let frame: Frame = serde_json::from_value(expected.clone()).unwrap_or_else(|e| {
            panic!("fixture did not deserialize as Frame: {}: {e}", rel);
        });

        let actual = serde_json::to_value(&frame).expect("serialize Frame");
        if actual != expected {
            // Helpful when running prebuilt test binaries or when fixtures are unexpectedly stale.
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            eprintln!("fixture path: {}", root.join(rel).display());
        }
        assert_eq!(
            actual, expected,
            "stable roundtrip mismatch for fixture: {rel}"
        );
    }
}

#[test]
fn render_profile_example_roundtrip_stably() {
    let expected = read_json_value("testdata/profile_example.json");
    let profile: RenderProfile =
        serde_json::from_value(expected.clone()).expect("deserialize RenderProfile");
    let actual = serde_json::to_value(&profile).expect("serialize RenderProfile");
    assert_eq!(actual, expected);
}
