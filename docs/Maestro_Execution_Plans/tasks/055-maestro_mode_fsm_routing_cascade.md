# TASK 055: Maestro Mode — FSM orchestration, Two-Towers routing, serial cascade

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** A user demand (`user_input`), the persona catalog
  (`src/domain/models/persona.rs`), config agent bindings (`src/domain/models/config.rs`), and the
  duplex server (`src/presentation/ipc/server.rs`).
* **Context Anchors:** #skill:fsm-orchestration, #skill:two-towers-routing,
  #file:docs/Maestro_Manifesto/ARCHITECTURE.md
* **Expected Output:** Submitting a demand in Maestro Mode drives a micro-project through the six-stage
  FSM, routes personas deterministically (Two-Towers), delegates to them in a serial cascade, and
  streams `fsm_transition` / `plan_proposed` / `agent_state` / `delegation` / `deliverable` events the
  TUI already renders.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* FSM transitions are strictly sequential; illegal transitions are typed errors (`FsmError`).
* Routing is deterministic and reproducible: identical input → identical ranking; explicit
  tie-breaker (id ascending) and a documented fallback persona.
* Execution is a **serial cascade**: each persona fully completes before the next begins.
* Orchestration is pure (no I/O, no LLM) and returns an ordered signal stream the server maps to IPC.
* No `unwrap`/`expect`/`panic!` in normal paths; `thiserror`; `tracing` logging.

## 3. ACCEPTANCE CRITERIA
* AC1: `FsmStage` + `MicroProject` model the six stages with `advance`/`transition_to`; the full legal
  table passes and illegal transitions are rejected.
* AC2: `route(demand, personas)` scores all non-orchestrator candidates, ranks stably (score desc, id
  asc), selects threshold winners, and falls back to a documented default when none clear it.
* AC3: `orchestrate(demand, personas, model_for)` walks Ideation→…→Verification, proposes a plan,
  brings selected personas online, and delegates+delivers per persona serially.
* AC4: `model_for` resolves a persona's model from config `agents`, defaulting to the system model.
* AC5: The server maps orchestration signals to `fsm_transition`/`plan_proposed`/`agent_state`/
  `delegation`/`deliverable` and streams them for a `user_input` command.
* AC6: All quality gates pass with tests for FSM, routing determinism/tie-break/fallback, orchestration
  ordering, and model routing.

## 4. VALIDATION COMMANDS
* `cargo fmt --all -- --check` · `cargo clippy --all-targets -- -D warnings` · `cargo test --all-targets`
* `scripts/quality-gate.sh`

## 5. INCREMENT MAP
* INC1 (done): `domain/models/fsm.rs` — stages, transitions, `MicroProject`. Covers AC1.
* INC2 (done): `domain/models/routing.rs` — deterministic Two-Towers matcher. Covers AC2.
* INC3 (done): `application/orchestrator.rs` — Plan→Delegate→Audit→Deliver signal stream. Covers AC3.
* INC4 (done): `application/model_router.rs` — per-persona model. Covers AC4.
* INC5 (done): server `user_input` → `orchestrate_demand` mapping signals to IPC. Covers AC5.
* INC6 (done): quality gates + E2E orchestration smoke. Covers AC6.

## 5b. VALIDATION EVIDENCE
* Rust: 97 unit + 1 boundary pass; clippy clean; `scripts/quality-gate.sh` OK.
* E2E: a demand streams `start→ideation→…→verification`, routes 3 personas, cascades serially, and
  Maestro delivers at verification.

## 6. RESIDUAL RISKS
* The Approval and Execution gates are auto-advanced here; the blocking IPC approval + rollback land in
  Phase 4 (tasks 048/049).
* The lexical scorer stands in for embedding towers; an `infrastructure` embeddings port can replace
  it without changing the ranking contract.
* Personas are the built-in catalog; loading the governed persona set into routing is a later refinement.
