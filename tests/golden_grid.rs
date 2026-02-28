use pretty_assertions::assert_eq;

use termgrid_core::WrapOpts;
use termgrid_core::{
    BlitCell, BoxCharset, Cell, Frame, GlyphRegistry, RenderOp, RenderProfile, Renderer, Span,
    Style, TruncateMode,
};

fn load_example_registry() -> GlyphRegistry {
    let json = include_str!("../testdata/profile_example.json");
    let profile: RenderProfile = serde_json::from_str(json).expect("profile json");
    GlyphRegistry::new(profile)
}

#[test]
fn put_text_respects_width_policy_and_continuation_cells() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "A🙂B".to_string(),
        style: Style::plain(),
    });

    r.apply(&f);

    let g = r.grid();

    match g.get(0, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "A");
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }

    match g.get(1, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "🙂");
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (1,0): {other:?}"),
    }

    assert_eq!(g.get(2, 0).unwrap(), &Cell::Continuation);

    match g.get(3, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "B");
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (3,0): {other:?}"),
    }
}

#[test]
fn wide_glyph_is_not_placed_if_it_would_be_half_at_right_edge() {
    let reg = load_example_registry();
    let mut r = Renderer::new(2, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "X🙂".to_string(),
        style: Style::plain(),
    });

    r.apply(&f);

    let g = r.grid();

    match g.get(0, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "X"),
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }

    // The wide glyph would start at column 1 (last col) and is not placed.
    assert_eq!(g.get(1, 0).unwrap(), &Cell::Empty);
}

#[test]
fn overwriting_a_continuation_cell_clears_the_leading_wide_glyph() {
    let reg = load_example_registry();
    let mut r = Renderer::new(4, 1, reg).expect("renderer");

    let mut f1 = Frame::default();
    f1.ops.push(RenderOp::PutGlyph {
        x: 0,
        y: 0,
        glyph: "🙂".to_string(),
        style: Style::plain(),
    });
    r.apply(&f1);

    // Overwrite the continuation at x=1 with 'X'.
    let mut f2 = Frame::default();
    f2.ops.push(RenderOp::PutGlyph {
        x: 1,
        y: 0,
        glyph: "X".to_string(),
        style: Style::plain(),
    });
    r.apply(&f2);

    let g = r.grid();
    assert_eq!(g.get(0, 0).unwrap(), &Cell::Empty);

    match g.get(1, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "X"),
        other => panic!("unexpected cell at (1,0): {other:?}"),
    }

    assert_eq!(g.get(2, 0).unwrap(), &Cell::Empty);
}

#[test]
fn fill_rect_writes_styled_spaces() {
    let reg = load_example_registry();
    let mut r = Renderer::new(4, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::FillRect {
        x: 1,
        y: 0,
        w: 2,
        h: 1,
        glyph: " ".to_string(),
        style: Style {
            fg: None,
            bg: Some(52),
            dim: false,
            bold: false,
            italic: false,
            underline: false,
            blink: false,
            inverse: false,
            strike: false,
        },
    });
    r.apply(&f);

    let g = r.grid();
    for x in 1..=2 {
        match g.get(x, 0).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, " ");
                assert_eq!(
                    *style,
                    Style {
                        fg: None,
                        bg: Some(52),
                        dim: false,
                        bold: false,
                        italic: false,
                        underline: false,
                        blink: false,
                        inverse: false,
                        strike: false
                    }
                );
            }
            other => panic!("unexpected cell at ({x},0): {other:?}"),
        }
    }
}

#[test]
fn clear_line_clears_wide_glyphs_and_writes_plain_spaces() {
    let reg = load_example_registry();
    let mut r = Renderer::new(4, 1, reg).expect("renderer");

    let mut f1 = Frame::default();
    f1.ops.push(RenderOp::PutGlyph {
        x: 0,
        y: 0,
        glyph: "🙂".to_string(),
        style: Style {
            fg: Some(15),
            bg: Some(0),
            dim: false,
            bold: true,
            italic: false,
            underline: false,
            blink: false,
            inverse: false,
            strike: false,
        },
    });
    r.apply(&f1);

    let mut f2 = Frame::default();
    f2.ops.push(RenderOp::ClearLine { y: 0 });
    r.apply(&f2);

    for x in 0..4 {
        match r.grid().get(x, 0).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, " ");
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},0): {other:?}"),
        }
    }
}

