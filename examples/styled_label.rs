use termgrid_core::{
    Cell, Frame, GlyphRegistry, RenderOp, RenderProfile, Renderer, Span, Style, TruncateMode,
};

fn registry() -> GlyphRegistry {
    // For examples we build a tiny profile inline.
    let mut p = RenderProfile::empty("example", 1);
    // Treat this emoji as wide (2 cells) to demonstrate continuation behavior.
    p.set_width("🙂", 2);
    GlyphRegistry::new(p)
}

fn cell_glyph(c: &Cell) -> &str {
    match c {
        Cell::Empty => " ",
        Cell::Continuation => "·",
        Cell::Glyph { grapheme, .. } => grapheme,
    }
}

fn main() {
    let reg = registry();
    let mut r = Renderer::new(40, 6, reg).expect("valid grid");

    let mut frame = Frame::default();
    frame.ops.push(RenderOp::Clear);
    frame.ops.push(RenderOp::Box {
        x: 0,
        y: 0,
        w: 40,
        h: 6,
        charset: Default::default(),
        style: Style::plain(),
    });

    let spans = vec![
        Span::new("Styled ", Style::plain()),
        Span::new(
            "label",
            Style {
                bold: true,
                ..Style::plain()
            },
        ),
        Span::new(" ", Style::plain()),
        Span::new("🙂", Style::plain()),
        Span::new(" clipped", Style::plain()),
    ];

    frame.ops.push(RenderOp::LabelStyled {
        x: 2,
        y: 2,
        w: 36,
        spans,
        truncate: TruncateMode::Ellipsis,
    });

    r.apply(&frame);

    // For demo purposes, print glyphs only (styles are not shown).
    // Continuation cells for wide glyphs are shown as "·".
    for y in 0..r.grid().height {
        let mut line = String::new();
        for x in 0..r.grid().width {
            let c = r.grid().get(x, y).unwrap_or(&Cell::Empty);
            line.push_str(cell_glyph(c));
        }
        println!("{}", line);
    }
}
