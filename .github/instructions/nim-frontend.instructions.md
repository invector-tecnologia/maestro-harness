---
applyTo: "frontend/**/*.nim"
description: "Use when writing Maestro's Nim TUI that consumes Niobium: widget composition, constraint layout, the stdio protocol client, and headless snapshot tests."
---

# Nim Frontend Rules (Niobium consumer)

Maestro's TUI is a **consumer** of [Niobium](https://github.com/invector-tecnologia/niobium), never a
re-implementation of it. Frontend Nim code lives under `frontend/`.

## Dependency & toolchain
- `requires "niobium >= 0.1.0"` in `frontend/*.nimble`. Nim ≥ 2.0, `--mm:orc`, formatted with `nph`.
- Use only Niobium's public API: `newTerminal(newAnsiBackend())`, `term.setup()` /
  `defer term.restore()`, `term.draw proc(f: var Frame) = ...`, `f.renderWidget(w, rect)`.

## Composition
- Compose only shipped widgets: `Block`, `Paragraph`, `List`, `Table`, `Tabs`, `Clear`, `Gauge`,
  `Sparkline`, `BarChart`, `Scrollbar`, `Chart`. Do not build custom cell/diff/backend logic.
- Lay out with constraints (`length`, `percentage`, `ratio`, `min`, `max`, `fill`) and
  `f.area.split(...)`; never hard-code absolute coordinates.
- Keep the render function a **pure function of state**: read the latest core snapshot, draw the
  frame, forward input. No business logic in the TUI.

## Boundary discipline
- The TUI talks to the Rust core **only** over the line-delimited JSON stdio protocol
  (`protocol.nim`). It never embeds orchestration logic or assumes core-internal types.
- Terminal state (raw mode, alt screen) must be restored via `defer` even on error.

## Testing
- Assert rendering with Niobium's **test backend** (renders a `Buffer` to text, no TTY) against
  golden snapshots under `frontend/tests/`. Changing a golden file must be intentional.