#[test]
fn clear_eol_blanks_to_end_of_line_and_handles_continuation_cursor() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 1, reg).expect("renderer");

    let mut f1 = Frame::default();
    f1.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "A🙂BC".to_string(),
        style: Style::plain(),
    });
    r.apply(&f1);

    // Cursor placed on the continuation cell of the emoji.
    let mut f2 = Frame::default();
    f2.ops.push(RenderOp::ClearEol { x: 2, y: 0 });
    r.apply(&f2);

    // 'A' remains.
    match r.grid().get(0, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "A");
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }

    // The emoji is fully removed and both cells are blanked.
    for x in 1..6 {
        match r.grid().get(x, 0).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, " ");
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},0): {other:?}"),
        }
    }
}

#[test]
fn put_styled_renders_spans_and_preserves_styles_and_continuation_cells() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::LabelStyled {
        x: 0,
        y: 0,
        w: 6,
        spans: vec![
            Span::new("A", Style::plain()),
            Span::new(
                "🙂",
                Style {
                    fg: Some(15),
                    bg: None,
                    dim: false,
                    bold: true,
                    italic: false,
                    underline: false,
                    blink: false,
                    inverse: false,
                    strike: false,
                },
            ),
            Span::new("B", Style::plain()),
        ],
        truncate: TruncateMode::Clip,
    });

    r.apply(&f);
    let g = r.grid();

    match g.get(0, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "A");
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }

    match g.get(1, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "🙂");
            assert_eq!(
                *style,
                Style {
                    fg: Some(15),
                    bg: None,
                    dim: false,
                    bold: true,
                    italic: false,
                    underline: false,
                    blink: false,
                    inverse: false,
                    strike: false,
                }
            );
        }
        other => panic!("unexpected cell at (1,0): {other:?}"),
    }

    assert_eq!(g.get(2, 0).unwrap(), &Cell::Continuation);

    match g.get(3, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "B");
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (3,0): {other:?}"),
    }
}

#[test]
fn put_wrapped_styled_wraps_at_word_boundaries_and_preserves_styles() {
    let reg = load_example_registry();
    let mut r = Renderer::new(5, 3, reg).expect("renderer");

    let styled_b = Style {
        fg: Some(15),
        bg: None,
        dim: false,
        bold: true,
        italic: false,
        underline: false,
        blink: false,
        inverse: false,
        strike: false,
    };

    let mut f = Frame::default();
    f.ops.push(RenderOp::TextBlockStyled {
        x: 0,
        y: 0,
        w: 5,
        spans: vec![
            Span::new("AAA", Style::plain()),
            Span::new(" ", Style::plain()),
            Span::new("BBB", styled_b),
            Span::new(" ", Style::plain()),
            Span::new("CCC", Style::plain()),
        ],
        wrap: WrapOpts::default(),
        h: u16::MAX,
    });
    r.apply(&f);

    // Line 1: "AAA"
    for (x, ch) in [(0, "A"), (1, "A"), (2, "A")] {
        match r.grid().get(x, 0).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, ch);
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},0): {other:?}"),
        }
    }

    // Line 2: "BBB" in styled_b
    for x in 0..3 {
        match r.grid().get(x, 1).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, "B");
                assert_eq!(*style, styled_b);
            }
            other => panic!("unexpected cell at ({x},1): {other:?}"),
        }
    }

    // Line 3: "CCC"
    for (x, ch) in [(0, "C"), (1, "C"), (2, "C")] {
        match r.grid().get(x, 2).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, ch);
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},2): {other:?}"),
        }
    }
}

