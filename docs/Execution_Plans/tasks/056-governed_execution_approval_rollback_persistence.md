# TASK 056: Governed execution — approval gates, rollback, git-standalone persistence

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** The meta-orchestrator (`src/application/orchestrator.rs`), the FSM
  (`src/domain/models/fsm.rs`), the duplex server (`src/presentation/ipc/server.rs`), and the v2
  approval messages (`src/presentation/ipc/mod.rs`).
* **Context Anchors:** #skill:fsm-orchestration, #skill:rollback-cascade,
  #file:docs/adr/0002-three-mode-workspace-and-interview-removal.md
* **Expected Output:** A demand's execution is **governed**: it blocks at the Approval and Execution
  gates until the user responds, rolls back on rejection, and — on success — packages the
  micro-project into a standalone git release that feeds Product Mode.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* No environment-affecting step runs without (a) explicit user approval and (b) a ready rollback plan.
* Runs are single-flight: a new demand is refused while one awaits approval.
* Rollback applies inverse actions in reverse order.
* Persistence is best-effort git (a missing `git` is not fatal); the release manifest is always written.
* No `unwrap`/`expect`/`panic!` in normal paths; `thiserror`; `tracing` (stderr) logging.

## 3. ACCEPTANCE CRITERIA
* AC1: A [`Session`] blocks at the plan gate emitting an `approval_request`; no delegation occurs
  before approval.
* AC2: Approving the plan advances to Instrumentation, builds a rollback plan, and blocks at the
  execution gate; approving execution runs the serial cascade and delivers at Verification.
* AC3: Rejecting the plan aborts with no side effects; rejecting execution emits the rollback inverse
  actions in reverse order.
* AC4: The server maps approval signals to `approval_request` and resumes on `approval_response`,
  refusing a second demand while a run is pending.
* AC5: On completion the micro-project is persisted to `maestro/releases/<id>/manifest.md` with an
  incrementing version and best-effort git commit.
* AC6: The TUI shows the pending approval and sends `approval_response` on `y`/`n` in Maestro Mode.
* AC7: All quality gates pass with tests for gates, rollback ordering, persistence, and the server flow.

## 4. VALIDATION COMMANDS
* `cargo fmt --all -- --check` · `cargo clippy --all-targets -- -D warnings` · `cargo test --all-targets`
* `scripts/quality-gate.sh` · `cd frontend && nimble test`

## 5. INCREMENT MAP
* INC1 (done): `domain/models/rollback.rs` — `CascadeStep`/`RollbackPlan` inverse-in-reverse. AC3.
* INC2 (done): `Session` gated state machine in `application/orchestrator.rs`. AC1–AC3.
* INC3 (done): `application/persistence.rs` — git-standalone release packaging. AC5.
* INC4 (done): server `PendingRun` + `begin_run`/`resume_run`; single-flight. AC4.
* INC5 (done): Nim approval state + `y`/`n` response in the Maestro panel. AC6.
* INC6 (done): quality gates + E2E gated-flow smoke. AC7.

## 5b. VALIDATION EVIDENCE
* Rust: 106 unit + 1 boundary pass; clippy clean; `scripts/quality-gate.sh` OK. Nim: 20 tests.
* E2E: demand blocks at plan and execution gates, cascades on approval, delivers at Verification, and
  persists `maestro/releases/r001/manifest.md` (version 0.1.1).

## 6. RESIDUAL RISKS
* The AI safety harness (`src/infrastructure/harness`) is still a stub beyond the cascade step-limit
  guard; sandboxing real environment actions is future work (task 033 follow-up).
* Rollback inverse actions are symbolic in this deterministic slice; real environment inverses land
  with the cascade executor doing real work.
