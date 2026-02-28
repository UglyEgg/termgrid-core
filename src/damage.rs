//! Damage tracking for incremental terminal rendering.
//!
//! Coordinates are expressed in terminal **cells**.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub fn right(&self) -> u16 {
        self.x.saturating_add(self.w)
    }

    pub fn bottom(&self) -> u16 {
        self.y.saturating_add(self.h)
    }

    pub fn intersects_or_touches(&self, other: &Rect) -> bool {
        // Treat adjacency as mergeable to reduce rect count.
        let ax1 = self.x;
        let ay1 = self.y;
        let ax2 = self.right();
        let ay2 = self.bottom();

        let bx1 = other.x;
        let by1 = other.y;
        let bx2 = other.right();
        let by2 = other.bottom();

        !(ax2 < bx1 || bx2 < ax1 || ay2 < by1 || by2 < ay1)
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = self.right().max(other.right());
        let y2 = self.bottom().max(other.bottom());
        Rect::new(x1, y1, x2.saturating_sub(x1), y2.saturating_sub(y1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Damage {
    pub full_redraw: bool,
    pub rects: Vec<Rect>,
}

impl Damage {
    pub fn empty() -> Self {
        Self {
            full_redraw: false,
            rects: Vec::new(),
        }
    }

    pub fn full() -> Self {
        Self {
            full_redraw: true,
            rects: Vec::new(),
        }
    }

    pub fn push_rect(&mut self, rect: Rect, max_rects: usize) {
        if self.full_redraw {
            return;
        }
        if rect.is_empty() {
            return;
        }

        self.rects.push(rect);
        self.merge_in_place();

        if self.rects.len() > max_rects {
            self.full_redraw = true;
            self.rects.clear();
        }
    }

    fn merge_in_place(&mut self) {
        // Small N; O(n^2) is fine and keeps behavior deterministic.
        let mut out: Vec<Rect> = Vec::with_capacity(self.rects.len());
        for r in self.rects.drain(..) {
            let mut merged = r;
            let mut i = 0;
            while i < out.len() {
                if merged.intersects_or_touches(&out[i]) {
                    merged = merged.union(&out[i]);
                    out.remove(i);
                    i = 0;
                } else {
                    i += 1;
                }
            }
            out.push(merged);
        }
        self.rects = out;
    }
}
