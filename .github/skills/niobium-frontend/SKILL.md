---
name: niobium-frontend
description: "Use when building or reviewing Maestro's Nim/Niobium TUI: mapping Maestro panels to Niobium widgets, constraint layout, the stdio protocol client, and headless snapshot tests."
---

# Niobium Frontend Skill

## Use When
- Building or changing a TUI panel in `frontend/`.
- Wiring the TUI to the Rust core's stdio protocol.

## Niobium at a glance
- Install: `nimble install niobium` (Nim ≥ 2.0). Pin `requires "niobium >= 0.1.0"`.
- Immediate mode: redraw the whole UI each tick from state; Niobium diffs and writes only changes.
- Constraint layout: `Length`, `Percentage`, `Ratio`, `Min`, `Max`, `Fill`; split with `f.area.split(...)`.
- Test backend renders a `Buffer` to text with no TTY → golden snapshots in CI.

### API taste
```nim
import niobium
var term = newTerminal(newAnsiBackend())
term.setup()
defer: term.restore()
term.draw proc(f: var Frame) =
  let chunks = f.area.split(Vertical, @[length(3), fill(1)])
  f.renderWidget(initBlock(title = " Maestro ", borders = AllBorders), chunks[0])
  f.renderWidget(paragraph("…"), chunks[1])
```

## Panel → widget map
| Maestro panel | Niobium widgets |
|---|---|
| Chat / conversation | Block + Paragraph + List + Scrollbar |
| Agent roster + status (`idle/observe/think/act/error`) | Table/List + Gauge |
| FSM stepper (Ideation→…→Verification) | Tabs + Block |
| Event / message log | List + Scrollbar |
| Project lists (available/done) | Tabs + Table/List |
| System / health metrics | Sparkline + BarChart + Gauge + Chart |
| Command input | Block input + event decoder |
| Approval / rollback modal | Clear + Block + Paragraph + List |

## Rules
- Consume Niobium only; never re-implement buffer/diff/backend/layout.
- Keep the draw function a pure function of the latest core snapshot; no business logic in the TUI.
- Talk to the core only via the line-delimited JSON stdio protocol (`protocol.nim`).
- Restore terminal state with `defer` even on error.

## Outputs
- A panel composed from shipped widgets, driven by core events, covered by test-backend snapshots.
