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

**Phases 0–6 COMPLETE — the three-mode Workspace (RC 0.3.0).**
Gates: `cargo fmt/clippy/test` = **116 unit + 1 boundary** pass; `nimble test` = **21** pass;
`scripts/quality-gate.sh` = OK. See [RELEASE_CANDIDATE_0.3.0.md](RELEASE_CANDIDATE_0.3.0.md).

Implemented modules:
- `src/domain/models/` — `agent_id`, `message`, `governance` (kinds/origin/immutability), `config`,
  `persona`, `fsm` (six-stage lifecycle), `routing` (Two-Towers), `rollback`.
- `src/domain/ports/` — `llm_provider` (probe/complete), `role` (observe→think→act).
- `src/application/` — `agent_runtime`, `agent_observability` (`RuntimeEvent`), `persona_agent`,
  `readiness` (SENSE), `governance` (CRUD + archive), `orchestrator` (`Session` gated FSM + Two-Towers
  + serial cascade), `model_router`, `persistence` (git-standalone releases), `demo_runner`, `error`.
- `src/infrastructure/` — `bus/broadcast_bus`, `llm/ollama`, `llm/openai`, `llm/registry`, `config`.
- `src/presentation/` — `cli` (version/validate-config/list-agents/doctor/scaffold-markdown/init-config/
  **init**/**run**/**tui**, `--no-tui`), `ipc` (v2 `CoreEvent`/`TuiCommand` + `server`).
- `frontend/` — `app.nim` (tick loop), `protocol.nim` (v2), `workspace.nim`, `theme.nim`,
  `panels/{config,maestro,product}.nim`; tests in `frontend/tests/`.
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
| W1–W6 | **Workspace pivot** (init CLI, IPC v2, Config/Maestro/Product) | ✅ W1–W6 (053–058) done — RC 0.3.0 |

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

## 6. Status & next steps

> **Direction change (2026-07-07):** Maestro pivoted to a **three-mode Workspace** — Config Mode,
> Maestro Mode, Product Mode — and **removed interview/onboarding mode entirely** (ADR
> [0002](../adr/0002-three-mode-workspace-and-interview-removal.md) /
> [0003](../adr/0003-ipc-v2-mode-scoped-protocol.md)).

**W1–W6 (tasks 053–058) are COMPLETE — RC 0.3.0.** The Workspace runs end-to-end: `maestro init`
→ Maestro Mode demand → gated FSM orchestration → persisted release → Product Mode live demo.

Recommended follow-ups (not yet started):
1. Wire the cascade to call the provider registry for real LLM-driven work (deliverables today are
   deterministic placeholders).
2. Native **Anthropic** and **Gemini** adapters (OpenAI-compatible only today).
3. Real environment rollback inverses + a fuller AI safety harness (`src/infrastructure/harness`).
4. Load the governed persona set into Two-Towers routing (routing uses the built-in catalog today).

## 7. Known gaps / debt

- Cascade deliverables are deterministic placeholders — not yet real LLM output through the registry.
- Provider adapters: **Ollama** (default) + **OpenAI-compatible** (`openai`, key via `OPENAI_API_KEY`);
  native Anthropic/Gemini adapters are future.
- Agent cognitive cycles are concurrent (`JoinSet`, read-only); the **serial cascade** rule applies to
  environment-affecting execution (enforced by the orchestrator `Session`) — do not "fix" the runtime.
- **Niobium** is not on the nimble registry: `frontend/*.nimble` uses bare `requires "niobium"`; install
  the exact pinned commit with `scripts/install-niobium.sh` (commit `0051e112…` = tag v0.1.0).
- `nph` formatting check is advisory where `nph` is unavailable in the environment.
- RC docs: refresh test-count evidence at each release gate (016 = 0.1.0, 058 = 0.3.0).
