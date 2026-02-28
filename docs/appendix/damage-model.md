# Damage Model

Damage describes which grid regions may have changed after applying RenderOps.

Damage is conservative:
- Regions reported may have changed.
- Regions not reported did not change.

---

## Full Redraw

When damage fragments exceed thresholds, damage escalates to full redraw.

Backends must repaint the entire grid when `full_redraw` is true.

---

## Geometry Guarantees

Damage rectangles are:
- Axis-aligned
- Bounded to valid grid coordinates

Overlap is permitted. Rectangles are not required to be minimal.