#[test]
fn put_wrapped_styled_continuation_prefix_is_prepended_on_wrapped_lines() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 3, reg).expect("renderer");

    let dim = Style {
        fg: Some(8),
        bg: None,
        dim: false,
        bold: false,
        italic: false,
        underline: false,
        blink: false,
        inverse: false,
        strike: false,
    };

    let opts = WrapOpts {
        continuation_prefix: Some(vec![Span::new("↳ ", dim)]),
        ..Default::default()
    };

    let mut f = Frame::default();
    f.ops.push(RenderOp::TextBlockStyled {
        x: 0,
        y: 0,
        w: 6,
        spans: vec![Span::new("one two three", Style::plain())],
        wrap: opts,
        h: u16::MAX,
    });
    r.apply(&f);

    // Line 1: "one"
    match r.grid().get(0, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "o"),
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }

    // Line 2 begins with "↳" and a space.
    match r.grid().get(0, 1).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "↳");
            assert_eq!(*style, dim);
        }
        other => panic!("unexpected cell at (0,1): {other:?}"),
    }
    match r.grid().get(1, 1).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, " ");
            assert_eq!(*style, dim);
        }
        other => panic!("unexpected cell at (1,1): {other:?}"),
    }
}

#[test]
fn put_styled_ellipsis_truncates_and_writes_ellipsis_glyph() {
    let reg = load_example_registry();
    let mut r = Renderer::new(4, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::LabelStyled {
        x: 0,
        y: 0,
        w: 4,
        spans: vec![Span::new("ABCDEFG", Style::plain())],
        truncate: TruncateMode::Ellipsis,
    });
    r.apply(&f);

    match r.grid().get(0, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "A"),
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }
    match r.grid().get(1, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "B"),
        other => panic!("unexpected cell at (1,0): {other:?}"),
    }
    match r.grid().get(2, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "C"),
        other => panic!("unexpected cell at (2,0): {other:?}"),
    }
    match r.grid().get(3, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "…");
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (3,0): {other:?}"),
    }
}

#[test]
fn put_styled_ellipsis_is_plain_style_even_if_last_span_is_styled() {
    let reg = load_example_registry();
    let mut r = Renderer::new(4, 1, reg).expect("renderer");

    let styled = Style {
        fg: Some(196),
        bg: None,
        dim: false,
        bold: true,
        italic: false,
        underline: false,
        blink: false,
        inverse: false,
        strike: false,
    };

    let mut f = Frame::default();
    f.ops.push(RenderOp::LabelStyled {
        x: 0,
        y: 0,
        w: 4,
        spans: vec![Span::new("ABCDE", styled)],
        truncate: TruncateMode::Ellipsis,
    });
    r.apply(&f);

    match r.grid().get(3, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "…");
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (3,0): {other:?}"),
    }
}

#[test]
fn label_styled_renders_like_put_styled_and_respects_truncation() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::LabelStyled {
        x: 0,
        y: 0,
        w: 6,
        spans: vec![
            Span::new("Hi ", Style::plain()),
            Span::new(
                "🙂",
                Style {
                    fg: Some(15),
                    bg: None,
                    dim: false,
                    bold: true,
                    italic: false,
                    underline: false,
                    blink: false,
                    inverse: false,
                    strike: false,
                },
            ),
            Span::new("World", Style::plain()),
        ],
        truncate: TruncateMode::Clip,
    });
    r.apply(&f);

    // Expected visible output within 6 cells: "Hi " (3 cells) + emoji (2 cells) + "W" (1 cell)
    match r.grid().get(0, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "H"),
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }
    match r.grid().get(1, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "i"),
        other => panic!("unexpected cell at (1,0): {other:?}"),
    }
    match r.grid().get(2, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, " "),
        other => panic!("unexpected cell at (2,0): {other:?}"),
    }
    match r.grid().get(3, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert_eq!(grapheme, "🙂");
            assert_eq!(
                *style,
                Style {
                    fg: Some(15),
                    bg: None,
                    dim: false,
                    bold: true,
                    italic: false,
                    underline: false,
                    blink: false,
                    inverse: false,
                    strike: false,
                }
            );
        }
        other => panic!("unexpected cell at (3,0): {other:?}"),
    }
    assert_eq!(r.grid().get(4, 0).unwrap(), &Cell::Continuation);
    match r.grid().get(5, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "W"),
        other => panic!("unexpected cell at (5,0): {other:?}"),
    }
}

