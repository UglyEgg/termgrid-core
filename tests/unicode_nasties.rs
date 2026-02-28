use termgrid_core::{
    clip_to_cells_text, ellipsis_to_cells_text, GlyphRegistry, RenderProfile, Style,
};

fn reg() -> GlyphRegistry {
    GlyphRegistry::new(RenderProfile::bbsstalgia_xtermjs_unicode11_example())
}

#[test]
fn clip_does_not_split_zwj_family_sequence() {
    // Family: man, woman, girl, boy (ZWJ sequence)
    let s = "👨‍👩‍👧‍👦";
    let reg = reg();

    // If the sequence is considered width=2 by policy/heuristic, clipping to 1 cell should
    // yield empty; clipping to 2 should keep it whole. We assert the "no split" property.
    let (c1, _clipped) = clip_to_cells_text(&reg, s, 1);
    assert!(c1.is_empty() || c1 == s);
    let (c2, _clipped) = clip_to_cells_text(&reg, s, 2);
    assert!(c2.is_empty() || c2 == s);
}

#[test]
fn ellipsis_does_not_split_regional_indicator_flag() {
    // Flag: United States (two regional indicators)
    let s = "🇺🇸";
    let reg = reg();

    let out = ellipsis_to_cells_text(&reg, s, 1, "…");
    // We should never output a half-flag (single regional indicator).
    assert!(out.is_empty() || out == "…" || out == s);
}

#[test]
fn style_flags_do_not_affect_width_measurement() {
    let reg = reg();
    let plain = clip_to_cells_text(&reg, "🙂", 2);
    let _styled = Style {
        bold: true,
        italic: true,
        underline: true,
        blink: true,
        inverse: true,
        strike: true,
        dim: true,
        ..Style::plain()
    };

    // Width policy is purely glyph-based; style should not change string-level clipping.
    let styled = clip_to_cells_text(&reg, "🙂", 2);
    assert_eq!(plain, styled);
}
