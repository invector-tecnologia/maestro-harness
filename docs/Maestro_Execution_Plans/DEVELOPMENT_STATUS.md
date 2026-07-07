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
| 013 | TUI creation wizards | ❌ RETIRED — folded into Config Mode (ADR 0002) |
| 017–032 | 0.2.0 Onboarding / interview UX | ❌ RETIRED — interview mode cut (ADR 0002) |
| 043–045 | LLM-driven interview engine | ❌ RETIRED — interview mode cut (ADR 0002) |
| 035 | Three-Mode Workspace (was Core/Interview modes) | 🔁 REWRITTEN — Config·Maestro·Product (ADR 0002) |
| 040, 041 | Mode naming + Nim module split | 🔁 REWRITTEN for the three modes |
| 033–034, 036–039, 042 | Governance / orchestration / multi-model harness | ⏳ kept, not started |
| 046–050 | Micro-project engine (FSM→cascade→rollback→git) | ⏳ kept, not started |
| W1–W6 | **Workspace pivot** (init CLI, IPC v2, Config/Maestro/Product) | 🚧 W1 (053) + W2 (054) done; W3–W6 next |

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

> **Direction change (2026-07-07):** Maestro pivots to a **three-mode Workspace** — Config Mode,
> Maestro Mode, Product Mode — and **removes interview/onboarding mode entirely**. See
> [ADR 0002](../adr/0002-three-mode-workspace-and-interview-removal.md) and
> [ADR 0003](../adr/0003-ipc-v2-mode-scoped-protocol.md). The seven-phase execution plan lives in
> the session plan; the workspace tasks are labelled `W1–W6` in the ledger above.

1. **W1 — Wire the boundary + shell (Phase 1):** IPC v1→v2 (ADR 0003), long-running `maestro run`/
   `tui`, Nim tick loop + `Tabs` Workspace shell, and the plain-CLI `maestro init` bootstrap.
2. **W2 — Config Mode (Phase 2):** governance CRUD + archive (config.yml + personas/skills/scopes,
   defaults and customs) with a unified markdown parser (absorbs 037/038; Architect Mode retired).
3. **W3 — Maestro Mode (Phase 3):** FSM engine (046) + meta-orchestrator (039) + Two-Towers (047) +
   serial cascade (048) + `ModelRouter` (042). Agents: `Cascade Runner`, `Persona Instrumenter`.
4. **W4 — Governed execution (Phase 4):** approval gates, rollback (049), git persistence (050),
   safety harness (033/034). Agents: `Cascade Runner`, `Rollback Planner`.
5. **W5 — Product Mode (Phase 5):** release model + demo runner (live artifact stream) + changelog.
6. **W6 — Providers / accessibility / packaging / RC (Phase 6).**

## 7. Known gaps / debt

- `maestro run` and `maestro tui` are not wired to the IPC boundary yet (stubs / planned).
- Only the **Ollama** provider adapter exists; OpenAI/Anthropic/Gemini are planned (registry supports the `ollama` kind only).
- Agent cognitive cycles are concurrent (`JoinSet`, read-only); the **serial cascade** rule applies to
  environment-affecting execution (task 048), enforced separately — do not "fix" the runtime to be serial.
- **Niobium** is not on the nimble registry: `frontend/*.nimble` uses bare `requires "niobium"`; install the
  exact pinned commit with `scripts/install-niobium.sh` (commit `0051e112…` = tag v0.1.0).
- RC docs: refresh test-count evidence at each release gate (016 = 0.1.0, 031 = 0.2.0).
