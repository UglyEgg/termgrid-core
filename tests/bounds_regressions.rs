use termgrid_core::{GlyphRegistry, Grid, RenderProfile, Style};

fn reg() -> GlyphRegistry {
    GlyphRegistry::new(RenderProfile::bbsstalgia_xtermjs_unicode11_example())
}

#[test]
fn wide_glyph_at_right_edge_is_not_partially_placed() {
    let reg = reg();
    let mut g = Grid::new(4, 1);
    g.put_text(3, 0, "🙂", Style::plain(), &reg);
    // Nothing should be written because the glyph would be half off-screen.
    assert!(matches!(g.get(3, 0).unwrap(), termgrid_core::Cell::Empty));
}

#[test]
fn overwriting_continuation_clears_lead_cell() {
    let reg = reg();
    let mut g = Grid::new(4, 1);

    g.put_text(1, 0, "🙂", Style::plain(), &reg);
    assert!(!g.get(1, 0).unwrap().is_continuation());
    assert!(g.get(2, 0).unwrap().is_continuation());

    // Overwrite the continuation cell.
    g.put_text(2, 0, "X", Style::plain(), &reg);

    // The leading wide glyph must be cleared, leaving a plain glyph at x=2.
    assert!(matches!(g.get(1, 0).unwrap(), termgrid_core::Cell::Empty));
    assert!(matches!(
        g.get(2, 0).unwrap(),
        termgrid_core::Cell::Glyph { .. }
    ));
}
