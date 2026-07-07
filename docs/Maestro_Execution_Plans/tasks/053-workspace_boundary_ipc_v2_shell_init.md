# TASK 053: Workspace Boundary — IPC v2, `maestro run/tui` wiring, Niobium shell, `maestro init`

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** Existing v1 IPC (`src/presentation/ipc/mod.rs`), the one-shot Nim shell
  (`frontend/src/app.nim`, `frontend/src/protocol.nim`, `frontend/src/panels/dashboard.nim`), the
  governance scaffold + config helpers (`src/presentation/cli/mod.rs`), and the runtime narration bus
  (`src/application/agent_observability.rs`, `src/infrastructure/bus/broadcast_bus.rs`).
* **Context Anchors:** #file:docs/adr/0002-three-mode-workspace-and-interview-removal.md,
  #file:docs/adr/0003-ipc-v2-mode-scoped-protocol.md, #file:docs/Maestro_Manifesto/ARCHITECTURE.md
* **Expected Output:** A live two-process Workspace. The Rust core runs a duplex IPC server
  (`maestro run` / `maestro tui`) speaking protocol **v2**; the Nim/Niobium TUI renders a three-tab
  Workspace (Config · Maestro · Product) driven by a real tick loop and stdin frames; and
  `maestro init [<name>]` performs a plain-CLI, non-LLM bootstrap that scaffolds defaults and opens
  the Workspace on Maestro Mode.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* The stdio JSON protocol is the ONLY Rust↔Nim coupling; neither side assumes the other's types.
* Protocol version is pinned at `2`; frames with `v != 2` and unknown `kind`s are rejected safely.
* No interview/onboarding flow anywhere. `maestro init` makes NO LLM calls — it is a plain terminal
  questionnaire over stdin.
* The Workspace exposes exactly three modes — Config, Maestro, Product — via a Niobium `Tabs` header.
* The Nim draw is a pure function of the latest `AppState`; no orchestration logic lives in Nim.
* Rust production paths use no `unwrap()`/`expect()`/`panic!()`; `thiserror` in domain/application,
  `anyhow` only at the CLI boundary; all logging via `tracing`.
* Shared async state uses `Arc<tokio::sync::RwLock<T>>`; no `std::sync::Mutex`/blocking I/O in async.

## 3. ACCEPTANCE CRITERIA
* AC1: `PROTOCOL_VERSION` is `2`. `CoreEvent` gains `mode_changed`, and `TuiCommand` gains
  `switch_mode`; both carry `mode ∈ {config, maestro, product}`. All v1 variants are retained.
* AC2: Reserved v2 mode-scoped variants exist and round-trip: Config (`config_tree`/`config_entry`/
  `config_validation`/`config_saved`; `config_list`/`config_open`/`config_edit`/`config_create`/
  `config_archive`/`config_validate`/`config_save`), Maestro (`plan_proposed`/`delegation`/
  `deliverable`), Product (`release_list`/`demo_output`/`demo_exited`; `list_releases`/`run_demo`/
  `stop_demo`). Every variant round-trips through `encode`/`decode` on both Rust and Nim sides.
* AC3: `maestro run` (headless) and `maestro tui` launch a duplex loop that reads `TuiCommand` from
  stdin and writes `CoreEvent` to stdout, bridging `BroadcastBus<RuntimeEvent>` narration to
  `CoreEvent`s. `--no-tui` keeps the core usable without the frontend process.
* AC4: The Nim shell runs an event loop that (a) polls Niobium keyboard events and (b) reads stdin
  frames non-blocking, updates `AppState`, and redraws. Global keys switch tabs and quit.
* AC5: `switch_mode`/`mode_changed` change the active tab and the core echoes the active mode.
* AC6: `frontend/src/protocol.nim` mirrors the v2 schema with typed encode/decode and rejects `v!=2`
  and unknown `kind`s. The Nim shell is split into `workspace.nim` + `panels/{config,maestro,
  product}.nim` (retiring the single `dashboard.nim`).
* AC7: `maestro init [<name>]` prompts for project name (REQUIRED), primary scope (REQUIRED), type
  (optional: library/Web/Desktop/Mobile), and layout-reference image paths (optional; repeat
  "add another? <path>" until the user answers No). It then scaffolds the governance defaults + a
  starter `config.yml`, writes the collected answers into the default persona/scope files, and hands
  off to the Workspace on Maestro Mode.
* AC8: `maestro init` re-run on an existing project is idempotent/safe (does not clobber existing
  files without acknowledgement) and validates required fields (rejects empty name/scope).
* AC9: All quality gates pass with added tests covering AC1–AC8 (Rust unit + Nim test-backend
  snapshots for the three-tab shell and tab switching).

## 4. VALIDATION COMMANDS
* `cargo fmt --all -- --check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test --all-targets`
* `scripts/quality-gate.sh`
* `scripts/install-niobium.sh` then `cd frontend && nph --check . && nimble test`

## 5. INCREMENT MAP
* INC1: Bump protocol to v2 and add mode-switching + reserved mode-scoped variants in
  `src/presentation/ipc/mod.rs` with round-trip tests. Covers AC1, AC2 (Rust).
* INC2: Mirror v2 schema in `frontend/src/protocol.nim` with typed encode/decode + rejection tests.
  Covers AC2 (Nim), AC6 (protocol half).
* INC3: Implement the duplex IPC server behind `maestro run`/`maestro tui` in
  `src/presentation/cli/mod.rs` (new `run`/`tui` module), bridging the narration bus. Covers AC3.
* INC4: Rewrite `frontend/src/app.nim` into a tick loop + `AppState`; add `frontend/src/workspace.nim`
  (Tabs shell) and `panels/{config,maestro,product}.nim`; retire `dashboard.nim`. Covers AC4–AC6.
* INC5: Implement `maestro init [<name>]` questionnaire + scaffold + default-file population + Maestro
  Mode hand-off in `src/presentation/cli/mod.rs`. Covers AC7, AC8.
* INC6: Quality-gate run + evidence capture. Covers AC9.

## 5b. VALIDATION EVIDENCE (2026-07-07)
* `cargo clippy --all-targets -- -D warnings`: clean. `cargo test --all-targets`: **68 unit + 1
  boundary** pass (adds IPC v2 round-trip, duplex server greet/switch/plan/config-list/quit/malformed,
  and `init` prompt/scaffold tests).
* `scripts/quality-gate.sh`: **OK** (fmt, clippy, tests, doc-links, Nim suite).
* Nim `nimble test`: **15** pass (9 protocol incl. v2 kind validation; 6 workspace snapshots + folding).
  `nph --check` not run — `nph` is not installed in this environment (follow-up).
* End-to-end smoke: piping `config_list` / `user_input` / `command:quit` into `maestro run` yields the
  correct v2 event stream (`mode_changed`→`log`→`agent_state`×4→`config_tree`→`plan_proposed`→`log`).
* Status: INC1–INC5 done; INC6 gate captured above. AC1–AC9 satisfied (AC9 modulo `nph`).

## 6. RESIDUAL RISKS
* Non-blocking stdin reads in Nim must not busy-spin; bound the poll interval and test headlessly.
* The v1→v2 bump is a flag day: Rust core and Nim client must ship together (ADR 0003).
* `init` file population must stay comment-preserving where feasible; document any lossy rewrite.
