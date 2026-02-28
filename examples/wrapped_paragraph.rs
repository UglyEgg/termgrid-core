use termgrid_core::{
    Cell, Frame, GlyphRegistry, RenderOp, RenderProfile, Renderer, Span, Style, TruncateMode,
    WrapOpts,
};

fn registry() -> GlyphRegistry {
    let mut p = RenderProfile::empty("example", 1);
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
    let mut r = Renderer::new(52, 10, reg).expect("valid grid");

    let mut frame = Frame::default();
    frame.ops.push(RenderOp::Clear);
    frame.ops.push(RenderOp::Box {
        x: 0,
        y: 0,
        w: 52,
        h: 10,
        charset: Default::default(),
        style: Style::plain(),
    });

    let spans = vec![
        Span::new("Press ", Style::plain()),
        Span::new(
            "Enter",
            Style {
                bold: true,
                ..Style::plain()
            },
        ),
        Span::new(" to submit. Use ", Style::plain()),
        Span::new(
            "Ctrl-F",
            Style {
                underline: true,
                ..Style::plain()
            },
        ),
        Span::new(" to search. Emoji: ", Style::plain()),
        Span::new("🙂", Style::plain()),
        Span::new(" stays aligned.", Style::plain()),
    ];

    let wrap = WrapOpts {
        continuation_prefix: Some(vec![Span::new("↳ ", Style::plain())]),
        ..Default::default()
    };

    frame.ops.push(RenderOp::PutWrappedStyled {
        x: 2,
        y: 2,
        w: 48,
        spans,
        wrap_opts: wrap,
        max_lines: Some(5),
    });

    frame.ops.push(RenderOp::Label {
        x: 2,
        y: 8,
        w: 48,
        text: "Footer label (ellipsis)".to_string(),
        style: Style::plain(),
        truncate: TruncateMode::Ellipsis,
    });

    r.apply(&frame);

    for y in 0..r.grid().height {
        let mut line = String::new();
        for x in 0..r.grid().width {
            let c = r.grid().get(x, y).unwrap_or(&Cell::Empty);
            line.push_str(cell_glyph(c));
        }
        println!("{}", line);
    }
}
