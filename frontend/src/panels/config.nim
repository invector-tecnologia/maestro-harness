## Config Mode panel — governance navigator + entry editor.
##
## Pure render of `ConfigState` into Tatui widgets. No orchestration or I/O; the
## state is supplied by the protocol client (`config_tree` / `config_entry`).

import tatui
import ../theme

type
  ConfigEntryView* = object ## One governance entry row.
    id*: string
    kind*: string ## config | persona | skill | scope
    origin*: string ## default | custom
    archived*: bool

  ConfigState* = object ## Config Mode view state.
    entries*: seq[ConfigEntryView]
    selected*: int
    body*: string ## The opened entry's body (editor pane).

proc renderConfig*(f: var Frame, area: Rect, s: ConfigState) =
  ## Render the governance navigator (left) and entry editor (right).
  let cols = area.split(Horizontal, @[fill(1), fill(1)])

  var nav = ""
  for i, e in s.entries:
    let marker = (if i == s.selected: "> " else: "  ")
    let arch = (if e.archived: "  [archived]" else: "")
    nav.add(marker & e.origin & "/" & e.kind & ": " & e.id & arch & "\n")
  if s.entries.len == 0:
    nav = "no governance entries\n"
  let navBlk = initBlock(title = panelTitle("Governance"), borders = panelBorders())
  f.renderWidget(navBlk, cols[0])
  f.renderWidget(paragraph(asciiHeader("Governance") & nav), navBlk.inner(cols[0]))

  let edBlk = initBlock(title = panelTitle("Editor"), borders = panelBorders())
  f.renderWidget(edBlk, cols[1])
  f.renderWidget(paragraph(asciiHeader("Editor") & s.body), edBlk.inner(cols[1]))
