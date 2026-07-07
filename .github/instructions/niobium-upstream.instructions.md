---
applyTo: "frontend/**/*.nim"
description: "Reference the Niobium TUI library (github.com/invector-tecnologia/niobium) as ground-truth for widgets, constraint layout, the event decoder, the tick loop, and the test backend."
---

# Niobium Upstream Reference

- Authoritative source: https://github.com/invector-tecnologia/niobium
  (`git@github.com:invector-tecnologia/niobium.git`). Pinned via `scripts/install-niobium.sh`
  (commit `0051e112` = v0.1.0).
- Compose panels only from shipped widgets: `Block`, `Paragraph`, `List`, `Table`, `Tabs`, `Clear`,
  `Gauge`, `Sparkline`, `BarChart`, `Scrollbar`, `Chart`. Do not reimplement primitives.
- Layout via `f.area.split(...)` constraints (`length`/`percentage`/`ratio`/`min`/`max`/`fill`).
- Study `examples/` and `src/niobium/{core,layout,backend,terminal,event,widgets}` for real usage.
- Test headlessly with `newTestBackend(w, h)`; snapshot the rendered text — no TTY required.
- Local doctrine wins: the `niobium-frontend` skill and `nim-frontend.instructions.md`.
