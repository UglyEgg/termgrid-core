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
            RenderOp::TextBlock {
                x, y, w: rw, h: rh, ..
            } => push(Rect::new(*x, *y, *rw, *rh)),
            RenderOp::TextBlockStyled {
                x, y, w: rw, h: rh, ..
            } => push(Rect::new(*x, *y, *rw, *rh)),
            RenderOp::Blit {
                x, y, w: bw, h: bh, ..
            } => push(Rect::new(*x, *y, *bw, *bh)),
            RenderOp::Box {
                x, y, w: bw, h: bh, ..
            } => push(Rect::new(*x, *y, *bw, *bh)),
        }
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

            RenderOp::TextBlock {
                x,
                y,
                w,
                h,
                text,
                style,
                wrap,
            } => {
                let spans = [Span::new(text.as_str(), *style)];
                self.put_wrapped_styled(*x, *y, *w, &spans, wrap, Some(*h));
            }

            RenderOp::TextBlockStyled {
                x,
                y,
                w,
                h,
                spans,
                wrap,
            } => {
                self.put_wrapped_styled(*x, *y, *w, spans, wrap, Some(*h));
            }

            RenderOp::Blit { x, y, w, h, cells } => {
                self.blit(*x, *y, *w, *h, cells);
            }

            RenderOp::FillRect {
                x,
                y,
                w,
                h,
                glyph,
                style,
            } => {
                self.fill_rect(*x, *y, *w, *h, glyph, *style);
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
        self.fill_rect(x, y, w, h, " ", Style::plain());
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

    // NOTE: plain text wrapping is implemented by converting text into a single span
    // and delegating to `wrap_spans_wordwise` so WrapOpts behavior is consistent.

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

    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, glyph: &str, style: Style) {
        if w == 0 || h == 0 {
            return;
        }
        if x >= self.grid.width || y >= self.grid.height {
            return;
        }

        let gw = self.reg.width(glyph) as u16;
        let gw = gw.max(1);
        // Avoid placing half of a wide glyph at the right edge.
        if gw == 2 && x + 1 >= self.grid.width {
            return;
        }

        let max_w = self.grid.width.saturating_sub(x);
        let max_h = self.grid.height.saturating_sub(y);
        let w = w.min(max_w);
        let h = h.min(max_h);
        if w == 0 || h == 0 {
            return;
        }

        for dy in 0..h {
            let yy = y.saturating_add(dy);
            let mut cx = x;
            let mut remaining = w;

            while remaining > 0 && cx < self.grid.width {
                if gw == 2 {
                    if remaining < 2 {
                        break;
                    }
                    if cx + 1 >= self.grid.width {
                        break;
                    }
                    self.grid.put_text(cx, yy, glyph, style, &self.reg);
                    cx = cx.saturating_add(2);
                    remaining = remaining.saturating_sub(2);
                } else {
                    self.grid.put_text(cx, yy, glyph, style, &self.reg);
                    cx = cx.saturating_add(1);
                    remaining = remaining.saturating_sub(1);
                }
            }
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
