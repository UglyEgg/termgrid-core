# Backend Integration

This guide describes the required control flow for integrating a backend.

## Reference Loop

```mermaid
flowchart TD
  A["Receive op(s) or frame"] --> B["Apply sequentially via Renderer"]
  B --> C["Collect Damage"]
  C --> D{"Damage empty?"}
  D -- yes --> E["No redraw"]
  D -- no --> F["Render damaged region(s)"]
  F --> G["Flush / present output"]
```

## Backend Responsibilities

Backends MUST:

- Use the `Grid` as the source of truth.
- Not reinterpret glyph width policy; the renderer has already expanded wide glyphs.
- Treat `Cell::Continuation` as non-rendering.

Backends SHOULD:

- Coalesce adjacent dirty cells into spans.
- Minimize style transitions by tracking current style state.
