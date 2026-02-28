use termgrid_core::{
    apply_highlight, match_positions_graphemes, Cell, Frame, GlyphRegistry, RenderOp,
    RenderProfile, Renderer, Span, Style, TruncateMode,
};

fn registry() -> GlyphRegistry {
    let mut p = RenderProfile::empty("example", 1);
    p.set_width("🧠", 2);
    p.set_width("🎧", 2);
    GlyphRegistry::new(p)
}

fn cell_glyph(c: &Cell) -> &str {
    match c {
        Cell::Empty => " ",
        Cell::Continuation => "·",
        Cell::Glyph { grapheme, .. } => grapheme,
    }
}

fn positions_to_ranges(pos: &[usize]) -> Vec<(usize, usize)> {
    if pos.is_empty() {
        return Vec::new();
    }
    let mut p = pos.to_vec();
    p.sort_unstable();
    let mut out = Vec::new();
    let mut start = p[0];
    let mut prev = p[0];
    for &i in p.iter().skip(1) {
        if i == prev + 1 {
            prev = i;
            continue;
        }
        out.push((start, prev + 1));
        start = i;
        prev = i;
    }
    out.push((start, prev + 1));
    out
}

fn main() {
    let reg = registry();
    let mut r = Renderer::new(64, 14, reg).expect("valid grid");

    let query = "meme";
    let candidates = [
        "Memetics Quarantine 🧠 — handling rules and overrides",
        "Acoustic Containment 🎧 — do not whistle",
        "Temporal Stabilization — clock discipline",
        "General Stacks — boring, safe, probably",
    ];

    let mut frame = Frame::default();
    frame.ops.push(RenderOp::Clear);
    frame.ops.push(RenderOp::Box {
        x: 0,
        y: 0,
        w: 64,
        h: 14,
        charset: Default::default(),
        style: Style::plain(),
    });

    frame.ops.push(RenderOp::Label {
        x: 2,
        y: 1,
        w: 60,
        text: format!("SEARCH (fzf)  query: {}", query),
        style: Style::plain(),
        truncate: TruncateMode::Clip,
    });

    let hl = Style {
        inverse: true,
        ..Style::plain()
    };

    let mut y = 3u16;
    for (idx, line) in candidates.iter().enumerate() {
        let base = vec![Span::new(*line, Style::plain())];
        let spans = if let Some(pos) = match_positions_graphemes(query, line) {
            let ranges = positions_to_ranges(&pos);
            apply_highlight(&base, &ranges, hl)
        } else {
            base
        };

        frame.ops.push(RenderOp::PutStyled {
            x: 2,
            y,
            w: 60,
            spans,
            truncate: TruncateMode::Ellipsis,
        });

        // A crude selection marker.
        if idx == 0 {
            frame.ops.push(RenderOp::PutGlyph {
                x: 1,
                y,
                glyph: ">".to_string(),
                style: Style::plain(),
            });
        }

        y += 1;
    }

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
