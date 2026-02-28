# Glossary

## Grid
A two-dimensional array of cells representing the entire render surface.

## Cell
A single position in the grid. Contains:
- A grapheme cluster
- Style information
- Continuation metadata

## Grapheme Cluster
A user-perceived character, potentially composed of multiple Unicode codepoints (e.g., emoji sequences, combining marks).

## Wide Glyph
A grapheme cluster occupying two grid cells.

## Continuation Cell
The trailing cell of a wide glyph. Must never be rendered independently.

## RenderOp
A structured mutation applied to the grid.

## Renderer
The component that applies RenderOps to a grid and returns damage information.

## Damage
A conservative set of rectangles indicating which grid regions may have changed.

## Full Redraw
A damage state indicating the entire grid should be repainted.

## Determinism
The guarantee that identical inputs produce identical grid state.

## Wire Format v1
A stable serialized representation of rendering operations or grid state.

## Backend
A projection layer that converts grid state into a target format (terminal, SVG, PNG, etc.).

## State Transition
The application of one or more RenderOps resulting in a new grid state.

---

## Canonical
Refers to a single deterministic representation of logically equivalent state.

## Serialization
The process of converting structured state into Wire Format.

## Mutation
Application of a RenderOp resulting in state transition.

## Projection
Conversion of grid state into a backend-specific representation.
