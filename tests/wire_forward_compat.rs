use termgrid_core::{Frame, RenderOp, Style};

#[test]
fn style_deserialization_ignores_unknown_fields_for_forward_compat() {
    // v1 should remain tolerant of new style keys appearing in the wild.
    // We intentionally include an unknown key and ensure deserialization succeeds.
    let v = serde_json::json!({
        "fg": 2,
        "dim": true,
        "unknown_future_flag": true
    });

    let s: Style = serde_json::from_value(v).expect("Style should accept unknown fields");
    assert_eq!(s.fg, Some(2));
    assert!(s.dim);
}

#[test]
fn frame_roundtrip_drops_unknown_style_fields_but_keeps_known_fields() {
    let fixture = serde_json::json!({
        "ops": [
            {
                "op": "put",
                "x": 0,
                "y": 0,
                "text": "hi",
                "style": {
                    "dim": true,
                    "unknown_future_flag": true
                }
            }
        ]
    });

    let f: Frame = serde_json::from_value(fixture).expect("Frame should deserialize");
    let out = serde_json::to_value(&f).expect("serialize Frame");

    // Unknown style field should not survive roundtrip, but known dim must.
    let style = &out["ops"][0]["style"];
    assert_eq!(style, &serde_json::json!({"dim": true}));

    // Sanity check the op was parsed as expected.
    assert!(matches!(f.ops[0], RenderOp::Put { .. }));
}
