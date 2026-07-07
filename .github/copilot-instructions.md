# Maestro — Copilot Instructions

Maestro is a **local-first, tactical Agentic Workflow Orchestrator** — an "AI harness" that plans,
instruments, and executes small, disposable **micro-projects** (scripts, infra automation, driver
prototypes) fully offline, with no external tokens required.

- **Rust is the brain.** The orchestration core (FSM, scheduler, state, git governance, Two-Towers
  persona/skill routing, multi-model runtime) lives in Rust for predictable execution, memory
  safety, and performance.
- **Nim + Niobium is the nervous system.** The interactive TUI is a separate Nim process that
  **consumes [Niobium](https://github.com/invector-tecnologia/niobium)** (an immediate-mode,
  ratatui-inspired Nim TUI library) as a `nimble` dependency. Maestro does **not** build Niobium;
  it depends on it.

## Mental model

- The user is a **validator of plans**, not a direct programmer. Maestro reduces hallucination by
  **serializing** work and forcing spec-driven development.
- Every micro-project flows through a strict **finite state machine**:
  `Ideation → Planning → Approval → Instrumentation → Execution → Verification`.
- Actions run **in cascade (serial)**, never in parallel, to protect state integrity.
- Every agent runs the canonical cognitive cycle:
  `SENSE → OBSERVE → THINK → ACT → AUDIT → DELIVER` (see
  [ARCHITECTURE.md](../docs/Maestro_Manifesto/ARCHITECTURE.md)).

## The architectural pillars (never violate)

1. **Hexagonal core (ports & adapters).** `domain` is pure (no I/O, no provider SDKs); `application`
   orchestrates; `infrastructure` adapts external systems; `presentation` is the CLI + IPC surface.
2. **Two-process boundary.** The Rust core and the Nim/Niobium TUI are **separate processes** that
   communicate over a **line-delimited JSON protocol on stdio** — never linked via FFI. The core is
   headless and fully usable with `--no-tui`; the TUI is a thin renderer of core events.
3. **Governed execution.** No environment-affecting action runs without (a) explicit user approval
   and (b) a mandatory **rollback plan**. Completed micro-projects are packaged and persisted to a
   **standalone git repository** for later reuse.
4. **Two-Towers routing.** Persona↔skill selection is deterministic and testable; the best sub-agent
   (persona) for a micro-project is chosen by the Two-Towers matcher, not by ad-hoc heuristics.

## Hard invariants (enforced by CI and review)

- **No `unwrap()` / `expect()` / `panic!()`** in normal Rust paths; propagate with `?`. Use
  `thiserror` for domain/application errors, `anyhow` only at the CLI boundary.
- **Async safety.** Shared mutable state uses `Arc<tokio::sync::RwLock<T>>` or `Mutex`. Never
  `std::sync::Mutex` or blocking I/O inside async Tokio paths.
- **Observability.** All logging goes through `tracing`. `println!`/`dbg!`/`eprintln!` are forbidden
  in production code.
- **Architecture boundaries.** `domain/` imports no I/O or provider SDK. Cross a boundary only
  through a port (trait).
- **IPC discipline.** The Rust↔Nim contract is versioned and schema-checked; neither side may assume
  the other's internal types. The protocol is the only coupling point.
- **Niobium is a dependency.** Frontend Nim code lives under `frontend/` and only *consumes* Niobium
  widgets; it does not re-implement TUI primitives.

## Platform scope

Linux and macOS. Local-first: Ollama by default (no API key); cloud providers (OpenAI, Anthropic,
Gemini) optional. Windows is not targeted (WSL2 for development only).

## Conventions

- **Rust:** edition-current, `cargo fmt`, `cargo clippy -D warnings`. DDD layering under `src/`.
- **Nim frontend:** Nim ≥ 2.0, `--mm:orc`, formatted with `nph`, `requires "niobium >= 0.1.0"`.
- **Error handling:** `Result` everywhere; surface failures early through quality gates.
- **Spec-first:** write/adjust the task spec in `docs/Maestro_Execution_Plans/tasks/`, add a failing
  test, then implement.

## Workflow

Read [AGENTS.md](../AGENTS.md) for the required reasoning loop and Definition of Done before making
changes. Maestro is **spec-first and governance-first**: every change maps to a plan task and
respects the delivery gates in `.github/instructions/`.
