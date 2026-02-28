# Non-goals

`termgrid-core` intentionally does not provide:

- ANSI input stream parsing
- Terminal emulation (cursor state machines, scrollback, alternate screen, etc.)
- Input handling
- Event loops or scheduling
- Terminal capability detection
- Line wrapping at the terminal layer (beyond explicit ops such as `PutWrapped*`)
- High-level layout engines (panels, widgets, constraint systems)

These responsibilities belong to higher-level systems.

This crate focuses strictly on deterministic grid state mutation, representation, and backend-agnostic output derivation.
