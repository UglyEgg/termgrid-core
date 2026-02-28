# Feature Flags

## `debug-validate`

Enables invariant validation after each operation application.

When enabled:

- The renderer validates `Grid` invariants after each `RenderOp`.
- Violations are treated as logic errors and result in panic.

Intended usage:

- Development builds
- CI (test and fuzz/proptest runs)

`debug-validate` may materially reduce throughput and is not recommended for latency/throughput sensitive production use unless correctness tripwires are explicitly desired.
