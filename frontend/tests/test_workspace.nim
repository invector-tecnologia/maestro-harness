## Headless Workspace snapshot tests (Tatui test backend, no TTY).
##
## Renders each of the three modes into an in-memory backend and asserts the
## plain-text snapshot, plus the pure `applyEvent`/mode-switch state folding.

import std/[json, os, strutils, unittest]
import tatui
import "../src/workspace"

proc renderToText(s: WorkspaceState): string =
  var term = newTerminal(newTestBackend(80, 16))
  term.draw proc(f: var Frame) =
    renderWorkspace(f, s)
  term.backend.render()

suite "workspace shell":
  test "config tab shows governance navigator":
    var s = newWorkspaceState(0)
    s.config.entries =
      @[
        ConfigEntryView(
          id: "personas/maestro", kind: "persona", origin: "default", archived: false
        )
      ]
    let r = renderToText(s)
    check r.contains("Config")
    check r.contains("Governance")
    check r.contains("personas/maestro")

  test "maestro tab shows personas and stage":
    var s = newWorkspaceState(1)
    s.maestro.stage = "Planning"
    s.maestro.agents = @[AgentView(name: "Software Engineer", state: "think")]
    let r = renderToText(s)
    check r.contains("Maestro")
    check r.contains("Software Engineer")
    check r.contains("Planning")

  test "product tab shows releases and live demo":
    var s = newWorkspaceState(2)
    s.product.releases = @[ReleaseView(version: "0.1.0", changelog: "initial")]
    let r = renderToText(s)
    check r.contains("Product")
    check r.contains("0.1.0")
    check r.contains("Live Demo")

  test "applyEvent folds mode_changed":
    var s = newWorkspaceState(0)
    s.applyEvent(parseJson("""{"v":2,"kind":"mode_changed","mode":"product"}"""))
    check s.modeIndex == 2

  test "applyEvent folds a log line into narration":
    var s = newWorkspaceState(1)
    s.applyEvent(parseJson("""{"v":2,"kind":"log","level":"info","message":"hi"}"""))
    check s.maestro.narration.len == 1
    check s.maestro.narration[0].contains("hi")

  test "cycleMode wraps around":
    var s = newWorkspaceState(2)
    s.cycleMode()
    check s.modeIndex == 0

  test "configSelectMove clamps and selectedConfigId tracks":
    var s = newWorkspaceState(0)
    s.config.entries =
      @[
        ConfigEntryView(id: "config.yml", kind: "config", origin: "default", archived: false),
        ConfigEntryView(
          id: "personas/maestro", kind: "persona", origin: "default", archived: false
        ),
      ]
    s.configSelectMove(-1) # clamp low
    check s.config.selected == 0
    check s.selectedConfigId == "config.yml"
    s.configSelectMove(1)
    check s.selectedConfigId == "personas/maestro"
    s.configSelectMove(5) # clamp high
    check s.config.selected == 1

  test "config_tree clamps a stale selection":
    var s = newWorkspaceState(0)
    s.config.selected = 9
    s.applyEvent(
      parseJson(
        """{"v":2,"kind":"config_tree","entries":[{"id":"config.yml","kind":"config","origin":"default","archived":false}]}"""
      )
    )
    check s.config.selected == 0

  test "selectedRelease tracks product selection":
    var s = newWorkspaceState(2)
    s.product.releases =
      @[ReleaseView(version: "0.2.0", changelog: ""), ReleaseView(version: "0.1.0", changelog: "")]
    s.productSelectMove(1)
    check s.selectedRelease == "0.1.0"

  test "approval_request sets pending approval and renders":
    var s = newWorkspaceState(1)
    check not s.hasPendingApproval
    s.applyEvent(
      parseJson(
        """{"v":2,"kind":"approval_request","id":"approve-plan","prompt":"Approve the plan?"}"""
      )
    )
    check s.hasPendingApproval
    check s.maestro.approvalId == "approve-plan"
    let r = renderToText(s)
    check r.contains("APPROVAL")
    check r.contains("Approve the plan?")
    s.clearApproval()
    check not s.hasPendingApproval

  test "ascii-only mode drops unicode borders":
    putEnv("MAESTRO_ASCII_ONLY", "1")
    var s = newWorkspaceState(0)
    s.config.entries =
      @[
        ConfigEntryView(id: "config.yml", kind: "config", origin: "default", archived: false)
      ]
    let r = renderToText(s)
    delEnv("MAESTRO_ASCII_ONLY")
    check not r.contains("\u2514") # box-drawing corner absent
    check r.contains("[Governance]")
