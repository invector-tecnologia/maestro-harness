## Maestro TUI application shell (TASK 052).
##
## Immediate-mode tick loop: read core events over the stdio protocol, update the
## snapshot, draw the frame with Niobium, forward input. The draw is a pure
## function of the latest snapshot — no orchestration logic lives here.

import niobium
import ./protocol
import ./panels/dashboard

proc demoSnapshot*(): Snapshot =
  ## A placeholder snapshot until the protocol client is wired to the core.
  Snapshot(
    title: "Maestro — Protocol v" & $ProtocolVersion,
    agents: @[
      AgentView(name: "Project Manager", state: "idle"),
      AgentView(name: "Software Engineer", state: "idle"),
      AgentView(name: "Quality Assurance", state: "idle"),
      AgentView(name: "User Experience", state: "idle"),
    ],
    logs: @["awaiting core…"],
    input: "",
  )

proc run*() =
  ## Entry point for the interactive TUI. Sets up the terminal, runs the tick
  ## loop, and always restores terminal state on exit.
  var term = newTerminal(newAnsiBackend())
  term.setup()
  defer:
    term.restore()

  let snapshot = demoSnapshot()
  term.draw proc(f: var Frame) =
    renderDashboard(f, snapshot)
