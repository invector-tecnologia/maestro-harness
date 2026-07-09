# TASK 051: Rust↔Nim IPC Stdio Protocol

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** Rust core events, TUI commands.
* **Context Anchors:** #file:docs/adr/0001-rust-core-nim-tatui-stdio-protocol.md, #file:docs/Maestro_Manifesto/ARCHITECTURE.md
* **Expected Output:** A versioned, schema-checked line-delimited JSON protocol on stdio.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* The protocol is the only coupling point; neither side assumes the other's internal types.
* Messages are versioned and schema-validated; unknown/invalid messages are rejected safely.
* Any contract change requires a new ADR.

## 3. EXECUTION PROMPT (Paste into Copilot Chat)
"""
Act as a Rust Presentation Engineer.
Goal: Implement the stdio JSON protocol in `src/presentation/ipc/` and mirror it in `frontend/src/protocol.nim`.

Before generating code, open a `<reasoning>` block and model the event set (agent_state, fsm_transition, log, metric, heartbeat, approval_request) and command set (user_input, command, approval_response).

Execute:
1. Define versioned message types and serialization on the Rust side.
2. Implement line-delimited framing and duplex read/write.
3. Provide the mirrored Nim decoder/encoder for the frontend.
4. Add tests for round-trip serialization and rejection of malformed/unknown messages.

[Cohesion Mechanism]:
- Confirm the core emits no TUI-specific types.

Return ONLY the modified code blocks in Markdown. No introduction.
"""

## 4. Acceptance Criteria
* **AC1:** Round-trip serialization is covered for every message type.
* **AC2:** Malformed or unknown messages are rejected without panicking.
* **AC3:** The message schema is versioned and referenced by an ADR.
