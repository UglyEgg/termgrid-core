use crate::{GlyphRegistry, Style};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Glyph {
        grapheme: String,
        style: Style,
    },
    /// Placeholder cell occupied by the trailing half of a width=2 glyph.
    Continuation,
}

impl Cell {
    pub fn is_continuation(&self) -> bool {
        matches!(self, Cell::Continuation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub width: u16,
    pub height: u16,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        let len = width as usize * height as usize;
        Self {
            width,
            height,
            cells: vec![Cell::Empty; len],
        }
    }

    pub fn clear(&mut self) {
        for c in &mut self.cells {
            *c = Cell::Empty;
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        let idx = self.idx(x, y)?;
        self.cells.get(idx)
    }

    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(idx) = self.idx(x, y) {
            self.cells[idx] = cell;
        }
    }

    pub fn idx(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(y as usize * self.width as usize + x as usize)
    }

    fn clear_overlaps_at(&mut self, x: u16, y: u16, reg: &GlyphRegistry) {
        let here = self.get(x, y).cloned();

        // If we're overwriting the trailing half of a wide glyph, clear the leading half too.
        if matches!(here, Some(Cell::Continuation)) {
            self.set(x, y, Cell::Empty);

            if x > 0 {
                if let Some(Cell::Glyph { grapheme, .. }) = self.get(x - 1, y).cloned() {
                    if reg.width(&grapheme) == 2 {
                        self.set(x - 1, y, Cell::Empty);
                    }
                }
            }
            return;
        }

        // If we're overwriting the leading half of a wide glyph, clear its continuation.
        if let Some(Cell::Glyph { grapheme, .. }) = here {
            if reg.width(&grapheme) == 2
                && x + 1 < self.width
                && matches!(self.get(x + 1, y), Some(Cell::Continuation))
            {
                self.set(x + 1, y, Cell::Empty);
            }
        }

        // If the cell to our left is a wide glyph but we are not its continuation, clear it.
        if x > 0 {
            if let Some(Cell::Glyph { grapheme, .. }) = self.get(x - 1, y).cloned() {
                if reg.width(&grapheme) == 2 && !matches!(self.get(x, y), Some(Cell::Continuation))
                {
                    self.set(x - 1, y, Cell::Empty);
                }
            }
        }
    }

    /// Places `text` starting at (`x`, `y`) using the registry for width policy.
    ///
    /// This function is newline-agnostic: `\n` will stop placement.
    /// Returns the next x position after the last placed glyph.
    pub fn put_text(
        &mut self,
        mut x: u16,
        y: u16,
        text: &str,
        style: Style,
        reg: &GlyphRegistry,
    ) -> u16 {
        if y >= self.height {
            return x;
        }

        for g in UnicodeSegmentation::graphemes(text, true) {
            if g == "\n" || g == "\r" {
                break;
            }

            let w = reg.width(g);
            if x >= self.width {
                break;
            }
            // Avoid placing half of a wide glyph at the right edge.
            if w == 2 && x + 1 >= self.width {
                break;
            }

            // Maintain wide-glyph invariants when overwriting.
            self.clear_overlaps_at(x, y, reg);
            if w == 2 {
                self.clear_overlaps_at(x + 1, y, reg);
            }

            self.set(
                x,
                y,
                Cell::Glyph {
                    grapheme: g.to_string(),
                    style,
                },
            );

            if w == 2 {
                self.set(x + 1, y, Cell::Continuation);
                x = x.saturating_add(2);
            } else {
                x = x.saturating_add(1);
            }
        }

        x
    }

    pub fn rows(&self) -> impl Iterator<Item = &[Cell]> {
        self.cells.chunks(self.width as usize)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantError {
    ContinuationAtColumn0 { y: u16 },
    OrphanContinuation { x: u16, y: u16 },
    MissingContinuationHalf { x: u16, y: u16 },
}

impl Grid {
    /// Validate internal grid invariants.
    ///
    /// This is intended for tests and debug builds. Production callers can
    /// enable it via the `debug-validate` feature.
    pub fn validate_invariants(&self, reg: &GlyphRegistry) -> Result<(), InvariantError> {
        for y in 0..self.height {
            for x in 0..self.width {
                let c = self.get(x, y).expect("in-bounds");
                match c {
                    Cell::Continuation => {
                        if x == 0 {
                            return Err(InvariantError::ContinuationAtColumn0 { y });
                        }
                        let prev = self.get(x - 1, y).expect("in-bounds");
                        match prev {
                            Cell::Glyph { grapheme, .. } => {
                                if reg.width(grapheme) != 2 {
                                    return Err(InvariantError::OrphanContinuation { x, y });
                                }
                            }
                            _ => return Err(InvariantError::OrphanContinuation { x, y }),
                        }
                    }
                    Cell::Glyph { grapheme, .. } => {
                        if reg.width(grapheme) == 2 {
                            if x + 1 >= self.width {
                                // A width=2 glyph must never be placed half-visible.
                                return Err(InvariantError::MissingContinuationHalf { x, y });
                            }
                            let next = self.get(x + 1, y).expect("in-bounds");
                            if !matches!(next, Cell::Continuation) {
                                return Err(InvariantError::MissingContinuationHalf { x, y });
                            }
                        }
                    }
                    Cell::Empty => {}
                }
            }
        }
        Ok(())
    }
}
