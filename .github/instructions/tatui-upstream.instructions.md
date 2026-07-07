---
applyTo: "frontend/**/*.nim"
description: "Reference the Tatui TUI library (github.com/invector-tecnologia/tatui) as ground-truth for widgets, constraint layout, the event decoder, the tick loop, and the test backend."
---

# Tatui Upstream Reference

- Authoritative source: https://github.com/invector-tecnologia/tatui
  (`git@github.com:invector-tecnologia/tatui.git`). Pinned via `scripts/install-tatui.sh`
  (commit `493d9fc0` = v0.1.2).
- Compose panels only from shipped widgets: `Block`, `Paragraph`, `List`, `Table`, `Tabs`, `Clear`,
  `Gauge`, `Sparkline`, `BarChart`, `Scrollbar`, `Chart`. Do not reimplement primitives.
- Layout via `f.area.split(...)` constraints (`length`/`percentage`/`ratio`/`min`/`max`/`fill`).
- Study `examples/` and `src/tatui/{core,layout,backend,terminal,event,widgets}` for real usage.
- Test headlessly with `newTestBackend(w, h)`; snapshot the rendered text — no TTY required.
- Local doctrine wins: the `tatui-frontend` skill and `nim-frontend.instructions.md`.