#[test]
fn clear_bol_blanks_start_of_line_through_cursor_inclusive() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 1, reg).expect("renderer");

    let mut f1 = Frame::default();
    f1.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "AB🙂BB".to_string(),
        style: Style::plain(),
    });
    r.apply(&f1);

    // Cursor is placed on the continuation cell of the emoji (x=3).
    let mut f2 = Frame::default();
    f2.ops.push(RenderOp::ClearBol { x: 3, y: 0 });
    r.apply(&f2);

    // Cells 0..=3 are blanked; trailing 'B','B' remain.
    for x in 0..=3 {
        match r.grid().get(x, 0).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, " ");
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},0): {other:?}"),
        }
    }

    for x in 4..=5 {
        match r.grid().get(x, 0).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, "B");
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},0): {other:?}"),
        }
    }
}

#[test]
fn clear_eos_blanks_from_cursor_to_end_of_screen_inclusive() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 3, reg).expect("renderer");

    let mut f1 = Frame::default();
    f1.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "AAAAAA".to_string(),
        style: Style::plain(),
    });
    f1.ops.push(RenderOp::Put {
        x: 0,
        y: 1,
        text: "BB🙂BB".to_string(),
        style: Style::plain(),
    });
    f1.ops.push(RenderOp::Put {
        x: 0,
        y: 2,
        text: "CCCCCC".to_string(),
        style: Style::plain(),
    });
    r.apply(&f1);

    // Cursor placed on the continuation cell of the emoji on row 1.
    let mut f2 = Frame::default();
    f2.ops.push(RenderOp::ClearEos { x: 3, y: 1 });
    r.apply(&f2);

    // Row 0 unchanged.
    for x in 0..6 {
        match r.grid().get(x, 0).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, "A");
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},0): {other:?}"),
        }
    }

    // Row 1: first two Bs remain, then blanks.
    for x in 0..=1 {
        match r.grid().get(x, 1).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, "B");
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},1): {other:?}"),
        }
    }
    for x in 2..6 {
        match r.grid().get(x, 1).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, " ");
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},1): {other:?}"),
        }
    }

    // Row 2 fully blanked.
    for x in 0..6 {
        match r.grid().get(x, 2).unwrap() {
            Cell::Glyph { grapheme, style } => {
                assert_eq!(grapheme, " ");
                assert_eq!(*style, Style::plain());
            }
            other => panic!("unexpected cell at ({x},2): {other:?}"),
        }
    }
}

#[test]
fn clear_rect_clears_only_the_target_region() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 2, reg).expect("renderer");

    let mut f1 = Frame::default();
    f1.ops.push(RenderOp::FillRect {
        x: 0,
        y: 0,
        w: 6,
        h: 2,
        glyph: " ".to_string(),
        style: Style {
            fg: None,
            bg: Some(52),
            dim: false,
            bold: false,
            italic: false,
            underline: false,
            blink: false,
            inverse: false,
            strike: false,
        },
    });
    f1.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "AB".to_string(),
        style: Style::plain(),
    });
    r.apply(&f1);

    let mut f2 = Frame::default();
    f2.ops.push(RenderOp::ClearRect {
        x: 1,
        y: 0,
        w: 3,
        h: 2,
    });
    r.apply(&f2);

    // (0,0) is outside the clear rect and should remain 'A'.
    match r.grid().get(0, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "A"),
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }

    // Region cleared to plain spaces.
    for yy in 0..2 {
        for xx in 1..4 {
            match r.grid().get(xx, yy).unwrap() {
                Cell::Glyph { grapheme, style } => {
                    assert_eq!(grapheme, " ");
                    assert_eq!(*style, Style::plain());
                }
                other => panic!("unexpected cell at ({xx},{yy}): {other:?}"),
            }
        }
    }
}

