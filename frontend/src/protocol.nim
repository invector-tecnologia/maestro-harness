## Maestro TUI ↔ core stdio protocol (line-delimited JSON), protocol v2.
##
## Frontend mirror of `src/presentation/ipc` in the Rust core. Framing is
## unchanged from v1 (`{ "v": 2, "kind": "...", ... }\n`); v2 adds mode switching
## and mode-scoped Config/Maestro/Product payloads (ADR 0003). Unknown `kind`s and
## version mismatches are rejected — no silent forward-compat.

import std/json
import std/strutils

const ProtocolVersion* = 2
  ## Bumped only alongside an ADR describing the contract change (ADR 0003).

const CoreEventKinds* = [
  # v1
  "agent_state",
  "fsm_transition",
  "log",
  "metric",
  "heartbeat",
  "approval_request",
  # v2 — mode + Config + Maestro + Product
  "mode_changed",
  "config_tree",
  "config_entry",
  "config_validation",
  "config_saved",
  "plan_proposed",
  "delegation",
  "deliverable",
  "release_list",
  "demo_output",
  "demo_exited",
] ## Every core→TUI event kind the frontend accepts (mirrors `CoreEvent`).

const TuiCommandKinds* = [
  # v1
  "user_input",
  "command",
  "approval_response",
  # v2
  "switch_mode",
  "config_list",
  "config_open",
  "config_edit",
  "config_create",
  "config_archive",
  "config_validate",
  "config_save",
  "list_releases",
  "run_demo",
  "stop_demo",
] ## Every TUI→core command kind the frontend emits (mirrors `TuiCommand`).

const WorkspaceModes* = ["config", "maestro", "product"]
  ## The three Workspace modes (ADR 0002).

proc frame*(payload: JsonNode): string =
  ## Encode a payload as a single newline-delimited JSON frame.
  $payload & "\n"

proc parseFrame*(line: string): JsonNode =
  ## Decode a single newline-delimited JSON frame.
  parseJson(line)

proc knownKind(kinds: openArray[string], kind: string): bool =
  for k in kinds:
    if k == kind:
      return true
  false

proc encode*(kind: string, fields: JsonNode = newJObject()): string =
  ## Encode a versioned, kind-tagged frame (mirrors the Rust core `encode`).
  var obj = %*{"v": ProtocolVersion, "kind": kind}
  for key, val in fields:
    obj[key] = val
  $obj & "\n"

proc encodeCommand*(kind: string, fields: JsonNode = newJObject()): string =
  ## Encode a TUI→core command frame, rejecting unknown command kinds.
  if not knownKind(TuiCommandKinds, kind):
    raise newException(ValueError, "unknown TUI command kind: " & kind)
  encode(kind, fields)

proc decode*(line: string): JsonNode =
  ## Decode a versioned frame, rejecting version mismatches (mirrors Rust `decode`).
  let node = parseJson(line.strip())
  let version = node{"v"}.getInt(-1)
  if version != ProtocolVersion:
    raise newException(ValueError, "unsupported protocol version: " & $version)
  node

proc decodeEvent*(line: string): JsonNode =
  ## Decode a core→TUI event frame, rejecting version mismatches and unknown kinds.
  let node = decode(line)
  let kind = node{"kind"}.getStr("")
  if not knownKind(CoreEventKinds, kind):
    raise newException(ValueError, "unknown core event kind: " & kind)
  node
