# AGENTS.md — Maestro agent operating contract

This file defines how automated agents (and humans) must work in this repository, including the architectural invariants.

## What Maestro is

A local-first **tactical agentic orchestrator** (an "AI harness") for disposable **micro-projects**.
Two stacks, one process boundary:

- **Rust core** — FSM, scheduler, Two-Towers routing, multi-model runtime, git/rollback governance,
  hexagonal domain. Headless; drives everything; usable with `--no-tui`.
- **Nim/Tatui TUI** — a separate process under `frontend/` that consumes
  [Tatui](https://github.com/invector-tecnologia/tatui) and renders core events over a
  line-delimited JSON stdio protocol.

## Development philosophy

**Infra-first, then spec-first.** The AI operating model (instructions, skills, agents, ADRs, plan
tasks, CI) is established before product code. Each unit of product code is driven by a task spec in
`docs/Maestro_Execution_Plans/tasks/` and a failing test before implementation.

## The reasoning loop (follow for every task)

1. **Plan** — read the relevant plan task, the ADRs in `docs/adr/`, and the manifesto docs. State the intended change and which FSM stage / architecture layer it touches. **Always suggest which Gemini model and category (e.g., Gemini 3.1 Pro - Low/High, Gemini 3.5 Flash) should be used, and wait for the user's explicit "proceed" approval before starting the implementation or editing code.**
2. **Red** — write or update a failing test for the right reason (Rust `#[cfg(test)]` unit test, or
   a Nim TUI golden snapshot via Tatui's test backend).
3. **Implement** — write the minimal code to pass. Respect the hard invariants and layer boundaries.
4. **Verify** — run the quality gates below, cheap → expensive. Fix and repeat on failure.
5. **Self-review** — re-check against the restrictions in the Component Instructions below.
6. **Record** — update the plan task's evidence and any relevant ADR.

## Quality gates

### Rust core (run in this order)

```
cargo fmt --all --check                       # 1. formatting
cargo clippy --all-targets -- -D warnings     # 2. lint (no warnings)
cargo test --all-targets                      # 3. unit + integration
scripts/quality-gate.sh                        # 4. aggregate project gate
```

### Nim / Tatui TUI (run in this order)

```
scripts/install-tatui.sh                    # 1. resolve the TUI dependency (pinned tatui commit; not on the nimble registry yet)
nph --check frontend                          # 2. formatting
nimble test                                   # 3. TUI tests (Tatui test backend, no TTY)
```

## Definition of Done

- All applicable quality gates pass (Rust and/or Nim, depending on scope).
- Change maps to exactly one plan task; acceptance criteria validated with evidence.
- Architecture boundaries respected; the Rust↔Nim coupling stays only in the IPC protocol.
- No `unwrap`/`expect`/`panic!` in normal Rust paths; all logging via `tracing`.
- Behavior-changing work documents its rollback and risk notes.

## Toolchain

- Rust: stable toolchain, `cargo fmt` + `cargo clippy`, Tokio async runtime.
- Nim ≥ 2.0.0 (`--mm:orc`), formatter `nph`, TUI via Tatui pinned to the exact v0.1.0 commit (`scripts/install-tatui.sh`).
- Never bypass gates (no skipping clippy/fmt, no `--no-verify`).


# Core Architectural Invariants (Migrated)

# Maestro — Copilot Instructions

Maestro is a **local-first, tactical Agentic Workflow Orchestrator** — an "AI harness" that plans,
instruments, and executes small, disposable **micro-projects** (scripts, infra automation, driver
prototypes) fully offline, with no external tokens required.

- **Rust is the brain.** The orchestration core (FSM, scheduler, state, git governance, Two-Towers
  persona/skill routing, multi-model runtime) lives in Rust for predictable execution, memory
  safety, and performance.
- **Nim + Tatui is the nervous system.** The interactive TUI is a separate Nim process that
  **consumes [Tatui](https://github.com/invector-tecnologia/tatui)** (an immediate-mode,
  ratatui-inspired Nim TUI library) as a `nimble` dependency. Maestro does **not** build Tatui;
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
2. **Two-process boundary.** The Rust core and the Nim/Tatui TUI are **separate processes** that
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
- **Tatui is a dependency.** Frontend Nim code lives under `frontend/` and only *consumes* Tatui
  widgets; it does not re-implement TUI primitives.

## Platform scope

Linux and macOS. Local-first: Ollama by default (no API key); cloud providers (OpenAI, Anthropic,
Gemini) optional. Windows is not targeted (WSL2 for development only).

## Conventions

- **Rust:** edition-current, `cargo fmt`, `cargo clippy -D warnings`. DDD layering under `src/`.
- **Nim frontend:** Nim ≥ 2.0, `--mm:orc`, formatted with `nph`, `requires "tatui >= 0.1.0"`.
- **Error handling:** `Result` everywhere; surface failures early through quality gates.
- **Spec-first:** write/adjust the task spec in `docs/Maestro_Execution_Plans/tasks/`, add a failing
  test, then implement.

## Workflow

Read [AGENTS.md](../AGENTS.md) for the required reasoning loop and Definition of Done before making
changes. Maestro is **spec-first and governance-first**: every change maps to a plan task and
respects the delivery gates in `.github/instructions/`.


# Component Instructions



## Fsm Orchestration

---
applyTo: "src/**/*.rs"
description: "Use when implementing or reviewing the micro-project finite state machine: stage transitions, approval gates, and instrumentation in Maestro's Rust core."
---

# FSM Orchestration Rules

## The canonical machine
Every micro-project advances through exactly these stages, in order:

`Ideation → Planning → Approval → Instrumentation → Execution → Verification`

## Transition rules
- Model stages and transitions as explicit domain types; never encode stage as a bare string.
- Only legal transitions are allowed. Reject illegal transitions with a typed error (`thiserror`) —
  never silently no-op.
- Every transition emits a `tracing` event carrying the micro-project id, from-stage, and to-stage.
- The FSM lives in `application/`; it is driven by pure `domain/` state and never performs I/O
  directly (delegate through ports).

## Gated stages
- **Approval** and the pre-**Execution** rollback gate require an explicit user decision relayed over
  the IPC boundary. The core blocks until an `approval_response` arrives; it never assumes consent.
- **Instrumentation** generates personas and injects their system prompt, skills, RAG context, and
  `.spec`/`.json` files. Instrumented artifacts are recorded so the run is reproducible.

## Verification
- Cover the full legal transition table with unit tests, plus at least one rejected-illegal-transition
  test per stage.
- Any new stage, gate, or transition semantics requires an ADR in `docs/adr/`.


## Github Delivery Gates

---
applyTo: "**/*"
description: "Use when preparing pull requests, reviewing merge readiness, validating CI evidence, and enforcing delivery governance in GitHub workflows."
---

# GitHub Delivery Gates

## Pull Request Requirements
1. Every PR must link a plan task or justify why no plan update is required.
2. Validation evidence must include executed commands and relevant outcomes.
3. Risks and rollback notes must be explicit for behavior-changing work.

## Merge Readiness Rules
- Required checklist items must be checked, not only present in the PR body.
- CI must pass all required quality gates before merge.
- Any known deviation from manifesto or conventions must be documented and approved.

## Minimum Evidence
- Local test evidence for impacted scope.
- Architecture boundary compliance confirmation.
- Acceptance criteria confirmation tied to specification.


## Kv Cache

---
applyTo: "src/**/*.rs"
description: "Use when implementing caching, prompt reuse, token reduction, response memoization, provider request deduplication, or KV cache related runtime behavior."
---

# KV Cache Policy

## Design Rules
1. Cache keys must be deterministic and include model/provider identity.
2. Cache entries must carry explicit freshness policy (TTL or invalidation signal).
3. Fail open on cache read errors unless user safety or correctness requires fail closed.
4. Never let cache bypass authorization or tenant boundaries.

## Safety Rules
- Avoid stale-cache hallucination by tying cache scope to prompt and context hash.
- Log cache hit, miss, and invalidation events with `tracing` for auditability.
- Provide a clear bypass path for debugging and incident response.

## Verification
- Add tests for hit/miss, invalidation, and stale-read prevention.
- Document measurable impact (latency, token reduction, cost proxy) when cache policy changes.


## Maestro Adr

---
applyTo: "docs/adr/**"
description: "Architecture Decision Record format (MADR-lite) for Maestro."
---

# ADR format — Maestro (MADR-lite)

## File
- Location: `docs/adr/`. Name: `NNNN-kebab-title.md` (zero-padded, sequential).
- Status is one of: `proposed`, `accepted`, `superseded by NNNN`, `deprecated`.

## Required sections
```
# NNNN. <Title>

- Status: <proposed | accepted | ...>
- Date: <YYYY-MM-DD>
- Deciders: <who>

## Context
Why this decision is needed; the forces and constraints.

## Decision
The choice made, stated in active voice.

## Consequences
Positive, negative, and any **testable invariant** introduced.

## Alternatives considered
Each option with a one-line reason it was not chosen.
```

## Rules
- One decision per ADR. Do not edit an accepted ADR's meaning; supersede it with a new one.
- Link the relevant plan task in `docs/Maestro_Execution_Plans/tasks/`.
- Any change to the Rust↔Nim IPC contract, the FSM stages, or the governance gates requires an ADR.


## Nim Frontend

---
applyTo: "frontend/**/*.nim"
description: "Use when writing Maestro's Nim TUI that consumes Tatui: widget composition, constraint layout, the stdio protocol client, and headless snapshot tests."
---

# Nim Frontend Rules (Tatui consumer)

Maestro's TUI is a **consumer** of [Tatui](https://github.com/invector-tecnologia/tatui), never a
re-implementation of it. Frontend Nim code lives under `frontend/`.

## Dependency & toolchain
- Declare a bare `requires "tatui"` in `frontend/*.nimble` and install it via `scripts/install-tatui.sh` (the exact pinned commit behind tatui v0.1.2); it is not on the nimble registry yet. Nim ≥ 2.0, `--mm:orc`, formatted with `nph`.
- Use only Tatui's public API: `newTerminal(newAnsiBackend())`, `term.setup()` /
  `defer term.restore()`, `term.draw proc(f: var Frame) = ...`, `f.renderWidget(w, rect)`.

## Composition
- Compose only shipped widgets: `Block`, `Paragraph`, `List`, `Table`, `Tabs`, `Clear`, `Gauge`,
  `Sparkline`, `BarChart`, `Scrollbar`, `Chart`. Do not build custom cell/diff/backend logic.
- Lay out with constraints (`length`, `percentage`, `ratio`, `min`, `max`, `fill`) and
  `f.area.split(...)`; never hard-code absolute coordinates.
- Keep the render function a **pure function of state**: read the latest core snapshot, draw the
  frame, forward input. No business logic in the TUI.

## Boundary discipline
- The TUI talks to the Rust core **only** over the line-delimited JSON stdio protocol
  (`protocol.nim`). It never embeds orchestration logic or assumes core-internal types.
- Terminal state (raw mode, alt screen) must be restored via `defer` even on error.

## Testing
- Assert rendering with Tatui's **test backend** (renders a `Buffer` to text, no TTY) against
  golden snapshots under `frontend/tests/`. Changing a golden file must be intentional.


## Nim Upstream

---
applyTo: "frontend/**/*.nim"
description: "Reference the upstream Nim language/std (github.com/nim-lang/Nim) as ground-truth for std modules, --mm:orc semantics, and syntax when local sources are insufficient."
---

# Nim Upstream Reference

- Authoritative source: https://github.com/nim-lang/Nim (`git@github.com:nim-lang/Nim.git`).
- Confirm `std/*` APIs, pragmas, and `--mm:orc` semantics upstream before relying on memory; cite
  exact permalinks.
- Target Nim ≥ 2.0 with `--mm:orc`; ignore pre-2.0 or GC-specific upstream guidance.
- Local doctrine wins: the `tatui-frontend` skill and `nim-frontend.instructions.md` take
  precedence. Keep the TUI a thin, pure renderer of core state.


## Rag Gate

---
applyTo: "src/**/*.rs"
description: "Use when implementing or reviewing RAG ingestion, retrieval, reranking, evaluation, dataset versioning, embeddings, citations, or grounding quality in Maestro."
---

# RAG Governance Gate

## Required Flow
1. Preserve separation between domain ports, application orchestration, and infrastructure adapters.
2. Keep lexical fallback available when embeddings are absent.
3. Ensure query outputs include traceable citations or provenance.
4. Keep evaluation datasets versioned under docs and reports persisted for later comparison.

## Change Rules
- Any RAG logic update must include at least one test covering regression risk.
- If scoring or ranking logic changes, compare baseline vs enhanced behavior.
- Reject hidden magic constants; document thresholds in code or config.

## Evidence Expectations
- Include command evidence for local validation (for example: `cargo test`).
- When possible, include a short before/after metric delta for relevance or hit-rate.


## Rollback Cascade

---
applyTo: "src/**/*.rs"
description: "Use when implementing serial cascade execution, rollback-as-a-service, or git-standalone persistence of micro-projects in Maestro's Rust core."
---

# Cascade & Rollback Rules

## Serial cascade
- Coordinated actions run **strictly in cascade (serial)** — never `tokio::spawn` parallel
  environment-affecting steps. Concurrency is allowed only for read-only observation.
- A failed step halts the cascade; downstream steps do not run.

## Rollback-as-a-service
- Before **any** environment-affecting action, produce a concrete rollback plan and request a fresh
  user approval over IPC. No rollback plan ⇒ no execution.
- Rollback steps are the inverse of forward steps and are themselves recorded and testable.
- On failure mid-cascade, surface the rollback plan for the steps already applied.

## Git-standalone persistence
- On successful completion, package the micro-project artifacts **and** the rollback state, then
  persist them to a standalone git repository for later reuse.
- Persistence goes through a port; `domain/` never touches git directly.

## Verification
- Test: cascade halts on first failure; rollback plan is required before execution; packaged output
  contains both artifacts and rollback state.
- All state transitions and approvals are logged via `tracing`.


## Rust Companion

---
applyTo: "src/**/*.rs"
description: "Use when editing Rust code in Maestro to enforce architecture boundaries, error handling, Tokio concurrency, and testing conventions."
---

# Rust Companion Rules

## Architecture
- Keep domain pure and free from direct I/O and provider SDK details.
- Keep orchestration in application; external adapters in infrastructure; parsing and UX in presentation.

## Error and Concurrency
- Never introduce `unwrap()`, `expect()`, or `panic!()` in normal paths.
- Use `thiserror` in domain/application error types and propagate with `?`.
- Use `Arc<tokio::sync::RwLock<T>>` for shared mutable async state by default.

## Quality Gate
- Add or update tests when behavior changes.
- Prefer focused unit tests close to changed module.
- Keep public API changes explicit and documented in PR notes.


## Rust Upstream

---
applyTo: "src/**/*.rs"
description: "Reference the upstream Rust language/std (github.com/rust-lang/rust) as ground-truth for std APIs, editions, and language semantics when local sources are insufficient."
---

# Rust Upstream Reference

- Authoritative source: https://github.com/rust-lang/rust (`git@github.com:rust-lang/rust.git`).
- Confirm `std`/`core`/`alloc` API shapes, stabilization, and edition semantics upstream before
  relying on memory; cite exact permalinks.
- Stable toolchain only — never adopt nightly/unstable APIs into Maestro.
- Local doctrine wins: `docs/Maestro_Manifesto/CONVENTIONS.md`, the `rust` skill, and
  `rust-companion.instructions.md` take precedence over upstream examples.


## Spec Driven

---
applyTo: "**/*"
description: "Use when planning or implementing features with specification-first delivery, acceptance criteria, milestone tracking, or execution plans in docs/Maestro_Execution_Plans."
---

# Spec-Driven Delivery

## Required Sequence
1. Define or update the execution plan task document before major code changes.
2. Encode acceptance criteria that can be validated by tests or scripted checks.
3. Implement in small increments mapped to the spec.
4. Record validation evidence and residual risks.

## Review Rules
- Reject features without explicit acceptance criteria.
- Reject merges when implementation diverges from documented scope without rationale.
- Prefer small PRs linked to one plan task whenever possible.

## Documentation Targets
- Task specs: `docs/Maestro_Execution_Plans/tasks/`
- Product doctrine: `docs/Maestro_Manifesto/`


## Tatui Upstream

---
applyTo: "frontend/**/*.nim"
description: "Reference the Tatui TUI library (github.com/invector-tecnologia/tatui) as ground-truth for widgets, constraint layout, the event decoder, the tick loop, and the test backend."
---

# Tatui Upstream Reference

- Authoritative source: https://github.com/invector-tecnologia/tatui
  (`git@github.com:invector-tecnologia/tatui.git`). Pinned via `scripts/install-tatui.sh`
  (commit `493d9fc0` = v0.1.2).
- Compose panels only from shipped widgets: `Block`, `Paragraph`, `List`, `Table`, `Tabs`, `Clear`,
  `Gauge`, `Sparkline`, `BarChart`, `Scrollbar`, `Chart`. Do not reimplement primitives.
- Layout via `f.area.split(...)` constraints (`length`/`percentage`/`ratio`/`min`/`max`/`fill`).
- Study `examples/` and `src/tatui/{core,layout,backend,terminal,event,widgets}` for real usage.
- Test headlessly with `newTestBackend(w, h)`; snapshot the rendered text — no TTY required.
- Local doctrine wins: the `tatui-frontend` skill and `nim-frontend.instructions.md`.


## Two Towers Routing

---
applyTo: "src/**/*.rs"
description: "Use when implementing or reviewing Two-Towers persona↔skill routing: deterministic matcher, scoring, and reproducible selection in Maestro's Rust core."
---

# Two-Towers Routing Rules

## Purpose
Select the best sub-agent (persona) for a micro-project by scoring a **persona tower** against a
**skill/task tower**. Selection must be deterministic and testable — never an ad-hoc heuristic
scattered across adapters.

## Design rules
- Keep the matcher in `domain/`/`application/`; embedding *providers* are ports implemented in
  `infrastructure/`.
- Given identical inputs, the ranked output must be identical (stable sort with an explicit
  tie-breaker; no reliance on hash-map iteration order).
- Return a ranked list with scores, not just the top pick, so decisions are auditable.
- Log the chosen persona, runner-up, and score margin via `tracing`.

## Safety rules
- Fall back to a documented default persona when no candidate clears the minimum score threshold;
  never route to an empty/undefined persona.
- Thresholds and weights are named constants or config — no magic numbers inline.

## Verification
- Unit-test determinism (same input → same ranking), tie-breaking, and the fallback path.
- When scoring or weighting changes, compare baseline vs new ranking on a fixed fixture set.
