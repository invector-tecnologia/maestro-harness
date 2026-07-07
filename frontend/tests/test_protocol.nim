## Headless protocol tests (no TTY, no Tatui required).
##
## Panel golden-snapshot tests using Tatui's test backend are added by TASK 052.

import std/[json, strutils, unittest]
import "../src/protocol"

suite "stdio protocol framing":
  test "frame is newline-delimited":
    check frame(%*{"kind": "heartbeat"}) == "{\"kind\":\"heartbeat\"}\n"

  test "round-trips a payload":
    let decoded = parseFrame(frame(%*{"v": ProtocolVersion}))
    check decoded["v"].getInt == ProtocolVersion

  test "protocol version is pinned":
    check ProtocolVersion == 2

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

  test "encodeCommand accepts a known command kind":
    let line = encodeCommand("switch_mode", %*{"mode": "config"})
    let node = decode(line)
    check node["kind"].getStr == "switch_mode"
    check node["mode"].getStr == "config"

  test "encodeCommand rejects an unknown command kind":
    expect ValueError:
      discard encodeCommand("bogus_command")

  test "decodeEvent accepts a known v2 event kind":
    let node = decodeEvent(encode("mode_changed", %*{"mode": "product"}))
    check node["mode"].getStr == "product"

  test "decodeEvent rejects an unknown event kind":
    expect ValueError:
      discard decodeEvent(encode("bogus_event"))
