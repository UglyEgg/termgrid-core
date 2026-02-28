# Design Rationale

## Why Grid State Instead of Stream Output?

Stream-based rendering (raw ANSI writes) couples application logic to presentation side effects and makes replay, testing, and validation difficult.

A grid state model:

- Is inspectable and testable.
- Enables deterministic replay.
- Supports multiple backends (ANSI, xterm.js, offscreen buffers).
- Separates mutation semantics from output encoding.

## Why Explicit Continuation Cells?

Wide glyphs create ambiguity in stream models. Explicit continuation cells:

- Prevent partial overwrites of wide glyphs.
- Make invariants verifiable.
- Keep backend logic simple (render lead cell only).

## Why Registry/Profile Width Policy?

Unicode width handling varies across environments. A registry/profile:

- Centralizes width policy.
- Makes width resolution deterministic and explicit.
- Allows controlled overrides without relying on terminal behavior.

## Why Damage Tracking?

Incremental redraw reduces backend work. Damage tracking:

- Enables redrawing only affected regions.
- Improves performance for high-frequency updates.
- Avoids backend-side heuristics.

## Why Invariant Enforcement?

Rendering bugs can be subtle and compounding. Formal invariants:

- Detect corruption early.
- Prevent silent layout drift.
- Support safe refactors and vendor review.
