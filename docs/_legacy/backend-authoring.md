# Writing a Backend

termgrid-core does not emit escape sequences.

You write a backend that projects grid state to a target surface. That surface may be:

- A terminal (ANSI/VT emission)
- xterm.js via a web transport
- SVG, PNG, PDF
- A remote renderer protocol

The grid is authoritative. The backend is projection.

---

## The Basic Loop

Most backends follow this pattern:

1. Apply an op (or a batch) through `Renderer`
2. Receive `Damage`
3. Redraw either the full surface or the damaged rectangles

```rust
let damage = renderer.apply_with_damage(op, &registry);

if damage.full_redraw {
    redraw_entire_surface(renderer.grid(), &mut out);
} else {
    for rect in &damage.rects {
        redraw_rect(renderer.grid(), *rect, &mut out);
    }
}
```

---

## Iterating Cells Safely

Cells may represent:

- A leading cell of a wide glyph (width 2)
- A continuation cell (trailing cell of a wide glyph)
- A normal cell

Rules for backends:

- Render leading cells.
- Skip continuation cells.
- When clearing, ensure wide glyphs are cleared as a unit.

Rendering continuation cells independently is a correctness bug.

---

## Style Emission

Backends are responsible for emitting style transitions efficiently.

Recommended approach:

- Track current backend style state.
- Emit only deltas when style changes.
- Avoid full resets per cell unless required by target behavior.

termgrid-core stores complete per-cell style; it does not compress escape streams.

---

## Wide Glyph Boundaries

A wide glyph occupies two cells: (x, y) and (x+1, y).

Backends must:

- Render content at the leading cell
- Skip the continuation cell
- Avoid writing half a wide glyph at the right edge

---

## Damage Semantics

Damage is conservative:

- Anything inside a damaged rectangle may have changed.
- Nothing outside has changed.

Damage is not required to be minimal.
If `full_redraw` is set, repaint everything.

Damage rectangles are axis-aligned and bounded to grid coordinates.

---

## xterm.js Notes

Stream-oriented backends often rely on terminal emulators to do layout.
That breaks down under mixed-width glyphs and frequent incremental redraw.

With termgrid-core:

- Layout correctness is resolved before emission.
- Damage limits redraw to the smallest necessary region.
- Backends can target xterm.js while preserving stable grid behavior.

If you see drift in xterm.js, it is a backend emission issue, not state ambiguity.
