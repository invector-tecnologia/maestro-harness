## Product Mode panel — shipped releases + live demo output.
##
## Pure render of `ProductState`: releases and their notes (left) side-by-side with
## the live artifact output (right). State is supplied by the protocol client
## (`release_list` / `demo_output` / `demo_exited`).

import tatui
import ../theme

type
  ReleaseView* = object ## One shipped release.
    version*: string
    changelog*: string

  ProductState* = object ## Product Mode view state.
    releases*: seq[ReleaseView]
    selected*: int
    output*: seq[string] ## Live demo output lines.
    running*: bool

proc renderProduct*(f: var Frame, area: Rect, s: ProductState) =
  ## Render releases + notes (left) and live demo output (right).
  let cols = area.split(Horizontal, @[fill(1), fill(1)])

  var rel = ""
  for i, r in s.releases:
    let marker = (if i == s.selected: "> " else: "  ")
    rel.add(marker & r.version & "\n")
  if s.releases.len == 0:
    rel = "no releases shipped\n"
  elif s.selected >= 0 and s.selected < s.releases.len:
    rel.add("\n" & s.releases[s.selected].changelog & "\n")
  let relBlk =
    initBlock(title = panelTitle("Releases & Notes"), borders = panelBorders())
  f.renderWidget(relBlk, cols[0])
  f.renderWidget(
    paragraph(asciiHeader("Releases & Notes") & rel), relBlk.inner(cols[0])
  )

  var outText = (if s.running: "[running]\n" else: "[idle]\n")
  for line in s.output:
    outText.add(line & "\n")
  let demoBlk = initBlock(title = panelTitle("Live Demo"), borders = panelBorders())
  f.renderWidget(demoBlk, cols[1])
  f.renderWidget(paragraph(asciiHeader("Live Demo") & outText), demoBlk.inner(cols[1]))
