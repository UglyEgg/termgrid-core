# Writing a Backend

Backends consume:

- An immutable `Grid`
- Optional `Damage` regions (`Damage`, `Rect`)

Backends MUST:

1. Treat `Cell::Continuation` as non-rendering.
2. Render wide glyphs only from the lead cell.
3. Respect the style model in `Style` (overlay semantics are applied by the renderer).

Backends SHOULD:

- Redraw only damaged cells when `Damage` is available.
- Coalesce adjacent cells into spans to reduce output churn.
- Track current style state and emit deltas rather than full resets.

## Rendering Rule Summary

- `Cell::Empty` MAY be rendered as a space.
- Cells that contain glyphs are rendered as visible content.
- Continuation cells do not render glyphs.
