# Non-Goals

termgrid-core is intentionally narrow in scope.

It provides deterministic mutation of a grid-based rendering state plus tooling
to project that state to any backend. Everything else sits above or beside it.

---

## Not a Terminal Emulator

Does not:

- Parse ANSI/VT escape streams
- Implement cursor modes
- Maintain scrollback
- Emulate terminal quirks

If you need a terminal emulator, use one.

---

## Not a Layout Engine

Does not provide:

- Widget hierarchies
- Layout constraints
- Focus systems
- Input routing

termgrid-core tracks render state. Layout and interaction are caller concerns.

---

## Not a Text Shaper

Does not:

- Perform bidi reordering
- Shape complex scripts
- Apply OpenType features
- Resolve font metrics

Text shaping and bidi must be handled upstream if required.

---

## Not an Async Runtime

No scheduler, reactor, task model, or IO abstraction.
Rendering is synchronous and deterministic.

---

## Not a Policy Layer

Does not decide:

- When to redraw
- How to batch operations
- How to debounce updates
- How to persist or sync state

You own policy. termgrid-core provides primitives and invariants.
