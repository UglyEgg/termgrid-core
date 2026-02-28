//! termgrid-core
//!
//! A deterministic, terminal-like grid rendering core built around two ideas:
//!
//! 1. **Producers emit structured draw ops**, not raw ANSI.
//! 2. **Glyph display width is a policy**, captured in a render profile (registry), not a guess.
//!
//! This crate purposely stays small and unopinionated about IPC and higher-level
//! engine semantics.

pub mod ansi;
pub mod damage;
pub mod grid;
pub mod ops;
pub mod registry;
pub mod render;
pub mod search;
pub mod style;
pub mod text;

pub use damage::{Damage, Rect};
pub use grid::{Cell, Grid};
pub use ops::{BlitCell, BoxCharset, Frame, RenderOp, TruncateMode};
pub use registry::{GlyphInfo, GlyphRegistry, RenderProfile};
pub use render::{RenderError, Renderer};
pub use search::{
    fuzzy_match_positions_graphemes, fuzzy_match_positions_graphemes_latest,
    fuzzy_match_positions_graphemes_v1, match_positions_graphemes,
};
pub use style::Style;
pub use text::{
    apply_highlight, clip_to_cells_spans, clip_to_cells_text, ellipsis_to_cells_spans,
    ellipsis_to_cells_text, measure_cells_spans, measure_cells_text, normalize_spans,
    spans_plain_text, wrap_spans_wordwise, Span, Spans, WrapOpts,
};
