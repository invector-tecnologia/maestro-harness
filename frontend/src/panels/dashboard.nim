## Maestro dashboard panel (TASK 052).
##
## A pure render of a core snapshot into Niobium widgets: a header block, an
## agents panel, and a log panel laid out with constraints. No orchestration
## logic — the snapshot is supplied by the protocol client.

import niobium

type
  AgentView* = object ## One agent row: name and cognitive state.
    name*: string
    state*: string

  Snapshot* = object ## The TUI-side view of core state.
    title*: string
    agents*: seq[AgentView]
    logs*: seq[string]
    input*: string

proc renderDashboard*(f: var Frame, s: Snapshot) =
  ## Render the dashboard as a pure function of `s`.
  let rows = f.area.split(Vertical, @[length(3), fill(1), length(3)])
  f.renderWidget(initBlock(title = " " & s.title & " ", borders = AllBorders), rows[0])

  let body = rows[1].split(Horizontal, @[fill(1), fill(2)])

  var agentsText = ""
  for a in s.agents:
    agentsText.add(a.name & ": " & a.state & "\n")
  f.renderWidget(paragraph(agentsText), body[0])

  var logText = ""
  for line in s.logs:
    logText.add(line & "\n")
  f.renderWidget(paragraph(logText), body[1])

  f.renderWidget(initBlock(title = " Command ", borders = AllBorders), rows[2])
  f.renderWidget(paragraph("> " & s.input), rows[2])
