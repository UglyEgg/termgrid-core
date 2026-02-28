use crate::{
    style::is_plain_style,
    text::{is_default_wrap_opts, Span, WrapOpts},
    Style,
};
use serde::{Deserialize, Serialize};

fn is_default_charset(c: &BoxCharset) -> bool {
    *c == BoxCharset::default()
}

/// A set of draw ops emitted by a producer for a single render tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Frame {
    pub ops: Vec<RenderOp>,
}

/// A single cell in a `blit` payload.
///
/// When present, this cell overwrites the destination.
/// When absent (`null` in JSON), the destination cell is left unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlitCell {
    pub glyph: String,
    #[serde(default, skip_serializing_if = "is_plain_style")]
    pub style: Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoxCharset {
    /// ASCII `+`, `-`, `|`.
    Ascii,
    /// Unicode single-line box drawing characters.
    UnicodeSingle,
    /// Unicode double-line box drawing characters.
    UnicodeDouble,
}

impl Default for BoxCharset {
    fn default() -> Self {
        Self::UnicodeSingle
    }
}

fn is_default_truncate_mode(m: &TruncateMode) -> bool {
    *m == TruncateMode::default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruncateMode {
    /// Drop any glyphs that do not fit.
    Clip,
    /// Replace the tail with an ellipsis ("…") when truncation occurs.
    Ellipsis,
}

impl Default for TruncateMode {
    fn default() -> Self {
        Self::Clip
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum RenderOp {
    /// Clear the entire grid to empty cells.
    Clear,

    /// Clear a full line (row) to plain spaces.
    ///
    /// This is rendered as plain spaces rather than `Cell::Empty` to avoid
    /// style bleed from earlier styled cells on the same line.
    ClearLine { y: u16 },

    /// Clear from (`x`,`y`) to end-of-line (inclusive) to plain spaces.
    ///
    /// This mirrors ANSI EL (erase in line) mode 0.
    ClearEol { x: u16, y: u16 },

    /// Clear from start-of-line to (`x`,`y`) (inclusive) to plain spaces.
    ///
    /// This mirrors ANSI EL (erase in line) mode 1.
    ClearBol { x: u16, y: u16 },

    /// Clear from (`x`,`y`) to end-of-screen (inclusive) to plain spaces.
    ///
    /// This mirrors ANSI ED (erase in display) mode 0.
    ClearEos { x: u16, y: u16 },

    /// Clear a rectangle to plain spaces.
    ///
    /// This is semantically equivalent to `FillRect` with a plain style.
    ClearRect { x: u16, y: u16, w: u16, h: u16 },

    /// Put text at a coordinate.
    Put {
        x: u16,
        y: u16,
        text: String,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
    },

    /// Put a single glyph (grapheme cluster) at a coordinate.
    PutGlyph {
        x: u16,
        y: u16,
        glyph: String,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
    },

    /// Put a single-line label, clipped to `w` cells.
    ///
    /// This is a convenience op for common UI labels.
    Label {
        x: u16,
        y: u16,
        /// Maximum width in cells.
        w: u16,
        text: String,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
        #[serde(default, skip_serializing_if = "is_default_truncate_mode")]
        truncate: TruncateMode,
    },

    /// Put a single-line styled label (spans), clipped to `w` cells.
    ///
    /// This op is analogous to `Label` but supports inline styling.
    LabelStyled {
        x: u16,
        y: u16,
        /// Maximum width in cells.
        w: u16,
        spans: Vec<Span>,
        #[serde(default, skip_serializing_if = "is_default_truncate_mode")]
        truncate: TruncateMode,
    },

    /// Put a single-line styled label (spans), clipped to `w` cells.
    ///
    /// This op is useful for UI where inline styling is needed (for example,
    /// highlighted search matches or mixed emphasis).
    PutStyled {
        x: u16,
        y: u16,
        /// Maximum width in cells.
        w: u16,
        spans: Vec<Span>,
        #[serde(default, skip_serializing_if = "is_default_truncate_mode")]
        truncate: TruncateMode,
    },

    /// Put wrapped text within `w` cells, flowing downward from (`x`,`y`).
    ///
    /// Wrapping is whitespace-aware with hard-break fallback for long words.
    PutWrapped {
        x: u16,
        y: u16,
        /// Wrap width in cells.
        w: u16,
        text: String,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
    },

    /// Put wrapped styled text (spans) within `w` cells, flowing downward from (`x`,`y`).
    ///
    /// Wrapping is whitespace-aware with hard-break fallback for long tokens.
    /// Use `wrap_opts` to control whitespace preservation and trimming.
    PutWrappedStyled {
        x: u16,
        y: u16,
        /// Wrap width in cells.
        w: u16,
        spans: Vec<Span>,
        #[serde(default, skip_serializing_if = "is_default_wrap_opts")]
        wrap_opts: WrapOpts,
        /// Optional maximum number of visual lines to render.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_lines: Option<u16>,
    },

    /// Blit (copy) a small source cell-map onto the grid.
    ///
    /// - `cells` is a row-major array of length `w*h`.
    /// - `null` cells are transparent (leave destination unchanged).
    /// - Wide glyphs (width=2) occupy two destination cells; the next source cell
    ///   in that row is ignored.
    Blit {
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        cells: Vec<Option<BlitCell>>,
    },

    /// Fill a rectangle with styled spaces.
    ///
    /// Use this for clearing regions and for background fills.
    FillRect {
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
    },

    /// Draw a horizontal line using a single glyph.
    ///
    /// `len` is measured in **cells**. For width=2 glyphs, placement will stop
    /// when fewer than 2 cells remain.
    #[serde(rename = "hline")]
    HLine {
        x: u16,
        y: u16,
        len: u16,
        glyph: String,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
    },

    /// Draw a vertical line using a single glyph.
    ///
    /// `len` is measured in **rows**.
    #[serde(rename = "vline")]
    VLine {
        x: u16,
        y: u16,
        len: u16,
        glyph: String,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
    },

    /// Draw a bordered box.
    ///
    /// The box is clipped to the grid bounds.
    Box {
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
        #[serde(default, skip_serializing_if = "is_default_charset")]
        charset: BoxCharset,
    },
}
