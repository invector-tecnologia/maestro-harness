## Maestro TUI application shell (ADR 0002 / TASK 053).
##
## Immediate-mode tick loop: spawn the headless core (`maestro run`), read its
## events over the stdio protocol, fold them into `WorkspaceState`, draw the three
## mode tabs with Tatui, and forward keyboard input as commands. The draw is a
## pure function of the latest state — no orchestration logic lives here.

import std/[json, options, os, osproc, posix, streams, strutils, unicode]
import tatui
import ./protocol
import ./workspace

proc corePath(): string =
  ## The core binary to spawn: `$MAESTRO_CORE` if set, else `maestro` on PATH.
  result = getEnv("MAESTRO_CORE")
  if result.len == 0:
    result = "maestro"

proc setNonBlocking(fd: cint) =
  let flags = fcntl(fd, F_GETFL, 0)
  if flags != -1:
    discard fcntl(fd, F_SETFL, flags or O_NONBLOCK)

proc drainCore(s: var WorkspaceState, fd: cint, buf: var string) =
  ## Read whatever is available on the core's stdout and fold complete frames.
  if fd < 0:
    return
  var chunk = newString(4096)
  while true:
    let n = read(fd, addr chunk[0], 4096)
    if n <= 0:
      break
    buf.add(chunk[0 ..< n])
  var nl = buf.find('\n')
  while nl >= 0:
    let line = buf[0 ..< nl]
    buf = buf[nl + 1 .. ^1]
    if line.strip().len > 0:
      try:
        s.applyEvent(decodeEvent(line))
      except CatchableError:
        discard
    nl = buf.find('\n')

proc sendCommand(inStream: Stream, kind: string, fields: JsonNode = newJObject()) =
  ## Best-effort write of a TUI→core command frame.
  if inStream == nil:
    return
  try:
    inStream.write(encodeCommand(kind, fields))
    inStream.flush()
  except CatchableError:
    discard

proc onModeEntered(s: WorkspaceState, inStream: Stream) =
  ## Notify the core of the active mode and request its initial data.
  inStream.sendCommand("switch_mode", %*{"mode": s.modeName})
  case s.modeName
  of "config":
    inStream.sendCommand("config_list")
  of "product":
    inStream.sendCommand("list_releases")
  else:
    discard

proc handleKey(s: var WorkspaceState, ev: KeyEvent, inStream: Stream) =
  ## Update state and emit commands from a single key event.
  case ev.code
  of kcEsc:
    s.running = false
  of kcFunction:
    if ev.function in 1 .. ModeTitles.len:
      s.switchTo(ev.function - 1)
      s.onModeEntered(inStream)
  of kcTab:
    s.cycleMode()
    s.onModeEntered(inStream)
  of kcUp:
    if s.modeName == "config":
      s.configSelectMove(-1)
    elif s.modeName == "product":
      s.productSelectMove(-1)
  of kcDown:
    if s.modeName == "config":
      s.configSelectMove(1)
    elif s.modeName == "product":
      s.productSelectMove(1)
  of kcBackspace:
    if s.input.len > 0:
      s.input.setLen(s.input.len - 1)
  of kcEnter:
    let text = s.input.strip()
    if text.len > 0:
      if text.startsWith("/"):
        inStream.sendCommand("command", %*{"name": text[1 .. ^1]})
      else:
        inStream.sendCommand("user_input", %*{"text": text})
      s.input = ""
    elif s.modeName == "config":
      let id = s.selectedConfigId
      if id.len > 0:
        inStream.sendCommand("config_open", %*{"id": id})
    elif s.modeName == "product":
      let rel = s.selectedRelease
      if rel.len > 0:
        inStream.sendCommand("run_demo", %*{"release": rel})
  of kcChar:
    let ch = $ev.rune
    if kmCtrl in ev.mods and (ch == "q" or ch == "Q"):
      s.running = false
      return
    if s.modeName == "maestro" and s.hasPendingApproval and ch in ["y", "Y", "n", "N"]:
      let approved = ch in ["y", "Y"]
      inStream.sendCommand(
        "approval_response", %*{"id": s.maestro.approvalId, "approved": approved}
      )
      s.clearApproval()
    elif ev.mods.card == 0 or ev.mods == {kmShift}:
      s.input.add(ch)
  else:
    discard

proc run*() =
  ## Entry point for the interactive TUI. Owns the terminal, spawns the core, and
  ## runs the tick loop until the user quits. Always restores the terminal on exit.
  var
    core: Process = nil
    inStream: Stream = nil
    outFd: cint = -1
  try:
    core = startProcess(corePath(), args = ["run"], options = {poUsePath})
    inStream = core.inputStream
    outFd = core.outputHandle.cint
    setNonBlocking(outFd)
  except OSError:
    core = nil

  var state = newWorkspaceState()
  if core == nil:
    state.status = "core not attached (set MAESTRO_CORE) — " & state.status

  var buf = ""
  var term = newTerminal(newAnsiBackend())
  term.setup()
  defer:
    term.restore()
    if inStream != nil:
      inStream.close()
    if core != nil:
      core.terminate()
      discard core.waitForExit()

  while state.running:
    state.drainCore(outFd, buf)
    let ev = pollEvent(50)
    if ev.isSome:
      let e = ev.get
      if e.kind == evKey:
        state.handleKey(e.key, inStream)
    term.draw proc(f: var Frame) =
      renderWorkspace(f, state)
