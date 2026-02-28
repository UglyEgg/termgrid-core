# Performance Contract

This document describes performance guarantees provided by termgrid-core.

---

## Bounded Mutation

Applying a RenderOp runs in time proportional to affected cells.

---

## Damage Escalation

Damage rectangles are capped. Excess fragmentation escalates to full redraw to avoid pathological overhead.

---

## Worst-Case Bound

Worst-case per frame is bounded by O(total_cells).
