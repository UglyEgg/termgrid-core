# Wire Format v1

This document specifies the JSON wire format for serialized render operations.

The wire format is designed for:

- Serialization of `Frame` and its `RenderOp` list.
- Deterministic *semantic* replay (operation ordering), not byte-identical canonical JSON.

## Frame

A frame contains an ordered list of operations.

Example:

```json
{
  "ops": [
    { "op": "Clear" },
    { "op": "Put", "x": 0, "y": 0, "text": "hello" }
  ]
}
```

## Versioning

The v1 wire format is defined by the schema implied by serde serialization of `Frame` / `RenderOp` in this crate.

Consumers MUST bind explicitly to v1.

Future versions MAY change:

- Operation shapes
- Defaulting rules
- Validation strictness

## Unknown Fields

- Unknown fields within known operations SHOULD be ignored for forward compatibility.
- Unknown operation tags (`op`) MUST be rejected.

## Determinism

Determinism is defined as:

- The ordered sequence of operations (`ops`) is preserved.
- Applying the same `ops` to the same initial `Grid` under the same registry/profile yields the same resulting grid state.

Byte-level canonicalization across independent JSON emitters is NOT guaranteed.
