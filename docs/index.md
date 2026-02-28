# termgrid-core

`termgrid-core` is a deterministic grid state engine for terminal-like rendering.

It provides:

- A canonical `Grid` state model.
- Structured draw operations (`RenderOp`, `Frame`) applied via `Renderer`.
- Unicode-aware glyph width policy via `GlyphRegistry` / `RenderProfile`.
- Explicit wide-glyph representation (`Cell::Continuation`).
- Damage tracking (`Damage`, `Rect`) for incremental redraw.
- Backend decoupling: state mutation is separate from output emission.

This crate does **not** implement a terminal emulator. It does not parse ANSI input streams, manage input state, scrollback, or event loops.

## Documents

- Architecture: `architecture.md`
- Invariants: `invariants.md`
- Non-goals: `non-goals.md`
- Rationale: `rationale.md`
- Performance: `performance.md`

Contracts:

- Rendering model: `contracts/rendering-model.md`
- Style model: `contracts/style-model.md`
- Wire format v1: `contracts/wire-format-v1.md`

Guides:

- Getting started: `guides/getting-started.md`
- Writing a backend: `guides/writing-a-backend.md`
- Backend integration: `guides/backend-integration.md`

Reference:

- Public API summary: `reference/public-api.md`
- Feature flags: `reference/feature-flags.md`
