# 0001. Rust core with a Nim/Niobium TUI over a stdio JSON protocol

- Status: accepted
- Date: 2026-07-06
- Deciders: Maestro maintainers

## Context
Maestro needs a fast, memory-safe orchestration brain and a rich, flicker-free interactive terminal
UI. The brain is best served by Rust (predictable execution, Tokio async, strong typing). The UI is
best served by [Niobium](https://github.com/invector-tecnologia/niobium), an immediate-mode,
ratatui-inspired TUI library distributed as a Nim `nimble` package (Nim ≥ 2.0). Because Niobium is a
Nim library — not a Rust crate and not a C-ABI artifact — the two stacks run on different runtimes.

## Decision
Split Maestro into two processes:

- A headless **Rust core** (`src/`, hexagonal DDD) that owns the FSM, Two-Towers routing, cascade,
  rollback, and git-standalone persistence. It is fully usable with `--no-tui`.
- A separate **Nim/Niobium TUI** (`frontend/`) that consumes Niobium and renders core state.

The processes communicate **only** over a **line-delimited JSON protocol on stdio** (duplex): the
core streams events (`agent_state`, `fsm_transition`, `log`, `metric`, `heartbeat`,
`approval_request`); the TUI sends commands (`user_input`, `command`, `approval_response`).

## Consequences
- **Positive:** clean language boundary; the core is headless and CI/automation friendly; the TUI is
  testable via Niobium's test backend (no TTY); no unsafe FFI marshalling.
- **Negative:** a serialization boundary and a versioned protocol to maintain.
- **Testable invariant:** the core produces no TUI-specific types; all coupling is the JSON protocol,
  and any change to the contract requires a new ADR.

## Alternatives considered
- **Direct FFI (Nim compiled to a C library, called from Rust):** rejected — complex marshalling and
  lifetime hazards for little gain over a stdio protocol.
- **Pure-Rust ratatui TUI:** rejected — the project standardizes on Niobium for the frontend.
- **Unix domain socket instead of stdio:** viable, deferred — stdio is simpler and matches the
  existing narration/heartbeat model; revisit if richer multiplexing is needed.
