## Headless dashboard snapshot test (TASK 052).
##
## Renders the dashboard into Niobium's in-memory test backend (no TTY) and
## asserts the plain-text snapshot, per the render-path testing convention.

import std/[strutils, unittest]
import niobium
import "../src/panels/dashboard"

suite "dashboard rendering":
  test "renders title and agents into the test backend":
    var term = newTerminal(newTestBackend(60, 12))
    let snapshot = Snapshot(
      title: "Maestro",
      agents: @[AgentView(name: "Software Engineer", state: "think")],
      logs: @["ready"],
    )
    term.draw proc(f: var Frame) =
      renderDashboard(f, snapshot)

    let rendered = term.backend.render()
    check rendered.len > 0
    check rendered.contains("Maestro")
    check rendered.contains("Software Engineer")
