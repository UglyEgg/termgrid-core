use crate::{
    ansi,
    damage::{Damage, Rect},
    BlitCell, BoxCharset, Cell, Frame, GlyphRegistry, Grid, RenderOp, Style, TruncateMode,
};

const DEFAULT_ELLIPSIS: &str = "…";
const DAMAGE_MAX_RECTS: usize = 64;
use crate::text::{
    clip_to_cells_spans, clip_to_cells_text, ellipsis_to_cells_spans, ellipsis_to_cells_text,
    normalize_spans, wrap_spans_wordwise, Span, WrapOpts,
};
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("invalid grid size")]
    InvalidGridSize,
}

/// A stateful renderer: owns a grid and applies frames to it.
#[derive(Debug, Clone)]
pub struct Renderer {
    grid: Grid,
    reg: GlyphRegistry,
}

impl Renderer {
    pub fn new(width: u16, height: u16, reg: GlyphRegistry) -> Result<Self, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidGridSize);
        }
        Ok(Self {
            grid: Grid::new(width, height),
            reg,
        })
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    pub fn apply(&mut self, frame: &Frame) {
        for op in &frame.ops {
            self.apply_op(op);
        }
    }

    /// Apply a frame and return the damaged regions.
    ///
    /// Damage is conservative and derived from the operations applied. It is
    /// suitable for incremental terminal backends that want to minimize redraw
    /// and flicker.
    pub fn apply_with_damage(&mut self, frame: &Frame) -> Damage {
        let mut dmg = Damage::empty();
        for op in &frame.ops {
            self.apply_op(op);
            self.note_damage_for_op(op, &mut dmg);
            if dmg.full_redraw {
                break;
            }
        }
        dmg
    }

    fn note_damage_for_op(&self, op: &RenderOp, dmg: &mut Damage) {
        let w = self.grid.width;
        let h = self.grid.height;

        fn clip_rect(r: Rect, gw: u16, gh: u16) -> Rect {
            if r.w == 0 || r.h == 0 {
                return Rect::new(0, 0, 0, 0);
            }
            let x1 = r.x.min(gw);
            let y1 = r.y.min(gh);
            let x2 = r.right().min(gw);
            let y2 = r.bottom().min(gh);
            Rect::new(x1, y1, x2.saturating_sub(x1), y2.saturating_sub(y1))
        }

        let mut push = |r: Rect| {
            let r = clip_rect(r, w, h);
            dmg.push_rect(r, DAMAGE_MAX_RECTS);
        };

        match op {
            RenderOp::Clear => {
                dmg.full_redraw = true;
                dmg.rects.clear();
            }
            RenderOp::ClearLine { y } => push(Rect::new(0, *y, w, 1)),
            RenderOp::ClearEol { x, y } => push(Rect::new(*x, *y, w.saturating_sub(*x), 1)),
            RenderOp::ClearBol { x, y } => push(Rect::new(0, *y, x.saturating_add(1), 1)),
            RenderOp::ClearEos { x, y } => {
                // First line from x..end, then remaining full lines.
                push(Rect::new(*x, *y, w.saturating_sub(*x), 1));
                if y.saturating_add(1) < h {
                    push(Rect::new(
                        0,
                        y.saturating_add(1),
                        w,
                        h.saturating_sub(y.saturating_add(1)),
                    ));
                }
            }
            RenderOp::ClearRect { x, y, w: rw, h: rh } => push(Rect::new(*x, *y, *rw, *rh)),
            RenderOp::FillRect {
                x, y, w: rw, h: rh, ..
            } => push(Rect::new(*x, *y, *rw, *rh)),
            RenderOp::Put { x, y, text, .. } => {
                let mw = crate::text::measure_cells_text(&self.reg, text) as u16;
                push(Rect::new(*x, *y, mw, 1));
            }
            RenderOp::PutGlyph { x, y, glyph, .. } => {
                let mw = crate::text::measure_cells_text(&self.reg, glyph) as u16;
                push(Rect::new(*x, *y, mw, 1));
            }
            RenderOp::Label { x, y, w: lw, .. } => push(Rect::new(*x, *y, *lw, 1)),
            RenderOp::LabelStyled { x, y, w: lw, .. } => push(Rect::new(*x, *y, *lw, 1)),
            RenderOp::PutStyled { x, y, w: lw, .. } => push(Rect::new(*x, *y, *lw, 1)),
            RenderOp::PutWrapped {
                x, y, w: lw, text, ..
            } => {
                let lines = self.estimate_wrapped_lines(text, *lw);
                push(Rect::new(*x, *y, *lw, lines));
            }
            RenderOp::PutWrappedStyled {
                x,
                y,
                w: lw,
                spans,
                wrap_opts,
                max_lines,
            } => {
                let lines = self.estimate_wrapped_spans_lines(spans, *lw, wrap_opts, *max_lines);
                push(Rect::new(*x, *y, *lw, lines));
            }
            RenderOp::Blit {
                x, y, w: bw, h: bh, ..
            } => push(Rect::new(*x, *y, *bw, *bh)),
            RenderOp::HLine { x, y, len, .. } => push(Rect::new(*x, *y, *len, 1)),
            RenderOp::VLine { x, y, len, .. } => push(Rect::new(*x, *y, 1, *len)),
            RenderOp::Box {
                x, y, w: bw, h: bh, ..
            } => push(Rect::new(*x, *y, *bw, *bh)),
        }
    }

    fn estimate_wrapped_lines(&self, text: &str, w: u16) -> u16 {
        if w == 0 {
            return 0;
        }
        if text.is_empty() {
            return 0;
        }
        let max_w = w.min(self.grid.width);
        if max_w == 0 {
            return 0;
        }
        let mut lines: u16 = 0;
        for para in text.split(['\n', '\r']) {
            if para.is_empty() {
                continue;
            }
            for _ in wrap_text_wordwise(para, max_w, &self.reg) {
                lines = lines.saturating_add(1);
                if lines == u16::MAX {
                    return lines;
                }
            }
        }
        lines.max(1)
    }

    fn estimate_wrapped_spans_lines(
        &self,
        spans: &[Span],
        w: u16,
        wrap_opts: &WrapOpts,
        max_lines: Option<u16>,
    ) -> u16 {
        if w == 0 {
            return 0;
        }
        let max_w = w.min(self.grid.width);
        if max_w == 0 {
            return 0;
        }
        let lines = wrap_spans_wordwise(&self.reg, spans, max_w as usize, wrap_opts);
        let mut n = lines.len() as u16;
        if let Some(lim) = max_lines {
            n = n.min(lim);
        }
        n
    }

    #[cfg(feature = "debug-validate")]
    #[inline]
    fn debug_validate_after(&self, op: &RenderOp) {
        if let Err(e) = self.grid.validate_invariants(&self.reg) {
            // Pointer breadcrumb avoids requiring RenderOp: Debug while still attributing
            // the invariant failure to a specific operation.
            panic!(
                "termgrid-core invariant violation after op_ptr={:p}: {:?}",
                op, e
            );
        }
    }

    #[cfg(not(feature = "debug-validate"))]
    #[inline]
    fn debug_validate_after(&self, _op: &RenderOp) {
        // No-op when invariant enforcement is disabled.
    }

    pub fn apply_op(&mut self, op: &RenderOp) {
        match op {
            RenderOp::Clear => self.grid.clear(),

            RenderOp::ClearLine { y } => self.clear_line(*y),

            RenderOp::ClearEol { x, y } => self.clear_eol(*x, *y),

            RenderOp::ClearBol { x, y } => self.clear_bol(*x, *y),

            RenderOp::ClearEos { x, y } => self.clear_eos(*x, *y),

            RenderOp::ClearRect { x, y, w, h } => self.clear_rect(*x, *y, *w, *h),

            RenderOp::Put { x, y, text, style } => {
                let s: Style = *style;
                self.grid.put_text(*x, *y, text, s, &self.reg);
            }

            RenderOp::PutGlyph { x, y, glyph, style } => {
                let s: Style = *style;
                self.grid.put_text(*x, *y, glyph, s, &self.reg);
            }

            RenderOp::Label {
                x,
                y,
                w,
                text,
                style,
                truncate,
            } => {
                self.label(*x, *y, *w, text, *style, *truncate);
            }

            RenderOp::LabelStyled {
                x,
                y,
                w,
                spans,
                truncate,
            } => {
                self.put_styled(*x, *y, *w, spans, *truncate);
            }

            RenderOp::PutStyled {
                x,
                y,
                w,
                spans,
                truncate,
            } => {
                self.put_styled(*x, *y, *w, spans, *truncate);
            }

            RenderOp::PutWrapped {
                x,
                y,
                w,
                text,
                style,
            } => {
                self.put_wrapped(*x, *y, *w, text, *style);
            }

            RenderOp::PutWrappedStyled {
                x,
                y,
                w,
                spans,
                wrap_opts,
                max_lines,
            } => {
                self.put_wrapped_styled(*x, *y, *w, spans, wrap_opts, *max_lines);
            }

            RenderOp::Blit { x, y, w, h, cells } => {
                self.blit(*x, *y, *w, *h, cells);
            }

            RenderOp::FillRect { x, y, w, h, style } => {
                self.fill_rect(*x, *y, *w, *h, *style);
            }

            RenderOp::HLine {
                x,
                y,
                len,
                glyph,
                style,
            } => {
                self.hline(*x, *y, *len, glyph, *style);
            }

            RenderOp::VLine {
                x,
                y,
                len,
                glyph,
                style,
            } => {
                self.vline(*x, *y, *len, glyph, *style);
            }

            RenderOp::Box {
                x,
                y,
                w,
                h,
                style,
                charset,
            } => {
                self.draw_box(*x, *y, *w, *h, *style, *charset);
            }
        }
        self.debug_validate_after(op);
    }

    fn clear_line(&mut self, y: u16) {
        if y >= self.grid.height {
            return;
        }
        let line = " ".repeat(self.grid.width as usize);
        self.grid.put_text(0, y, &line, Style::plain(), &self.reg);
    }

    fn clear_eol(&mut self, x: u16, y: u16) {
        if y >= self.grid.height || x >= self.grid.width {
            return;
        }

        // If x is the continuation half of a wide glyph, blank the leading half too.
        if x > 0 && matches!(self.grid.get(x, y), Some(Cell::Continuation)) {
            self.grid.put_text(x - 1, y, " ", Style::plain(), &self.reg);
        }

        let count = (self.grid.width - x) as usize;
        let line = " ".repeat(count);
        self.grid.put_text(x, y, &line, Style::plain(), &self.reg);
    }

    fn clear_bol(&mut self, x: u16, y: u16) {
        if y >= self.grid.height || x >= self.grid.width {
            return;
        }
        let count = (x + 1) as usize;
        let line = " ".repeat(count);
        self.grid.put_text(0, y, &line, Style::plain(), &self.reg);
    }

    fn clear_eos(&mut self, x: u16, y: u16) {
        if y >= self.grid.height || x >= self.grid.width {
            return;
        }
        self.clear_eol(x, y);
        for yy in (y + 1)..self.grid.height {
            self.clear_line(yy);
        }
    }

    fn clear_rect(&mut self, x: u16, y: u16, w: u16, h: u16) {
        self.fill_rect(x, y, w, h, Style::plain());
    }

    fn label(&mut self, x: u16, y: u16, w: u16, text: &str, style: Style, truncate: TruncateMode) {
        if w == 0 {
            return;
        }
        if x >= self.grid.width || y >= self.grid.height {
            return;
        }
        let max_w = w.min(self.grid.width - x);
        if max_w == 0 {
            return;
        }

        // Stop at the first newline, if present.
        let line = text.split(['\n', '\r']).next().unwrap_or("");

        let rendered = match truncate {
            TruncateMode::Clip => clip_to_cells_text(&self.reg, line, max_w as usize).0,
            TruncateMode::Ellipsis => {
                ellipsis_to_cells_text(&self.reg, line, max_w as usize, DEFAULT_ELLIPSIS)
            }
        };

        let spans = [Span::new(rendered, style)];
        self.put_spans_line(x, y, max_w, &spans);
    }

    fn put_wrapped(&mut self, x: u16, y: u16, w: u16, text: &str, style: Style) {
        if w == 0 {
            return;
        }
        if x >= self.grid.width || y >= self.grid.height {
            return;
        }
        let max_w = w.min(self.grid.width - x);
        if max_w == 0 {
            return;
        }

        let mut cy = y;
        for para in text.split(['\n', '\r']) {
            if cy >= self.grid.height {
                break;
            }
            // Wrap each paragraph independently; blank paragraphs advance a line.
            if para.is_empty() {
                cy = cy.saturating_add(1);
                continue;
            }

            for line in wrap_text_wordwise(para, max_w, &self.reg) {
                if cy >= self.grid.height {
                    break;
                }
                let spans = [Span::new(line, style)];
                self.put_spans_line(x, cy, max_w, &spans);
                cy = cy.saturating_add(1);
            }
            // Paragraph boundary: advance one line between paragraphs if there are more.
            // The split() iterator discards delimiters; we keep it simple and do not add
            // an extra blank line here beyond what empty paras already produce.
        }
    }

    fn put_wrapped_styled(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        spans: &[Span],
        wrap_opts: &WrapOpts,
        max_lines: Option<u16>,
    ) {
        if w == 0 {
            return;
        }
        if x >= self.grid.width || y >= self.grid.height {
            return;
        }
        let max_w = w.min(self.grid.width - x);
        if max_w == 0 {
            return;
        }

        let lines = wrap_spans_wordwise(&self.reg, spans, max_w as usize, wrap_opts);
        let limit = max_lines.map(|v| v as usize);

        let mut cy = y;
        for (rendered, line_spans) in lines.into_iter().enumerate() {
            if cy >= self.grid.height {
                break;
            }
            if let Some(lim) = limit {
                if rendered >= lim {
                    break;
                }
            }
            self.put_spans_line(x, cy, max_w, &line_spans);
            cy = cy.saturating_add(1);
        }
    }

    fn put_styled(&mut self, x: u16, y: u16, w: u16, spans: &[Span], truncate: TruncateMode) {
        if w == 0 {
            return;
        }
        if x >= self.grid.width || y >= self.grid.height {
            return;
        }
        let max_w = w.min(self.grid.width - x);
        if max_w == 0 {
            return;
        }

        let spans = normalize_spans(spans);

        let rendered_spans = match truncate {
            TruncateMode::Clip => {
                let (s, _clipped) = clip_to_cells_spans(&self.reg, &spans, max_w as usize);
                s
            }
            TruncateMode::Ellipsis => {
                // Deterministic: ellipsis is always rendered in plain style.
                // Callers who want a styled ellipsis should construct it themselves
                // by pre-ellipsizing spans before emitting render ops.
                let ell = Span::new(DEFAULT_ELLIPSIS, Style::plain());
                ellipsis_to_cells_spans(&self.reg, &spans, max_w as usize, &ell)
            }
        };

        self.put_spans_line(x, y, max_w, &rendered_spans);
    }

    fn put_spans_line(&mut self, x: u16, y: u16, max_w: u16, spans: &[Span]) {
        if max_w == 0 {
            return;
        }
        let limit_x = x.saturating_add(max_w);

        let mut cx = x;
        for s in spans {
            for g in s.text.graphemes(true) {
                // Hard cap at the requested width, independent of grid width.
                if cx >= limit_x {
                    return;
                }
                if cx >= self.grid.width {
                    return;
                }
                let gw = self.reg.width(g);
                if gw == 0 {
                    continue;
                }
                // Avoid placing a half-wide glyph either at the grid edge or beyond the op width.
                if gw == 2 {
                    if cx + 1 >= self.grid.width {
                        return;
                    }
                    if cx + 1 >= limit_x {
                        return;
                    }
                }
                self.grid.put_text(cx, y, g, s.style, &self.reg);
                cx = cx.saturating_add(gw as u16);
            }
        }
    }

    fn blit(&mut self, x: u16, y: u16, w: u16, h: u16, cells: &[Option<BlitCell>]) {
        if w == 0 || h == 0 {
            return;
        }
        if x >= self.grid.width || y >= self.grid.height {
            return;
        }

        let max_w = w.min(self.grid.width.saturating_sub(x));
        let max_h = h.min(self.grid.height.saturating_sub(y));
        if max_w == 0 || max_h == 0 {
            return;
        }

        let src_w = w as usize;

        for sy in 0..max_h {
            let ty = y.saturating_add(sy);
            let mut sx: u16 = 0;

            while sx < max_w {
                let idx = sy as usize * src_w + sx as usize;
                if idx >= cells.len() {
                    break;
                }

                let tx = x.saturating_add(sx);
                if tx >= self.grid.width {
                    break;
                }

                if let Some(cell) = &cells[idx] {
                    let style = cell.style;
                    let glyph = cell.glyph.as_str();
                    self.grid.put_text(tx, ty, glyph, style, &self.reg);

                    let gw = self.reg.width(glyph) as u16;
                    if gw == 2 {
                        // A wide glyph consumes two cells; ignore the next source cell.
                        sx = sx.saturating_add(2);
                        continue;
                    }
                }
                sx = sx.saturating_add(1);
            }
        }
    }

    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, style: Style) {
        if w == 0 || h == 0 {
            return;
        }
        if x >= self.grid.width || y >= self.grid.height {
            return;
        }

        let max_w = self.grid.width.saturating_sub(x);
        let max_h = self.grid.height.saturating_sub(y);
        let w = w.min(max_w);
        let h = h.min(max_h);
        if w == 0 || h == 0 {
            return;
        }

        let line = " ".repeat(w as usize);
        for dy in 0..h {
            let yy = y.saturating_add(dy);
            self.grid.put_text(x, yy, &line, style, &self.reg);
        }
    }

    fn hline(&mut self, x: u16, y: u16, len_cells: u16, glyph: &str, style: Style) {
        if len_cells == 0 {
            return;
        }
        if y >= self.grid.height {
            return;
        }
        if x >= self.grid.width {
            return;
        }

        let gw = self.reg.width(glyph) as u16;
        let gw = gw.max(1);

        let mut cx = x;
        let mut remaining = len_cells;
        while remaining > 0 && cx < self.grid.width {
            if gw == 2 {
                if remaining < 2 {
                    break;
                }
                if cx + 1 >= self.grid.width {
                    break;
                }
                self.grid.put_text(cx, y, glyph, style, &self.reg);
                cx = cx.saturating_add(2);
                remaining = remaining.saturating_sub(2);
            } else {
                self.grid.put_text(cx, y, glyph, style, &self.reg);
                cx = cx.saturating_add(1);
                remaining = remaining.saturating_sub(1);
            }
        }
    }

    fn vline(&mut self, x: u16, y: u16, len_rows: u16, glyph: &str, style: Style) {
        if len_rows == 0 {
            return;
        }
        if x >= self.grid.width {
            return;
        }
        if y >= self.grid.height {
            return;
        }

        // If the glyph is wide, ensure we won't place half at the right edge.
        if self.reg.width(glyph) == 2 && x + 1 >= self.grid.width {
            return;
        }

        let max_len = self.grid.height.saturating_sub(y);
        let len = len_rows.min(max_len);
        for dy in 0..len {
            let yy = y.saturating_add(dy);
            self.grid.put_text(x, yy, glyph, style, &self.reg);
        }
    }

    fn draw_box(&mut self, x: u16, y: u16, w: u16, h: u16, style: Style, charset: BoxCharset) {
        if w < 2 || h < 2 {
            return;
        }
        if x >= self.grid.width || y >= self.grid.height {
            return;
        }

        let x1 = x.saturating_add(w - 1).min(self.grid.width - 1);
        let y1 = y.saturating_add(h - 1).min(self.grid.height - 1);

        if x1 <= x || y1 <= y {
            return;
        }

        let (tl, tr, bl, br, hz, vt) = match charset {
            BoxCharset::Ascii => ("+", "+", "+", "+", "-", "|"),
            BoxCharset::UnicodeSingle => ("┌", "┐", "└", "┘", "─", "│"),
            BoxCharset::UnicodeDouble => ("╔", "╗", "╚", "╝", "═", "║"),
        };

        // Corners
        self.grid.put_text(x, y, tl, style, &self.reg);
        self.grid.put_text(x1, y, tr, style, &self.reg);
        self.grid.put_text(x, y1, bl, style, &self.reg);
        self.grid.put_text(x1, y1, br, style, &self.reg);

        // Horizontal lines
        if x1 > x + 1 {
            let count = (x1 - x - 1) as usize;
            let line = hz.repeat(count);
            self.grid.put_text(x + 1, y, &line, style, &self.reg);
            self.grid.put_text(x + 1, y1, &line, style, &self.reg);
        }

        // Vertical lines
        if y1 > y + 1 {
            for yy in (y + 1)..y1 {
                self.grid.put_text(x, yy, vt, style, &self.reg);
                self.grid.put_text(x1, yy, vt, style, &self.reg);
            }
        }
    }

    /// Renders the current grid to an ANSI string.
    pub fn to_ansi(&self) -> String {
        ansi::grid_to_ansi(&self.grid)
    }
}

