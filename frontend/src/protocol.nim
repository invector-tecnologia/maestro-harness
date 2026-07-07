## Maestro TUI ↔ core stdio protocol (line-delimited JSON).
##
## This is the frontend mirror of `src/presentation/ipc` in the Rust core. The full
## versioned message schema (core→TUI events, TUI→core commands) is delivered by
## TASK 051. This bootstrap provides the framing primitives so the contract can be
## tested headlessly before the schema lands.

import std/json
import std/strutils

const ProtocolVersion* = 1
  ## Bumped only alongside an ADR describing the contract change.

proc frame*(payload: JsonNode): string =
  ## Encode a payload as a single newline-delimited JSON frame.
  $payload & "\n"

proc parseFrame*(line: string): JsonNode =
  ## Decode a single newline-delimited JSON frame.
  parseJson(line)

proc encode*(kind: string, fields: JsonNode = newJObject()): string =
  ## Encode a versioned, kind-tagged frame (mirrors the Rust core `encode`).
  var obj = %*{"v": ProtocolVersion, "kind": kind}
  for key, val in fields:
    obj[key] = val
  $obj & "\n"

proc decode*(line: string): JsonNode =
  ## Decode a versioned frame, rejecting version mismatches (mirrors Rust `decode`).
  let node = parseJson(line.strip())
  let version = node{"v"}.getInt(-1)
  if version != ProtocolVersion:
    raise newException(ValueError, "unsupported protocol version: " & $version)
  node

