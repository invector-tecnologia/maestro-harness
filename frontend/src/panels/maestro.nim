## Maestro Mode panel — orchestration monitor.
##
## Pure render of `MaestroState`: the personas Maestro delegates to (left) and the
## live narration / FSM stage (right). State is supplied by the protocol client.

import niobium

type
  AgentView* = object ## One persona row: name and cognitive state.
    name*: string
    state*: string

  MaestroState* = object ## Maestro Mode view state.
    stage*: string ## Current FSM stage.
    agents*: seq[AgentView]
    narration*: seq[string]

proc renderMaestro*(f: var Frame, area: Rect, s: MaestroState) =
  ## Render personas (left) and Maestro narration + stage (right).
  let cols = area.split(Horizontal, @[fill(1), fill(2)])

  var agentsText = ""
  for a in s.agents:
    agentsText.add(a.name & ": " & a.state & "\n")
  if s.agents.len == 0:
    agentsText = "no active personas\n"
  let pBlk = initBlock(title = " Personas ", borders = AllBorders)
  f.renderWidget(pBlk, cols[0])
  f.renderWidget(paragraph(agentsText), pBlk.inner(cols[0]))

  var narr = "Stage: " & (if s.stage.len > 0: s.stage else: "idle") & "\n\n"
  for line in s.narration:
    narr.add(line & "\n")
  let mBlk = initBlock(title = " Maestro ", borders = AllBorders)
  f.renderWidget(mBlk, cols[1])
  f.renderWidget(paragraph(narr), mBlk.inner(cols[1]))
