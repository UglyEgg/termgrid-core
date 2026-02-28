use pretty_assertions::assert_eq;

use termgrid_core::{GlyphRegistry, RenderProfile};

#[test]
fn builtin_unicode11_example_matches_testdata_json() {
    let json = include_str!("../testdata/profile_example.json");
    let from_json: RenderProfile = serde_json::from_str(json).expect("profile json");

    let from_fn = RenderProfile::bbsstalgia_xtermjs_unicode11_example();

    assert_eq!(from_fn, from_json);

    // And the registry constructed from either should yield the same answers.
    let r1 = GlyphRegistry::new(from_json);
    let r2 = GlyphRegistry::new(from_fn);

    assert_eq!(r1.width("🙂"), 2);
    assert_eq!(r1.width("⚙️"), 2);
    assert_eq!(r1.width("🧠"), 2);
    assert_eq!(r1.width("❤"), 1);

    assert_eq!(r2.width("🙂"), 2);
    assert_eq!(r2.width("⚙️"), 2);
    assert_eq!(r2.width("🧠"), 2);
    assert_eq!(r2.width("❤"), 1);
}
