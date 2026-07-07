# Release Candidate 0.3.0 — Three-Mode Workspace

_Date: 2026-07-07._

## Summary
RC 0.3.0 delivers the **three-mode Workspace** pivot (ADR
[0002](../adr/0002-three-mode-workspace-and-interview-removal.md)) and retires interview/onboarding
mode. The Rust core is the headless brain; the Nim/Niobium TUI renders three modes over the v2 stdio
protocol (ADR [0003](../adr/0003-ipc-v2-mode-scoped-protocol.md)).

## Scope delivered (tasks 053–058)
- **W1 — Boundary & shell (053):** protocol v2, duplex `maestro run`, `maestro tui`, plain-CLI
  `maestro init`, three-tab Niobium Workspace with a real tick loop.
- **W2 — Config Mode (054):** governance CRUD + archive for config + personas/skills/scopes (defaults
  and customs); the Maestro persona is immutable. Absorbs the former Architect Mode.
- **W3 — Maestro Mode (055):** six-stage FSM, deterministic Two-Towers routing, serial cascade,
  per-persona model routing.
- **W4 — Governed execution (056):** blocking plan + execution approval gates, rollback-as-a-service,
  git-standalone release persistence, single-flight runs.
- **W5 — Product Mode (057):** release listing + live demo runner streaming a release's artifact.
- **W6 — Providers & accessibility (058):** OpenAI-compatible provider adapter, `MAESTRO_ASCII_ONLY`
  accessibility fallback, config template with an optional cloud provider.

## Acceptance
- [x] Interview/onboarding mode removed; Workspace exposes exactly Config · Maestro · Product.
- [x] `maestro init` is plain-CLI (no LLM); opens the Workspace on Maestro Mode.
- [x] Approval gates block until the user responds; rejection rolls back.
- [x] Completed micro-projects persist to `maestro/releases/` and appear in Product Mode.
- [x] Ollama remains the local-first default; OpenAI is optional via `OPENAI_API_KEY`.

## Test evidence
- Rust: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings` clean;
  `cargo test --all-targets` = **116 unit + 1 boundary** pass.
- Nim: `nimble test` = **21** pass (protocol + workspace + accessibility snapshots).
- Aggregate: `scripts/quality-gate.sh` = OK.
- End-to-end (via `maestro run`): demand → FSM walk → two approval gates → serial cascade →
  release `0.1.1` persisted → Product Mode lists it and streams its demo (exit 0).

## Known limitations / follow-ups
- Anthropic and Gemini native adapters are not yet shipped (OpenAI-compatible only).
- The cascade produces deterministic placeholder deliverables; wiring real LLM-driven work through the
  provider registry is the next milestone.
- Rollback inverse actions are symbolic until the cascade performs real environment actions.
- `nph` formatting check is advisory where `nph` is unavailable in the environment.
