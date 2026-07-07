---
name: niobium-upstream
description: >
  Authoritative reference to the Niobium immediate-mode TUI library for Nim
  (github.com/invector-tecnologia/niobium). Use when composing Maestro panels
  from Niobium widgets, the constraint layout engine, the event decoder, the
  tick loop, or the test backend.
---

# Niobium Upstream Skill

Ground-truth reference for the Niobium TUI library that Maestro's `frontend/` consumes.

## Authoritative source
- Repo: `git@github.com:invector-tecnologia/niobium.git` — https://github.com/invector-tecnologia/niobium
- Pinned in Maestro via `scripts/install-niobium.sh` (commit `0051e112` = v0.1.0). Latest tag v0.1.1.
- Study `examples/` (`hello.nim` interactive, `demo.nim` headless) and `src/niobium/`
  (`core`, `layout`, `backend`, `terminal`, `event`, `widgets`).

## Capabilities (v1 core)
- Immediate mode: rebuild the whole UI each tick from state; Niobium diffs and writes only deltas.
- Constraint layout: `f.area.split(Vertical|Horizontal, @[...])` with `length`, `percentage`,
  `ratio`, `min`, `max`, `fill`.
- Widgets: `Block`, `Paragraph`, `List`, `Table`, `Tabs`, `Clear`, `Gauge`, `Sparkline`, `BarChart`,
  `Scrollbar`, `Chart`.
- Backends: `newAnsiBackend()` (TTY) and `newTestBackend(w, h)` (headless snapshots, no TTY).
- Tick loop + event decoder for keyboard input.

## Maestro mapping
- Mode switching → `Tabs` (Config · Maestro · Product).
- Config navigator → `List`/`Table`; editor → `Table`/`Paragraph`.
- Maestro Mode → `List` (persona activity), `Gauge`/`Tabs` (FSM stage), `Paragraph` (narration).
- Product Mode → `Paragraph` + `Scrollbar` (live output), `Chart`/`Sparkline` (metrics), side-by-side
  release notes via `split(Horizontal, ...)`.

## Guardrails
- Spec-first, like Niobium itself: adjust the panel spec, add a failing test-backend snapshot, implement.
- Do not reimplement primitives Niobium already ships; compose them.
