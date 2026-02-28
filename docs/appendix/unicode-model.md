# Unicode Model

Defines Unicode handling guarantees within termgrid-core.

---

## Grapheme Boundary Rule

All layout operations occur at grapheme cluster boundaries.

A grapheme cluster:

- May contain multiple codepoints
- May include combining marks
- May include ZWJ sequences

No operation splits a grapheme cluster.

---

## Width Determination

Width is computed before placement.

Possible widths:

- 0 (combining-only cluster)
- 1 (standard width)
- 2 (wide glyph)

Width > 2 is not supported.

---

## Combining Marks

Combining marks are preserved within their cluster.

Wrap logic MUST NOT separate combining marks from base characters.

---

## ZWJ Sequences

ZWJ sequences are treated as atomic grapheme clusters.

They are placed entirely or not at all.

---

## Wrap Boundary Behavior

If a grapheme cluster does not fit within remaining width:

- If wrapping enabled: cluster moves to next line
- If hard clipping: cluster is omitted entirely

Partial placement is forbidden.

---

## Wide Glyph Invariants

If a wide glyph occupies cells (x, x+1):

- Cell x contains grapheme and style
- Cell x+1 is continuation
- Clearing either cell clears both

No orphan continuation cells may exist.

---

## Bidi Disclaimer

Bidirectional text reordering is not performed.

Input text MUST be preprocessed if bidi support is required.

---

## Normalization

Does not normalize Unicode.

Normalization responsibility lies with caller.