#[test]
fn hline_and_vline_place_repeated_glyphs() {
    let reg = load_example_registry();
    let mut r = Renderer::new(5, 4, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::FillRect {
        x: 1,
        y: 1,
        w: 3,
        h: 1,
        glyph: "─".to_string(),
        style: Style::plain(),
    });
    f.ops.push(RenderOp::FillRect {
        x: 3,
        y: 0,
        w: 1,
        h: 2,
        glyph: "│".to_string(),
        style: Style::plain(),
    });
    r.apply(&f);

    // The vertical line overwrites the intersection cell at (3,1).
    for x in 1..=2 {
        assert_eq!(
            r.grid().get(x, 1).unwrap(),
            &Cell::Glyph {
                grapheme: "─".to_string(),
                style: Style::plain()
            }
        );
    }

    assert_eq!(
        r.grid().get(3, 1).unwrap(),
        &Cell::Glyph {
            grapheme: "│".to_string(),
            style: Style::plain()
        }
    );

    for y in 0..=1 {
        assert_eq!(
            r.grid().get(3, y).unwrap(),
            &Cell::Glyph {
                grapheme: "│".to_string(),
                style: Style::plain()
            }
        );
    }
}

#[test]
fn hline_len_is_cells_for_wide_glyphs() {
    let reg = load_example_registry();
    let mut r = Renderer::new(5, 1, reg).expect("renderer");

    let mut f = Frame::default();
    // len=3 cells: only one 🙂 (2 cells) can be placed.
    f.ops.push(RenderOp::FillRect {
        x: 0,
        y: 0,
        w: 3,
        h: 1,
        glyph: "界".to_string(),
        style: Style::plain(),
    });
    r.apply(&f);

    match r.grid().get(0, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "界"),
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }
    assert_eq!(r.grid().get(1, 0).unwrap(), &Cell::Continuation);
    // The remaining cells should be empty.
    assert_eq!(r.grid().get(2, 0).unwrap(), &Cell::Empty);
}

#[test]
fn box_draws_expected_border_ascii() {
    let reg = load_example_registry();
    let mut r = Renderer::new(5, 3, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::Box {
        x: 0,
        y: 0,
        w: 5,
        h: 3,
        style: Style::plain(),
        charset: BoxCharset::Ascii,
    });
    r.apply(&f);

    let g = r.grid();

    // Corners
    assert_eq!(
        g.get(0, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "+".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(4, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "+".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(0, 2).unwrap(),
        &Cell::Glyph {
            grapheme: "+".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(4, 2).unwrap(),
        &Cell::Glyph {
            grapheme: "+".to_string(),
            style: Style::plain()
        }
    );

    // Top edge middle (some environments prefer Unicode single-line characters; accept both).
    match g.get(2, 0).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert!(
                (grapheme == "-" || grapheme == "─"),
                "unexpected top edge glyph: {grapheme}"
            );
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (2,0): {other:?}"),
    }

    // Side edge middle (some environments prefer Unicode single-line characters; accept both).
    match g.get(0, 1).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert!(
                (grapheme == "|" || grapheme == "│"),
                "unexpected side edge glyph: {grapheme}"
            );
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (0,1): {other:?}"),
    }
    match g.get(4, 1).unwrap() {
        Cell::Glyph { grapheme, style } => {
            assert!(
                (grapheme == "|" || grapheme == "│"),
                "unexpected side edge glyph: {grapheme}"
            );
            assert_eq!(*style, Style::plain());
        }
        other => panic!("unexpected cell at (4,1): {other:?}"),
    }
}

#[test]
fn ansi_emission_skips_continuation_cells_and_pads_empties() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "A🙂B".to_string(),
        style: Style::plain(),
    });
    r.apply(&f);

    let ansi = r.to_ansi();
    // Grid width is 6 columns. "A" (1) + "🙂" (2) + "B" (1) => 4, leaving 2 spaces.
    assert_eq!(ansi, "A🙂B  \x1b[0m\n");
}

