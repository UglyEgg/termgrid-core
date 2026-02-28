use termgrid_core::{
    Frame, GlyphRegistry, RenderOp, RenderProfile, Renderer, Span, Style, TruncateMode, WrapOpts,
};

fn reg() -> GlyphRegistry {
    GlyphRegistry::new(RenderProfile::bbsstalgia_xtermjs_unicode11_example())
}

#[test]
fn applying_same_frame_twice_is_idempotent() {
    let reg = reg();
    let mut r = Renderer::new(12, 4, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::Clear);
    f.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "hello 🙂 world".to_string(),
        style: Style::plain(),
    });
    f.ops.push(RenderOp::Label {
        x: 0,
        y: 1,
        w: 8,
        text: "abcdefghi".to_string(),
        style: Style {
            bold: true,
            ..Style::plain()
        },
        truncate: TruncateMode::Ellipsis,
    });

    r.apply(&f);
    let g1 = r.grid().clone();
    r.apply(&f);
    let g2 = r.grid().clone();

    assert_eq!(g1, g2);
}

#[test]
fn apply_frame_equivalent_to_apply_ops_in_order() {
    let mut r1 = Renderer::new(16, 3, reg()).expect("renderer");
    let mut r2 = Renderer::new(16, 3, reg()).expect("renderer");

    let spans = vec![Span::new("One two three four", Style::plain())];
    let wrap = WrapOpts {
        continuation_prefix: Some(vec![Span::new(
            "↳ ",
            Style {
                dim: true,
                ..Style::plain()
            },
        )]),
        ..Default::default()
    };

    let op = RenderOp::TextBlockStyled {
        x: 0,
        y: 0,
        w: 8,
        spans,
        wrap,
        h: 2,
    };

    let mut f = Frame::default();
    f.ops.push(RenderOp::Clear);
    f.ops.push(op.clone());

    r1.apply(&f);
    r2.apply_op(&RenderOp::Clear);
    r2.apply_op(&op);

    assert_eq!(r1.grid(), r2.grid());
}
