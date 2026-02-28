# AI Integration Guide

This guide describes optimal integration patterns for LLM-generated TUIs.

## Core Pattern

```rust
let mut renderer = Renderer::new(width, height);
let registry = GlyphRegistry::unicode11();

let damage = renderer.apply_with_damage(frame);

for rect in damage.rects {
    backend.repaint(rect);
}

if damage.full_redraw {
    backend.repaint_full();
}
```

---

## Best Practices for AI-Generated TUIs

1. Never manually mutate Grid.
2. Always use Renderer.
3. Always consume Damage.
4. Validate invariants in debug builds.
5. Preprocess bidi text externally if required.

---

## Markdown Editor Scenario

An AI can:

- Convert parsed markdown to styled spans.
- Use put_wrapped_styled for paragraphs.
- Use damage rects to repaint only modified regions.
