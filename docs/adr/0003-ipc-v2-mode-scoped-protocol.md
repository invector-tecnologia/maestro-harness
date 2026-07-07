# 0003. IPC protocol v2 — mode-scoped events and commands

- Status: accepted
- Date: 2026-07-07
- Deciders: Maestro maintainers

## Context
The v1 stdio protocol (ADR [0001](0001-rust-core-nim-niobium-tui-stdio-protocol.md)) defines a small
event/command set sufficient for a single dashboard: `agent_state`, `fsm_transition`, `log`,
`metric`, `heartbeat`, `approval_request` (core→TUI) and `user_input`, `command`,
`approval_response` (TUI→core). The three-mode Workspace (ADR
[0002](0002-three-mode-workspace-and-interview-removal.md)) needs the TUI to switch modes and to
carry mode-specific payloads: Config governance CRUD, Maestro orchestration detail, and Product
release/demo streaming. These do not fit the v1 set, and per the invariant in ADR 0001 any contract
change requires an ADR.

## Decision
Bump `PROTOCOL_VERSION` from `1` to `2` and extend the kind-tagged, newline-delimited JSON envelope
(unchanged framing: `{ "v": 2, "kind": "...", ... }`) with mode-scoped variants. The existing v1
variants are retained. Added variants:

- **Mode switching:** `TuiCommand::switch_mode { mode }`, `CoreEvent::mode_changed { mode }`
  (`mode` ∈ `config | maestro | product`).
- **Config Mode:** `CoreEvent::config_tree`, `config_entry`, `config_validation`, `config_saved`;
  `TuiCommand::config_list`, `config_open`, `config_edit`, `config_create`, `config_archive`,
  `config_validate`, `config_save`.
- **Maestro Mode:** reuse `user_input` for the demand; add `CoreEvent::plan_proposed`,
  `delegation`, `deliverable` atop existing narration and `approval_request`.
- **Product Mode:** `CoreEvent::release_list`, `demo_output`, `demo_exited`;
  `TuiCommand::list_releases`, `run_demo`, `stop_demo`.

The Rust `src/presentation/ipc` module remains the source of truth; `frontend/src/protocol.nim`
mirrors it. Unknown `kind` values continue to be rejected (no silent forward-compat).

## Consequences
- **Positive:** one envelope serves all three modes; strict version pinning keeps both sides honest;
  additions are localised behind `#[serde(tag = "kind")]`.
- **Negative:** a v1↔v2 flag day — the Nim client and Rust core must ship the bump together; no mixed
  versions across the boundary.
- **Testable invariant:** every added variant round-trips through `encode`/`decode` on both sides;
  decoding a frame with `v != 2` returns `UnsupportedVersion`; decoding an unknown `kind` is rejected.

## Alternatives considered
- **Additive on v1 without a version bump:** rejected — violates the ADR-gated contract rule and
  breaks the "unknown kind is rejected" guarantee for older clients.
- **A generic `mode_payload { json }` catch-all:** rejected — erases schema checking and pushes
  parsing/validation into both sides ad hoc.
- **Separate sockets per mode:** rejected — stdio duplex already suffices; multiplexing is premature.
