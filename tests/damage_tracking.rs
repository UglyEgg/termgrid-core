use termgrid_core::{
    Frame, GlyphRegistry, Rect, RenderOp, RenderProfile, Renderer, Span, Style, WrapOpts,
};

fn reg() -> GlyphRegistry {
    GlyphRegistry::new(RenderProfile::bbsstalgia_xtermjs_unicode11_example())
}

fn cell_changed(a: &termgrid_core::Cell, b: &termgrid_core::Cell) -> bool {
    a != b
}

fn covered_by(rects: &[Rect], x: u16, y: u16) -> bool {
    rects
        .iter()
        .any(|r| x >= r.x && y >= r.y && x < r.x.saturating_add(r.w) && y < r.y.saturating_add(r.h))
}

#[test]
fn damage_covers_all_changed_cells_for_simple_ops() {
    let reg = reg();
    let mut r = Renderer::new(12, 4, reg).unwrap();

    let before = r.grid().clone();

    let f = Frame {
        ops: vec![
            RenderOp::Put {
                x: 0,
                y: 0,
                text: "Hi".to_string(),
                style: Style::plain(),
            },
            RenderOp::FillRect {
                x: 3,
                y: 1,
                w: 4,
                h: 2,
                glyph: " ".to_string(),
                style: Style {
                    bold: true,
                    ..Style::plain()
                },
            },
            RenderOp::TextBlockStyled {
                x: 0,
                y: 3,
                w: 10,
                spans: vec![Span::new("One two three", Style::plain())],
                wrap: WrapOpts::default(),
                h: 1,
            },
        ],
    };

    let dmg = r.apply_with_damage(&f);

    assert!(!dmg.full_redraw);
    assert!(!dmg.rects.is_empty());

    let after = r.grid().clone();

    for y in 0..after.height {
        for x in 0..after.width {
            let a = before.get(x, y).unwrap();
            let b = after.get(x, y).unwrap();
            if cell_changed(a, b) {
                assert!(
                    covered_by(&dmg.rects, x, y),
                    "changed cell not covered: ({x},{y}) dmg={dmg:?}"
                );
            }
        }
    }
}

#[test]
fn damage_caps_to_full_redraw_when_too_many_rects() {
    let reg = reg();
    let mut r = Renderer::new(40, 10, reg).unwrap();

    // Many tiny, non-adjacent ops -> many rects -> should flip to full redraw.
    // Ensure no rectangles touch so merge_in_place() cannot coalesce them.
    let mut ops = Vec::new();
    for y in (0u16..10u16).step_by(2) {
        for x in (0u16..40u16).step_by(2) {
            ops.push(RenderOp::PutGlyph {
                x,
                y,
                glyph: "x".to_string(),
                style: Style::plain(),
            });
        }
    }
    // This creates 20*5 = 100 distinct 1x1 dirty rects (> cap of 64).
    let f = Frame { ops };

    let dmg = r.apply_with_damage(&f);
    assert!(dmg.full_redraw);
    assert!(dmg.rects.is_empty());
}
