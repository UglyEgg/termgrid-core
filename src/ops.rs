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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BoxCharset {
    /// ASCII `+`, `-`, `|`.
    Ascii,
    /// Unicode single-line box drawing characters.
    #[default]
    UnicodeSingle,
    /// Unicode double-line box drawing characters.
    UnicodeDouble,
}

fn is_default_truncate_mode(m: &TruncateMode) -> bool {
    *m == TruncateMode::default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TruncateMode {
    /// Drop any glyphs that do not fit.
    #[default]
    Clip,
    /// Replace the tail with an ellipsis ("…") when truncation occurs.
    Ellipsis,
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

    /// Put a multi-line block of text, wrapped and clipped within a rectangle.
    ///
    /// This is a higher-level convenience op.
    TextBlock {
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        text: String,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
        #[serde(default, skip_serializing_if = "is_default_wrap_opts")]
        wrap: WrapOpts,
    },

    /// Put a styled multi-line block (spans), wrapped and clipped within a rectangle.
    TextBlockStyled {
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        spans: Vec<Span>,
        #[serde(default, skip_serializing_if = "is_default_wrap_opts")]
        wrap: WrapOpts,
    },

    /// Fill a rectangle with a single glyph.
    FillRect {
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        glyph: String,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
    },

    /// Draw a box (border) around a rectangle.
    Box {
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        #[serde(default, skip_serializing_if = "is_default_charset")]
        charset: BoxCharset,
        #[serde(default, skip_serializing_if = "is_plain_style")]
        style: Style,
    },

    /// Blit a rectangular payload of optional cells.
    ///
    /// `cells` is row-major with length `w*h`.
    Blit {
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        cells: Vec<Option<BlitCell>>,
    },
}