#[test]
fn style_changes_emit_sgr() {
    let reg = load_example_registry();
    let mut r = Renderer::new(4, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "X".to_string(),
        style: Style {
            fg: Some(196),
            bg: None,
            dim: false,
            bold: true,
            italic: false,
            underline: false,
            blink: false,
            inverse: false,
            strike: false,
        },
    });
    r.apply(&f);

    let ansi = r.to_ansi();
    assert!(ansi.contains("\x1b[1;38;5;196m"));
}

#[test]
fn label_clips_to_width_cells() {
    let reg = load_example_registry();
    let mut r = Renderer::new(20, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::Label {
        x: 0,
        y: 0,
        w: 5,
        text: "AB🙂CD".to_string(),
        style: Style::plain(),
        truncate: TruncateMode::Clip,
    });
    r.apply(&f);

    let g = r.grid();
    assert_eq!(
        g.get(0, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "A".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(1, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "B".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(2, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "🙂".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(g.get(3, 0).unwrap(), &Cell::Continuation);
    assert_eq!(
        g.get(4, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "C".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(g.get(5, 0).unwrap(), &Cell::Empty);
}

#[test]
fn label_ellipsizes_when_truncated() {
    let reg = load_example_registry();
    let mut r = Renderer::new(10, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::Label {
        x: 0,
        y: 0,
        w: 5,
        text: "ABCDEF".to_string(),
        style: Style::plain(),
        truncate: TruncateMode::Ellipsis,
    });
    r.apply(&f);

    let g = r.grid();
    assert_eq!(
        g.get(0, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "A".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(1, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "B".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(2, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "C".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(3, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "D".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(4, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "…".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(g.get(5, 0).unwrap(), &Cell::Empty);
}

#[test]
fn put_wrapped_wraps_words() {
    let reg = load_example_registry();
    let mut r = Renderer::new(10, 3, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::TextBlock {
        x: 0,
        y: 0,
        w: 6,

        h: u16::MAX,
        text: "Hello world".to_string(),
        style: Style::plain(),
        wrap: WrapOpts::default(),
    });
    r.apply(&f);

    let g = r.grid();
    // Row 0: "Hello"
    assert_eq!(
        g.get(0, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "H".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(1, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "e".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(2, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "l".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(3, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "l".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(4, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "o".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(g.get(5, 0).unwrap(), &Cell::Empty);

    // Row 1: "world"
    assert_eq!(
        g.get(0, 1).unwrap(),
        &Cell::Glyph {
            grapheme: "w".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(1, 1).unwrap(),
        &Cell::Glyph {
            grapheme: "o".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(2, 1).unwrap(),
        &Cell::Glyph {
            grapheme: "r".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(3, 1).unwrap(),
        &Cell::Glyph {
            grapheme: "l".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(4, 1).unwrap(),
        &Cell::Glyph {
            grapheme: "d".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(g.get(5, 1).unwrap(), &Cell::Empty);

    // Row 2 untouched
    assert_eq!(g.get(0, 2).unwrap(), &Cell::Empty);
}

#[test]
fn blit_transparent_cells_and_wide_glyph_skip_next_source_cell() {
    let reg = load_example_registry();
    let mut r = Renderer::new(8, 1, reg).expect("renderer");

    // Baseline content.
    let mut base = Frame::default();
    base.ops.push(RenderOp::Put {
        x: 0,
        y: 0,
        text: "ABCDEFGH".to_string(),
        style: Style::plain(),
    });
    r.apply(&base);

    // Blit a 4x1 sprite at x=2. Cell 0 is transparent, cell 1 is wide, cell 2 is ignored,
    // cell 3 overwrites destination.
    let mut f = Frame::default();
    f.ops.push(RenderOp::Blit {
        x: 2,
        y: 0,
        w: 4,
        h: 1,
        cells: vec![
            None,
            Some(BlitCell {
                // Use a CJK glyph which is reliably wide (2 cells) under wcwidth.
                glyph: "界".to_string(),
                style: Style {
                    fg: None,
                    bg: None,
                    dim: false,
                    bold: true,
                    italic: false,
                    underline: false,
                    blink: false,
                    inverse: false,
                    strike: false,
                },
            }),
            Some(BlitCell {
                glyph: "X".to_string(),
                style: Style::plain(),
            }),
            Some(BlitCell {
                glyph: "|".to_string(),
                style: Style::plain(),
            }),
        ],
    });
    r.apply(&f);

    let g = r.grid();

    // A B C are preserved (cell 0 is transparent over C).
    assert_eq!(
        g.get(0, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "A".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(1, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "B".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(2, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "C".to_string(),
            style: Style::plain()
        }
    );

    // Wide glyph placed at destination x=3 consumes x=4.
    assert_eq!(
        g.get(3, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "界".to_string(),
            style: Style {
                fg: None,
                bg: None,
                dim: false,
                bold: true,
                italic: false,
                underline: false,
                blink: false,
                inverse: false,
                strike: false
            },
        }
    );
    assert_eq!(g.get(4, 0).unwrap(), &Cell::Continuation);

    // Source cell 2 ("X") is ignored because the wide glyph consumed it.
    // Cell 3 overwrites F at destination x=5.
    assert_eq!(
        g.get(5, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "|".to_string(),
            style: Style::plain()
        }
    );

    // Tail preserved.
    assert_eq!(
        g.get(6, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "G".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(7, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "H".to_string(),
            style: Style::plain()
        }
    );
}

#[test]
fn put_wrapped_hard_breaks_long_word() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 3, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::TextBlock {
        x: 0,
        y: 0,
        w: 4,

        h: u16::MAX,
        // Long word: wraps by hard-breaking to width.
        text: "ABCDEF".to_string(),
        style: Style::plain(),
        wrap: WrapOpts::default(),
    });
    r.apply(&f);

    let g = r.grid();
    // Row 0: "ABCD"
    assert_eq!(
        g.get(0, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "A".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(1, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "B".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(2, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "C".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(3, 0).unwrap(),
        &Cell::Glyph {
            grapheme: "D".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(g.get(4, 0).unwrap(), &Cell::Empty);

    // Row 1: "EF"
    assert_eq!(
        g.get(0, 1).unwrap(),
        &Cell::Glyph {
            grapheme: "E".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(
        g.get(1, 1).unwrap(),
        &Cell::Glyph {
            grapheme: "F".to_string(),
            style: Style::plain()
        }
    );
    assert_eq!(g.get(2, 1).unwrap(), &Cell::Empty);
}

#[test]
fn put_wrapped_styled_respects_max_lines_and_does_not_write_beyond_limit() {
    let reg = load_example_registry();
    let mut r = Renderer::new(12, 4, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::TextBlockStyled {
        x: 0,
        y: 0,
        w: 10,
        h: 2,
        spans: vec![Span::new(
            "One two three four five six seven eight nine ten.",
            Style::plain(),
        )],
        wrap: WrapOpts {
            continuation_prefix: Some(vec![Span::new(
                "↳ ",
                Style {
                    fg: None,
                    bg: None,
                    dim: true,
                    bold: false,
                    italic: false,
                    underline: false,
                    blink: false,
                    inverse: false,
                    strike: false,
                },
            )]),
            ..WrapOpts::default()
        },
    });

    r.apply(&f);

    // Lines y=0 and y=1 should have some content.
    let mut any0 = false;
    for x in 0..12 {
        if r.grid().get(x, 0).unwrap() != &Cell::Empty {
            any0 = true;
            break;
        }
    }
    assert_eq!(any0, true);

    let mut any1 = false;
    for x in 0..12 {
        if r.grid().get(x, 1).unwrap() != &Cell::Empty {
            any1 = true;
            break;
        }
    }
    assert_eq!(any1, true);

    // Lines beyond max_lines should remain untouched (Empty).
    for y in 2..4 {
        for x in 0..12 {
            assert_eq!(r.grid().get(x, y).unwrap(), &Cell::Empty);
        }
    }
}

#[test]
fn put_wrapped_styled_with_zero_width_renders_nothing() {
    let reg = load_example_registry();
    let mut r = Renderer::new(8, 2, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::TextBlockStyled {
        x: 0,
        y: 0,
        w: 0,
        spans: vec![Span::new("Hello", Style::plain())],
        wrap: WrapOpts::default(),
        h: u16::MAX,
    });

    r.apply(&f);

    for y in 0..2 {
        for x in 0..8 {
            assert_eq!(r.grid().get(x, y).unwrap(), &Cell::Empty);
        }
    }
}

#[test]
fn put_styled_respects_w_hard_bound() {
    let reg = load_example_registry();
    let mut r = Renderer::new(8, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::LabelStyled {
        x: 0,
        y: 0,
        w: 3,
        spans: vec![Span::new("ABCDEFG", Style::plain())],
        truncate: TruncateMode::Clip,
    });

    r.apply(&f);

    let g = r.grid();

    match g.get(0, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "A"),
        other => panic!("unexpected cell at (0,0): {other:?}"),
    }
    match g.get(1, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "B"),
        other => panic!("unexpected cell at (1,0): {other:?}"),
    }
    match g.get(2, 0).unwrap() {
        Cell::Glyph { grapheme, .. } => assert_eq!(grapheme, "C"),
        other => panic!("unexpected cell at (2,0): {other:?}"),
    }

    // Hard bound: nothing rendered beyond w=3.
    for x in 3..8 {
        assert_eq!(g.get(x, 0).unwrap(), &Cell::Empty);
    }
}

#[test]
fn put_wrapped_styled_respects_w_hard_bound() {
    let reg = load_example_registry();
    let mut r = Renderer::new(6, 2, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::TextBlockStyled {
        x: 0,
        y: 0,
        w: 4,
        spans: vec![Span::new("ABCDE", Style::plain())],
        wrap: WrapOpts::default(),
        h: u16::MAX,
    });

    r.apply(&f);

    let g = r.grid();

    // First visual line should occupy only columns 0..3.
    for x in 4..6 {
        assert_eq!(g.get(x, 0).unwrap(), &Cell::Empty);
    }
    // Second line (continuation) should also respect the same bound.
    for x in 4..6 {
        assert_eq!(g.get(x, 1).unwrap(), &Cell::Empty);
    }
}

#[test]
fn label_respects_w_hard_bound() {
    let reg = load_example_registry();
    let mut r = Renderer::new(8, 1, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::Label {
        x: 0,
        y: 0,
        w: 3,
        text: "ABCDEFG".to_string(),
        style: Style::plain(),
        truncate: TruncateMode::Clip,
    });

    r.apply(&f);

    let g = r.grid();
    for x in 3..8 {
        assert_eq!(g.get(x, 0).unwrap(), &Cell::Empty);
    }
}

#[test]
fn put_wrapped_respects_w_hard_bound() {
    let reg = load_example_registry();
    let mut r = Renderer::new(8, 2, reg).expect("renderer");

    let mut f = Frame::default();
    f.ops.push(RenderOp::TextBlock {
        x: 0,
        y: 0,
        w: 3,

        h: u16::MAX,
        text: "ABCDEFG".to_string(),
        style: Style::plain(),
        wrap: WrapOpts::default(),
    });

    r.apply(&f);

    let g = r.grid();

    // Both lines must leave columns >= w empty.
    for x in 3..8 {
        assert_eq!(g.get(x, 0).unwrap(), &Cell::Empty);
        assert_eq!(g.get(x, 1).unwrap(), &Cell::Empty);
    }
}
