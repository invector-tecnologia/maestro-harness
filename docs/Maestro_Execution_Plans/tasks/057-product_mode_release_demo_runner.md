# TASK 057: Product Mode — release listing + live demo runner

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** Persisted releases under `maestro/releases/` (TASK 056 / `persistence.rs`), the duplex
  server (`src/presentation/ipc/server.rs`), and the v2 Product messages (`src/presentation/ipc/mod.rs`).
* **Context Anchors:** #file:docs/adr/0002-three-mode-workspace-and-interview-removal.md,
  #file:docs/User_Manual/COMMANDS_AND_PANELS.md
* **Expected Output:** Product Mode lists shipped releases and, for a chosen release, runs its built
  artifact live, streaming the output alongside the release notes.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* The demo runs the release's own artifact (`demo.sh`) as a subprocess; a missing artifact is reported,
  not fatal.
* Output is streamed line by line as `demo_output` and terminated by `demo_exited`.
* Release listing reads from disk manifests — no in-memory-only state.
* No `unwrap`/`expect`/`panic!` in normal paths; `thiserror`; `tracing` logging.

## 3. ACCEPTANCE CRITERIA
* AC1: `persist_release` writes a runnable `demo.sh` artifact echoing the release + deliverables.
* AC2: `list_releases(root)` parses each release manifest into `ReleaseRecord`s (id, version,
  changelog, created_at), sorted by id.
* AC3: `run_demo(root, version, on_output)` locates the release by version, streams stdout/stderr
  lines, and returns the exit code (`127` when no artifact exists, error when the release is missing).
* AC4: The server maps `list_releases → release_list` and `run_demo → demo_output* + demo_exited`.
* AC5: The Nim Product panel lists releases with notes and renders the live demo output; Enter on a
  selected release sends `run_demo`, and entering the mode sends `list_releases`.
* AC6: All quality gates pass with tests for persistence listing, the demo runner, and the server flow.

## 4. VALIDATION COMMANDS
* `cargo fmt --all -- --check` · `cargo clippy --all-targets -- -D warnings` · `cargo test --all-targets`
* `scripts/quality-gate.sh` · `cd frontend && nimble test`

## 5. INCREMENT MAP
* INC1 (done): `persist_release` writes `demo.sh`; `list_releases` reader in `persistence.rs`. AC1, AC2.
* INC2 (done): `application/demo_runner.rs` — subprocess streaming runner. AC3.
* INC3 (done): server `ListReleases`/`RunDemo` handlers. AC4.
* INC4 (done): Nim Product panel + `run_demo`/`list_releases` wiring (already present from W1). AC5.
* INC5 (done): quality gates + E2E Product Mode smoke. AC6.

## 5b. VALIDATION EVIDENCE
* Rust: 110 unit + 1 boundary pass; clippy clean; `scripts/quality-gate.sh` OK. Nim: 20 tests.
* E2E: a persisted release lists as `0.1.1`; `run_demo` streams the artifact's stdout and exits `0`.

## 6. RESIDUAL RISKS
* stdout is fully drained before stderr (adequate for the short demo artifact); truly interleaved
  streaming would need a select/thread and is deferred.
* The demo artifact is a generated `demo.sh`; when the cascade produces real build outputs, the runner
  can target the real entrypoint without protocol changes.
