# Style Model

This document defines how `Style` is applied to cells.

## Style Fields

A cell style may include:

- Foreground color
- Background color
- Flags (bold, italic, underline, etc.)

Exact fields are defined by `Style`.

## Overlay Rules

When an operation applies a `Style` overlay:

- If a foreground value is present, it MUST override the destination foreground.
- If a background value is present, it MUST override the destination background.
- Flags MUST be additive (logical OR).
- The absence of a field MUST NOT be interpreted as reset.

Reset behavior MUST be explicit.

## Empty Cells

`Cell::Empty` has no inherent style. Backends MUST define a consistent rendering for empty cells.

Backends MUST treat styled spaces as styled glyphs; consequently, clear operations that aim to erase visible content SHOULD use explicit plain spaces rather than `Cell::Empty` when the goal is to remove style bleed.
