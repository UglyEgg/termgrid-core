# Security Policy

termgrid-core is a deterministic state engine for grid-based rendering.
It does not execute untrusted code, interpret escape sequences, or perform
network I/O. However, misuse or adversarial inputs may still create risk
conditions. This document outlines those boundaries clearly.

---

## Threat Model

termgrid-core:

- Does not parse ANSI/VT escape streams
- Does not evaluate scripts
- Does not perform font shaping
- Does not normalize Unicode input
- Does not perform bidirectional reordering

It operates purely on structured `RenderOp` inputs.

Security exposure therefore exists primarily in:

- Extremely large inputs
- Pathological Unicode sequences
- Adversarial replay logs
- Backend misuse

---

## Unicode Abuse Considerations

The engine treats grapheme clusters atomically. However:

- Combining mark chains are not artificially length-limited.
- Extremely long grapheme clusters may increase processing cost.
- Width behavior is version-pinned and deterministic.

Applications embedding termgrid-core should:

- Impose input size limits
- Consider upstream normalization policies
- Validate untrusted replay logs before execution

---

## Resource Exhaustion

Worst-case per-frame complexity is O(total_cells).
Damage escalation prevents unbounded rectangle growth.

However, callers remain responsible for:

- Limiting grid dimensions
- Limiting replay size
- Controlling animation frame rate

termgrid-core does not internally rate-limit or sandbox execution.

---

## Serialization Integrity

The wire format is canonical and deterministic.

If byte-level integrity is required:

- Do not reserialize using non-canonical encoders
- Validate version field before replay
- Consider hashing serialized frames for audit trails

---

## Backend Responsibility

Backends are responsible for:

- Correct terminal reset semantics
- Avoiding escape injection vulnerabilities
- Sanitizing output channels if required

termgrid-core guarantees structural correctness of the grid state.
It does not guarantee backend safety.

---

## Reporting Vulnerabilities

If a deterministic invariant can be violated or state corruption
can be induced through valid API usage, it is considered a defect.

Please report issues with:

- Minimal reproducible example
- Grid dimensions
- Unicode content used
- Expected vs actual behavior

Security-sensitive reports may be submitted privately before disclosure.

---

## Explicit Non-Goals

termgrid-core is not:

- A sandbox
- A VT emulator
- A security boundary
- A privilege isolation layer

It is a deterministic rendering engine.

Consumers must provide appropriate containment where required.