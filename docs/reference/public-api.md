# Public API Summary

This document summarizes the primary public API surface re-exported from `termgrid-core`.

The canonical export list is defined in `src/lib.rs`.

## Core State

### `Grid`

A 2D grid of `Cell` values. The `Grid` is the canonical render state.

Typical usage:

- Allocate once with `Grid::new(width, height)`
- Mutate only through `Renderer` operations
- Read for backend output

### `Cell`

Represents one grid cell. Cells include:

- empty cells
- visible glyph cells
- continuation cells for wide glyphs

Backends MUST treat continuation cells as non-rendering.

## Rendering

### `Renderer`

Applies `RenderOp` / `Frame` to a `Grid` using a `GlyphRegistry` (or profile-derived registry).

Key entrypoints:

- `Renderer::apply(&mut Grid, &GlyphRegistry, &Frame) -> Result<(), RenderError>`
- `Renderer::apply_op(&mut Grid, &GlyphRegistry, &RenderOp) -> Result<(), RenderError>`
- `Renderer::apply_with_damage(&mut Grid, &GlyphRegistry, &Frame) -> Result<Damage, RenderError>`
- `Renderer::to_ansi(&Grid) -> String` (debug-oriented ANSI emission)

### `RenderError`

Error type for renderer application.

Errors represent invalid inputs or failure to apply an operation (distinct from invariant violations, which are logic errors under `debug-validate`).

## Operations

### `Frame`

A batch of operations for a single tick.

`Frame { ops: Vec<RenderOp> }`

### `RenderOp`

A single operation applied by the renderer. Current operations include (non-exhaustive summary; see rustdoc for full detail):

- Clear and erase ops: `Clear`, `ClearLine`, `ClearEol`, `ClearBol`, `ClearEos`, `ClearRect`
- Text ops: `Put`, `PutGlyph`, `PutStyled`, `PutWrapped`, `PutWrappedStyled`
- Label ops: `Label`, `LabelStyled`
- Region ops: `FillRect`
- Line/box ops: `HLine`, `VLine`, `Box`
- Blit ops: `Blit` (structured cell payload)

### `BlitCell`

Represents a single cell in a `Blit` payload. When present, it overwrites the destination; when absent (`null` in JSON), the destination cell is left unchanged.

### `BoxCharset`

Specifies the glyph set used for line/box drawing operations.

### `TruncateMode`

Truncation policy used by specific operations that can elide content to fit bounds.

## Width Policy

### `GlyphRegistry`

Resolves display width (cells) for grapheme clusters under a configured policy.

### `RenderProfile`

Profile describing width policy and rendering behavior.

### `GlyphInfo`

Metadata returned/used by the registry for width and classification decisions.

## Styling

### `Style`

Style attributes applied to cells (colors + flags). Style overlay semantics are defined in `contracts/style-model.md`.

## Text Utilities

These utilities operate on text and span models used by renderer operations.

### Spans

- `Span`
- `Spans`

### Measurement and clipping

- `measure_cells_text`, `measure_cells_spans`
- `clip_to_cells_text`, `clip_to_cells_spans`
- `ellipsis_to_cells_text`, `ellipsis_to_cells_spans`

### Normalization and wrapping

- `normalize_spans`
- `wrap_spans_wordwise`
- `WrapOpts`

### Highlighting

- `apply_highlight`

## Search Utilities

Functions for matching/grapheme-aware search:

- `match_positions_graphemes`
- `fuzzy_match_positions_graphemes_v1`
- `fuzzy_match_positions_graphemes_latest`
- `fuzzy_match_positions_graphemes`