fn clip_to_cells(text: &str, max_cells: u16, reg: &GlyphRegistry) -> String {
    if max_cells == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used: u16 = 0;
    for g in UnicodeSegmentation::graphemes(text, true) {
        let gw = reg.width(g) as u16;
        if used + gw > max_cells {
            break;
        }
        // Avoid placing half of a wide glyph in the last cell of the clip region.
        if gw == 2 && used + 1 == max_cells {
            break;
        }
        out.push_str(g);
        used += gw.max(1);
        if used >= max_cells {
            break;
        }
    }
    out
}

fn visible_width_cells(text: &str, reg: &GlyphRegistry) -> u16 {
    let mut w: u16 = 0;
    for g in UnicodeSegmentation::graphemes(text, true) {
        if g == "\n" || g == "\r" {
            break;
        }
        w = w.saturating_add(reg.width(g) as u16);
    }
    w
}

fn wrap_text_wordwise(text: &str, max_cells: u16, reg: &GlyphRegistry) -> Vec<String> {
    // Tokenize into runs of whitespace vs non-whitespace graphemes.
    #[derive(Clone)]
    struct Tok {
        s: String,
        is_space: bool,
    }

    let mut toks: Vec<Tok> = Vec::new();
    let mut cur = String::new();
    let mut cur_is_space: Option<bool> = None;

    for g in UnicodeSegmentation::graphemes(text, true) {
        let is_space = g.chars().all(|c| c.is_whitespace());
        match cur_is_space {
            None => {
                cur_is_space = Some(is_space);
                cur.push_str(g);
            }
            Some(same) if same == is_space => {
                cur.push_str(g);
            }
            Some(_) => {
                toks.push(Tok {
                    s: cur.clone(),
                    is_space: cur_is_space.unwrap(),
                });
                cur.clear();
                cur_is_space = Some(is_space);
                cur.push_str(g);
            }
        }
    }
    if !cur.is_empty() {
        toks.push(Tok {
            s: cur,
            is_space: cur_is_space.unwrap_or(false),
        });
    }

    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut used: u16 = 0;

    fn push_line(lines: &mut Vec<String>, line: &mut String, used: &mut u16) {
        lines.push(std::mem::take(line));
        *used = 0;
    }

    let mut i = 0;
    while i < toks.len() {
        let tok = toks[i].clone();
        if tok.is_space {
            // Skip leading spaces.
            if used == 0 {
                i += 1;
                continue;
            }
            // Collapse spaces to a single space to keep UI sane.
            let s = " ";
            let sw = reg.width(s) as u16;

            // Avoid writing trailing spaces at end-of-line.
            // If the next word would not fit after a space, wrap before the word.
            // This keeps lines like "Hello" instead of "Hello ".
            let mut next_word_w: Option<u16> = None;
            for tok in toks.iter().skip(i + 1) {
                if !tok.is_space {
                    next_word_w = Some(visible_width_cells(&tok.s, reg));
                    break;
                }
            }

            if used + sw > max_cells {
                push_line(&mut lines, &mut line, &mut used);
            } else if let Some(ww) = next_word_w {
                if used + sw + ww > max_cells {
                    // Wrap before the next word; do not emit a trailing space.
                    push_line(&mut lines, &mut line, &mut used);
                } else {
                    line.push_str(s);
                    used += sw.max(1);
                }
            } else {
                // Trailing whitespace at end of input: ignore.
            }
            i += 1;
            continue;
        }

        let word = tok.s;
        let ww = visible_width_cells(&word, reg);
        if ww <= max_cells {
            if used == 0 {
                line.push_str(&word);
                used = ww;
            } else if used + ww <= max_cells {
                line.push_str(&word);
                used += ww;
            } else {
                push_line(&mut lines, &mut line, &mut used);
                line.push_str(&word);
                used = ww;
            }
            i += 1;
            continue;
        }

        // Word longer than line: hard-break.
        let mut remainder = word.as_str();
        loop {
            if remainder.is_empty() {
                break;
            }
            if used != 0 {
                push_line(&mut lines, &mut line, &mut used);
            }
            let part = clip_to_cells(remainder, max_cells, reg);
            if part.is_empty() {
                break;
            }
            line.push_str(&part);
            used = visible_width_cells(&part, reg);
            push_line(&mut lines, &mut line, &mut used);
            remainder = &remainder[part.len()..];
        }
        i += 1;
    }

    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
