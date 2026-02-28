# Comparison: Stream-Oriented Rendering

Most terminal UI stacks render by emitting a byte stream.

The terminal emulator interprets that stream and mutates its internal state.
This works until you need correctness guarantees under mixed-width Unicode,
partial redraw, or deterministic replay.

termgrid-core follows a different model: structured state transitions on an explicit grid.

---

## Stream Model

Properties:

- Output stream is authoritative
- Emulator resolves layout and wrap behavior
- State reconstruction requires replay of the stream
- Behavior varies between emulators and browser terminals

Common failure modes:

- Wide glyph drift
- Wrap inconsistencies
- Backend-specific quirks
- Non-deterministic output under partial redraw

---

## Grid Model

Properties:

- Grid state is authoritative
- Layout is resolved before emission
- State transitions are deterministic
- Damage is explicit

Consequences:

- Identical input yields identical grid state
- Backends can target terminals, xterm.js, or file formats with consistent output
- Snapshot testing is straightforward
- Replay is independent of emulator behavior

---

## Architectural Maturity Signal

Stream rendering treats UI as side-effect.
Grid rendering treats UI as a state machine.

That shift enables:

- Canonical serialization
- Portable projections
- Bounded mutation cost
- Explicit correctness invariants
