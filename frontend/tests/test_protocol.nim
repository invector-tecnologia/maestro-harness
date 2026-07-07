## Headless protocol tests (no TTY, no Niobium required).
##
## Panel golden-snapshot tests using Niobium's test backend are added by TASK 052.

import std/[json, strutils, unittest]
import "../src/protocol"

suite "stdio protocol framing":
  test "frame is newline-delimited":
    check frame(%*{"kind": "heartbeat"}) == "{\"kind\":\"heartbeat\"}\n"

  test "round-trips a payload":
    let decoded = parseFrame(frame(%*{"v": ProtocolVersion}))
    check decoded["v"].getInt == ProtocolVersion

  test "protocol version is pinned":
    check ProtocolVersion == 1

  test "encode produces a versioned kind-tagged frame":
    let line = encode("heartbeat", %*{"seq": 1})
    check line.endsWith("\n")
    let node = decode(line)
    check node["v"].getInt == ProtocolVersion
    check node["kind"].getStr == "heartbeat"
    check node["seq"].getInt == 1

  test "decode rejects an unsupported version":
    expect ValueError:
      discard decode("{\"v\": 99, \"kind\": \"heartbeat\"}")
