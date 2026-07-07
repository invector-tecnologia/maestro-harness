# Maestro — Development Status & Continuation Plan

> **Audience:** future AI agent (and human) sessions. Read this first to know what
> exists, what's next, and how to proceed without re-deriving the plan.
> Complements [AGENTS.md](../../AGENTS.md) (operating contract) and
> [.github/copilot-instructions.md](../../.github/copilot-instructions.md) (invariants).

_Last updated: 2026-07-07._

## 1. What Maestro is (one paragraph)

A local-first, tactical **Agentic Workflow Orchestrator**. Two processes:
a headless **Rust core** (hexagonal DDD: `domain` → `application` → `infrastructure` /
`presentation`) and a separate **Nim/Niobium TUI** under `frontend/`. They communicate
**only** over a line-delimited JSON protocol on stdio. See
[docs/Maestro_Manifesto/ARCHITECTURE.md](../Maestro_Manifesto/ARCHITECTURE.md) and
[docs/adr/0001-rust-core-nim-niobium-tui-stdio-protocol.md](../adr/0001-rust-core-nim-niobium-tui-stdio-protocol.md).

## 2. Current state (verified green)

**Phase 0 (bootstrap) + Phase 1 (Milestone 0.1.0) are COMPLETE.**
Gates: `cargo fmt/clippy/test` = **57 unit + 1 boundary** pass; `nimble test` = **6** pass;
`scripts/quality-gate.sh` = OK.

Implemented modules:
- `src/domain/models/` — `agent_id`, `message`, `governance`, `config`, `persona`.
- `src/domain/ports/` — `llm_provider` (probe/complete), `role` (observe→think→act).
- `src/application/` — `agent_runtime` (concurrent cycle, failure isolation), `agent_observability`
  (`RuntimeEvent`), `persona_agent`, `readiness` (SENSE), `governance`, `wizard`, `error`.
- `src/infrastructure/` — `bus/broadcast_bus`, `llm/ollama`, `llm/registry`, `config` loader.
- `src/presentation/` — `cli` (version/validate-config/list-agents/doctor/scaffold-markdown/init-config, `--no-tui`),
  `ipc` (versioned `CoreEvent`/`TuiCommand` + framing).
- `frontend/` — `app.nim`, `protocol.nim` (mirrors `ipc`), `panels/dashboard.nim`; tests in `frontend/tests/`.
- Tooling — `scripts/quality-gate.sh`, `scripts/build-deb.sh`, `scripts/install-niobium.sh`.

## 3. Task ledger (docs/Maestro_Execution_Plans/tasks/)

| Range | Milestone | Status |
|---|---|---|
| 000 | Bootstrap | ✅ done |
| 001–016 | 0.1.0 Foundation + Core | ✅ done |
| 051, 052 | IPC + Nim shell (resequenced into 0.1.0) | ✅ done |
| 017–032 | 0.2.0 Onboarding + Advanced | ⏳ not started |
| 033–045 | Post-0.2.0 governance/orchestration/interview | ⏳ not started |
| 046–050 | Micro-project engine (FSM→cascade→rollback→git) | ⏳ not started |
| 041, 042 | TUI module split / multi-model harness | ⚠️ **spec files missing** — author via `Plan Author` agent first |

## 4. Key sequencing decisions (already applied)

1. **051 (IPC) + 052 (Nim shell) pulled into 0.1.0.** Clean chain `010 → 051 → 052 → 012`
   (the old circular `051 → 012` edge was removed).
2. **046 (FSM engine) promoted** to the front of Phase 3 as a parallel **Track B**
   (`046 → 048 → 049 → 050`), run alongside **Track A** (governance/orchestration
   `033/034 → 035 → 036/037/038 → 039/040` + interview `043/044/045`). `047` (Two-Towers)
   is the sync point (needs `038`).

## 5. How to continue (per-task loop — from AGENTS.md)

1. **Plan** — read the task file in `docs/Maestro_Execution_Plans/tasks/`, relevant ADRs, and the
   matching `.github/instructions/`.
2. **Red** — write a failing test (Rust `#[cfg(test)]`, or Nim golden snapshot via Niobium's test backend).
3. **Implement** — minimal code; respect the invariants (no `unwrap`/`expect`/`panic!`;
   `Arc<tokio::sync::RwLock>`; `tracing` only; `domain` imports nothing outward).
4. **Verify** — `cargo fmt --all --check` → `cargo clippy --all-targets -- -D warnings` →
   `cargo test --all-targets` → `scripts/quality-gate.sh`. Nim: `scripts/install-niobium.sh` then
   `nimble test` in `frontend/`.
5. **Record** — update the task's evidence section.

## 6. Recommended next steps (in order)

1. **Author specs 041 + 042** (missing) with the `Plan Author` agent.
2. **Wire the process boundary end-to-end**: implement `maestro run`/`tui` so the core streams
   `CoreEvent`s to the `frontend/` process and consumes `TuiCommand`s (uses existing `ipc` + `protocol.nim`).
3. **Start Phase 3 Track B (micro-project engine)** — it is the product's identity and is unblocked
   now that IPC exists: 046 FSM → 048 cascade → 049 rollback → 050 git-persistence
   (agents: `Cascade Runner`, `Rollback Planner`). Use `fsm-orchestration` / `rollback-cascade` instructions.
4. **Or Phase 2 (0.2.0 onboarding, 017–032)** if breadth-first product coverage is preferred.

## 7. Known gaps / debt

- `maestro run` and `maestro tui` are not wired to the IPC boundary yet (stubs / planned).
- Only the **Ollama** provider adapter exists; OpenAI/Anthropic/Gemini are planned (registry supports the `ollama` kind only).
- Agent cognitive cycles are concurrent (`JoinSet`, read-only); the **serial cascade** rule applies to
  environment-affecting execution (task 048), enforced separately — do not "fix" the runtime to be serial.
- **Niobium** is not on the nimble registry: `frontend/*.nimble` uses bare `requires "niobium"`; install the
  exact pinned commit with `scripts/install-niobium.sh` (commit `0051e112…` = tag v0.1.0).
- RC docs: refresh test-count evidence at each release gate (016 = 0.1.0, 031 = 0.2.0).
