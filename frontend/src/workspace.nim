## Maestro Workspace shell — the three-mode container (ADR 0002).
##
## Renders a `Tabs` header (Config · Maestro · Product), the active mode's panel,
## and a command footer. `renderWorkspace` is a pure function of `WorkspaceState`;
## `applyEvent` folds a decoded core event into the state. No I/O lives here.

import std/json
import niobium
import ./theme
import ./panels/config
import ./panels/maestro
import ./panels/product

export config, maestro, product

const ModeTitles* = ["Config", "Maestro", "Product"]
const ModeNames* = ["config", "maestro", "product"]

type WorkspaceState* = object ## The whole TUI-side view of core state.
  modeIndex*: int ## 0 = Config, 1 = Maestro, 2 = Product.
  config*: ConfigState
  maestro*: MaestroState
  product*: ProductState
  input*: string
  status*: string
  running*: bool ## Whether the tick loop should keep running.

proc newWorkspaceState*(startMode = 1): WorkspaceState =
  ## A fresh state; defaults to Maestro Mode (index 1) per the `init` hand-off.
  WorkspaceState(
    modeIndex: startMode,
    status: "F1 Config  F2 Maestro  F3 Product  |  Tab cycle  |  Esc quit",
    running: true,
  )

proc modeName*(s: WorkspaceState): string =
  ## The snake_case name of the active mode.
  ModeNames[s.modeIndex]

proc modeIndexOf*(name: string): int =
  ## The index of a mode name, or -1 if unknown.
  for i, n in ModeNames:
    if n == name:
      return i
  -1

proc switchTo*(s: var WorkspaceState, idx: int) =
  ## Switch the active mode by index (bounds-checked).
  if idx >= 0 and idx < ModeTitles.len:
    s.modeIndex = idx

proc cycleMode*(s: var WorkspaceState) =
  ## Advance to the next mode, wrapping around.
  s.modeIndex = (s.modeIndex + 1) mod ModeTitles.len

proc clampSelection(sel, count: int): int =
  if count <= 0: 0
  elif sel < 0: 0
  elif sel >= count: count - 1
  else: sel

proc configSelectMove*(s: var WorkspaceState, delta: int) =
  ## Move the Config navigator selection, clamped to the entry list.
  s.config.selected = clampSelection(s.config.selected + delta, s.config.entries.len)

proc selectedConfigId*(s: WorkspaceState): string =
  ## The id of the selected Config entry, or "" if none.
  if s.config.selected >= 0 and s.config.selected < s.config.entries.len:
    s.config.entries[s.config.selected].id
  else:
    ""

proc productSelectMove*(s: var WorkspaceState, delta: int) =
  ## Move the Product release selection, clamped to the release list.
  s.product.selected = clampSelection(s.product.selected + delta, s.product.releases.len)

proc selectedRelease*(s: WorkspaceState): string =
  ## The version of the selected release, or "" if none.
  if s.product.selected >= 0 and s.product.selected < s.product.releases.len:
    s.product.releases[s.product.selected].version
  else:
    ""

proc hasPendingApproval*(s: WorkspaceState): bool =
  ## Whether Maestro is blocked awaiting the user's approval.
  s.maestro.approvalId.len > 0

proc clearApproval*(s: var WorkspaceState) =
  ## Clear the pending approval after the user responds.
  s.maestro.approvalId = ""
  s.maestro.approvalPrompt = ""

proc applyEvent*(s: var WorkspaceState, node: JsonNode) =
  ## Fold a decoded core event (`{ "v": 2, "kind": ..., ... }`) into the state.
  let kind = node{"kind"}.getStr("")
  case kind
  of "mode_changed":
    let idx = modeIndexOf(node{"mode"}.getStr(""))
    if idx >= 0: s.modeIndex = idx
  of "log":
    s.maestro.narration.add(
      node{"level"}.getStr("info") & ": " & node{"message"}.getStr("")
    )
  of "agent_state":
    let name = node{"agent"}.getStr("")
    let state = node{"state"}.getStr("")
    var found = false
    for a in s.maestro.agents.mitems:
      if a.name == name:
        a.state = state
        found = true
    if not found and name.len > 0:
      s.maestro.agents.add(AgentView(name: name, state: state))
  of "fsm_transition":
    s.maestro.stage = node{"to"}.getStr(s.maestro.stage)
  of "plan_proposed":
    for step in node{"steps"}.getElems():
      s.maestro.narration.add("plan: " & step.getStr(""))
  of "delegation":
    s.maestro.narration.add(
      "delegate " & node{"persona"}.getStr("") & " <- " & node{"task"}.getStr("")
    )
  of "deliverable":
    s.maestro.narration.add(
      "delivered " & node{"persona"}.getStr("") & ": " & node{"summary"}.getStr("")
    )
  of "approval_request":
    s.maestro.approvalId = node{"id"}.getStr("")
    s.maestro.approvalPrompt = node{"prompt"}.getStr("")
    s.maestro.narration.add("approval requested: " & s.maestro.approvalPrompt)
  of "config_tree":
    s.config.entries = @[]
    for e in node{"entries"}.getElems():
      s.config.entries.add(
        ConfigEntryView(
          id: e{"id"}.getStr(""),
          kind: e{"kind"}.getStr(""),
          origin: e{"origin"}.getStr(""),
          archived: e{"archived"}.getBool(false),
        )
      )
    s.config.selected = clampSelection(s.config.selected, s.config.entries.len)
  of "config_entry":
    s.config.body = node{"body"}.getStr("")
  of "release_list":
    s.product.releases = @[]
    for r in node{"releases"}.getElems():
      s.product.releases.add(
        ReleaseView(version: r{"version"}.getStr(""), changelog: r{"changelog"}.getStr(""))
      )
  of "demo_output":
    s.product.running = true
    s.product.output.add(node{"chunk"}.getStr(""))
  of "demo_exited":
    s.product.running = false
    s.product.output.add("[exited " & $node{"code"}.getInt(0) & "]")
  else:
    discard

proc renderWorkspace*(f: var Frame, s: WorkspaceState) =
  ## Render the tab bar, the active mode's panel, and the command footer.
  let rows = f.area.split(Vertical, @[length(1), fill(1), length(3)])
  f.renderWidget(tabs(ModeTitles, selected = s.modeIndex), rows[0])

  case s.modeIndex
  of 0: renderConfig(f, rows[1], s.config)
  of 2: renderProduct(f, rows[1], s.product)
  else: renderMaestro(f, rows[1], s.maestro)

  let footer = initBlock(title = panelTitle("Command"), borders = panelBorders())
  f.renderWidget(footer, rows[2])
  f.renderWidget(
    paragraph(asciiHeader("Command") & "> " & s.input & "    " & s.status), footer.inner(rows[2])
  )
