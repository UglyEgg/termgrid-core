![termgrid-core](docs/assets/termgrid-core-banner.webp)

# termgrid-core

[![Crates.io](https://img.shields.io/crates/v/termgrid-core.svg)](https://crates.io/crates/termgrid-core)
[![Docs](https://img.shields.io/badge/docs-site-blue)](https://termgrid.entropy.quest)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue)](#msrv)
[![License](https://img.shields.io/badge/license-AGPL%20%2B%20commercial-informational)](#license)

[![ci](https://github.com/UglyEgg/termgrid-core/actions/workflows/ci.yml/badge.svg)](https://github.com/UglyEgg/termgrid-core/actions/workflows/ci.yml)
[![docs](https://github.com/UglyEgg/termgrid-core/actions/workflows/docs.yml/badge.svg)](https://github.com/UglyEgg/termgrid-core/actions/workflows/docs.yml)

`termgrid-core` is a deterministic terminal grid state engine.

It provides a canonical grid model with invariant enforcement, explicit Unicode width policy via a glyph registry, and damage tracking for incremental redraw. The output surface is backend-decoupled: you apply render operations to a grid, then render the grid through whatever backend you choose (terminal, xterm.js, headless tests, etc.).

## Why this exists

Modern “terminal-ish” rendering is frequently stream-oriented (ANSI output) and environment-dependent (Unicode width behavior varies across terminals). For systems that need correctness, replayability, and consistent layout, treating rendering as deterministic state mutation is materially simpler to test and reason about.

This crate was built after encountering the limitations of stream-based terminal rendering in modern environments. Deterministic state proved more reliable than implicit ANSI behavior.

## Quick start

```rust
use termgrid_core::{Grid, Renderer};

let mut grid = Grid::new(80, 25);
let mut r = Renderer::new();

r.draw_text(&mut grid, 0, 0, "Hello, termgrid.");
let damage = r.take_damage();

// Backend renders (&grid, damage)
```

## Documentation

- Documentation site: <https://termgrid.entropy.quest>
- Rust API (docs.rs): <https://docs.rs/termgrid-core>
- Wire format v1 (contract): [Wire Format v1](docs/contracts/wire-format-v1.md)

Local docs build (includes rustdoc under `site/api/`):

```bash
./scripts/docs_build.sh
python -m http.server -d site 8000
```

## MSRV

Minimum supported Rust version: **1.85**.

CI enforces MSRV via a dedicated test lane.

## What this crate does not do

- Parse arbitrary ANSI input streams
- Implement a terminal emulator
- Provide IPC/protocol layers (belongs above this crate)

## License

Licensed under **AGPL-3.0-or-later** by default.

A paid commercial license (**LicenseRef-VaranidWorks-Commercial-1.0**) is available to allow use without AGPL obligations.

See `LICENSE` and `LICENSES/`.
