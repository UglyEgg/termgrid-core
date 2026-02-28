# Getting Started

This guide shows the minimal workflow:

1. Create a `Grid`.
2. Create a `GlyphRegistry` (or `RenderProfile`) to define width policy.
3. Apply operations using `Renderer`.
4. Render the resulting grid with a backend (optionally using `Damage`).

## Minimal Example

```rust
use termgrid_core::{Frame, GlyphRegistry, Grid, RenderOp, Renderer, Style};

let reg = GlyphRegistry::default();
let mut grid = Grid::new(80, 24);
let mut r = Renderer::new();

let frame = Frame {
    ops: vec![
        RenderOp::Clear,
        RenderOp::Put { x: 0, y: 0, text: "hello".into(), style: Style::default() },
    ],
};

let damage = r.apply_with_damage(&mut grid, &reg, &frame).expect("render");
```

Notes:

- Use `apply_with_damage` when integrating an incremental backend.
- Enable `debug-validate` during development/CI to catch invariant violations early.
