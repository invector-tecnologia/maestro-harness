# MAESTRO AI HARNESS: Architecture Guidelines

## 1. Overview and Paradigm
Maestro AI Harness is a complete multi-agent orchestration ecosystem for software engineering. The architecture is based on:
- **Actor Model (Event-Driven):** Agents are autonomous entities running in async tasks (`tokio::spawn`) and communicating only through asynchronous message exchange.
- **Hexagonal Architecture (Ports and Adapters):** Strict isolation between agent decision logic (Domain) and external AI/system APIs (Infrastructure).
- **AI Harness (Control and Evaluation):** A safe sandbox where AIs run with scoped context, token monitoring, and continuous validation (Quality Gates) before executing tasks.
- **Two-Process Split:** A headless **Rust core** (the brain) and a separate **Nim/Tatui TUI** (the nervous system) that communicate only over a line-delimited JSON protocol on stdio. The core is fully usable with `--no-tui`; the TUI never embeds core logic.

## 2. Directory Topology (Strict DDD)
All generated code must respect the following segregation in `src/`:

```text
src/
├── domain/         # Core. Zero I/O dependencies, external APIs, or heavy frameworks.
│   ├── models/     # Entities and Value Objects (for example: Message, Role, Memory).
│   └── ports/      # Traits (interfaces) implemented by infrastructure (for example: LlmProvider).
├── application/    # Use cases and orchestration; environment and agent lifecycle live here.
│   └── sops/       # Standard Operating Procedures for agents.
├── infrastructure/ # Port implementations and external integrations.
│   ├── llm/        # Adapters for Ollama, Gemini, and future providers.
│   ├── bus/        # Event bus implementation (for example: tokio::sync::broadcast).
│   └── harness/    # Sandbox, token limits, and AI action safety auditing.
└── presentation/   # Entry points and UX surfaces.
    ├── cli/        # CLI argument parsing (clap) and startup wiring (--no-tui).
    └── ipc/        # Line-delimited JSON protocol: core → TUI events, TUI → core commands.
```

The interactive TUI is a **separate Nim process**, not a Rust module:

```text
frontend/            # Nim/Tatui TUI process (consumes Tatui via nimble).
├── maestro_tui.nimble   # requires "tatui >= 0.1.0"
├── src/
│   ├── app.nim      # tick loop: read core events, render frame, forward input.
│   ├── protocol.nim # decode/encode the stdio JSON contract (mirrors src/presentation/ipc).
│   └── panels/      # chat, agents, fsm stepper, logs, projects, metrics (Tatui widgets only).
└── tests/           # golden snapshots via Tatui's test backend (no TTY).
```

Tatui is a **dependency**, never vendored or re-implemented. Panels compose only shipped Tatui
widgets (Block, Paragraph, List, Table, Tabs, Clear, Gauge, Sparkline, BarChart, Scrollbar, Chart)
and constraint layout (`Length`, `Percentage`, `Ratio`, `Min`, `Max`, `Fill`).

## 3. Canonical Cognitive Pattern
Every agent — personas, retrieval, and the orchestrator — runs the same
cognitive cycle: **SENSE → OBSERVE → THINK → ACT → AUDIT → DELIVER**. The innermost
`observe → think → act` loop is the `Role` trait; `SENSE`, `AUDIT`, and `DELIVER` are
orchestration-level stages that wrap it during collaboration. See
[reference/COGNITIVE_PATTERN.md](reference/COGNITIVE_PATTERN.md) for the canonical
definition and the code map.

## 4. Micro-Project Finite State Machine
Every micro-project advances through a strict FSM owned by the Rust core:

`Ideation → Planning → Approval → Instrumentation → Execution → Verification`

- Transitions are explicit and logged via `tracing`; illegal transitions are rejected, not ignored.
- **Approval** and the pre-Execution **rollback gate** require explicit user consent relayed over IPC.
- **Instrumentation** dynamically generates personas and injects their system prompts, skills, RAG,
  and `.spec`/`.json` files.
- The FSM lives in `application/` and is driven by domain state; the TUI only *renders* the current
  stage (a Tabs/Block stepper) and *relays* approvals.

## 5. Two-Towers Persona↔Skill Routing
Selection of the best sub-agent (persona) for a micro-project is deterministic and testable: a
Two-Towers matcher scores persona embeddings against skill/task embeddings and returns a ranked,
reproducible choice. Routing is a domain/application concern — no ad-hoc heuristics in adapters.

## 6. Serial Cascade & Rollback-as-a-Service
- Actions run **in cascade (serial)**, never in parallel, to protect state integrity.
- Before any environment-affecting action, the core produces a mandatory **rollback plan** and
  requests a fresh approval over IPC.
- On completion, the micro-project and its rollback state are packaged and persisted to a
  **standalone git repository** for later reuse (git-as-a-service governance).

## 7. The Rust↔Nim Boundary (the only coupling point)
- Transport: **line-delimited JSON on stdio** (duplex). The core streams events
  (`agent_state`, `fsm_transition`, `log`, `metric`, `heartbeat`, `approval_request`); the TUI sends
  commands (`user_input`, `command`, `approval_response`).
- The protocol is **versioned and schema-checked**. Neither side may assume the other's internal
  types. Changing the contract requires an ADR in `docs/adr/`.
- Rationale for a process boundary (not FFI): Tatui is a Nim `nimble` library, so the TUI is a
  distinct runtime; the stdio protocol keeps the core headless, testable, and CI-friendly.
