# Replay and Snapshots

Because state transitions are deterministic, grid state can be reconstructed precisely.

## Snapshot Strategy

Store:
- Initial grid state
- Ordered RenderOps

Reapply to reconstruct grid state.

## Diff Strategy

Alternatively, store:
- Initial snapshot
- Damage-based deltas

This allows efficient replay and random access.

## Use Cases

- UI regression testing
- Time-travel debugging
- Terminal recording systems
- State synchronization across processes

Replay does not depend on terminal emulator behavior.
