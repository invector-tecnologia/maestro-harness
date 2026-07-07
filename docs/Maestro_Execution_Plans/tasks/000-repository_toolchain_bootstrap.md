# TASK 000: Repository & Toolchain Bootstrap

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** Empty repository (docs + harness only, no code).
* **Context Anchors:** #file:docs/Maestro_Manifesto/ARCHITECTURE.md, #file:docs/adr/0001-rust-core-nim-tatui-stdio-protocol.md, #file:.github/instructions/rust-companion.instructions.md
* **Expected Output:** A compiling Rust `maestro` crate and a Nim/Tatui `frontend/` skeleton, both green in CI.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* `src/` mirrors the hexagonal topology; `domain/` imports nothing from `infrastructure`/`presentation`.
* No `unwrap`/`expect`/`panic!` in normal paths; all logging via `tracing`.
* The Nim frontend depends on `tatui >= 0.1.0`; it never re-implements TUI primitives.

## 3. EXECUTION PROMPT (Paste into Copilot Chat)
"""
Act as a Rust + Nim Platform Engineer.
Goal: Bootstrap the two-process workspace so implementation can begin.

Before generating code, open a `<reasoning>` block and model the module tree for both stacks.

Execute:
1. Create the `maestro` binary crate: `Cargo.toml` with tokio, thiserror, anyhow, tracing, clap, serde.
2. Scaffold `src/` with the four DDD layers and module docs; wire `main.rs` (clap + tracing-subscriber + `--no-tui`).
3. Add a boundary test proving `domain` has no `infrastructure`/`presentation` imports.
4. Scaffold `frontend/` (`maestro_tui.nimble` requiring `tatui >= 0.1.0`, `src/app.nim`, `src/protocol.nim`, `tests/`).

[Cohesion Mechanism]:
- Confirm both stacks build and the guarded CI jobs go green.

Return ONLY the modified code blocks in Markdown. No introduction.
"""

## 4. Acceptance Criteria
* **AC1:** `cargo build` and `cargo test` succeed; `maestro --help` shows a `--no-tui` flag.
* **AC2:** `src/` contains `domain/{models,ports}`, `application`, `infrastructure/{llm,bus,harness}`, `presentation/{cli,ipc}`; a test asserts domain-layer purity.
* **AC3:** `frontend/maestro_tui.nimble` requires `tatui >= 0.1.0` and the project compiles/tests headlessly.
* **AC4:** The guarded `ci.yml` Rust and Nim jobs both run green once `Cargo.toml` and `frontend/*.nimble` exist.

## 5. Validation Evidence
* **AC1:** `cargo build` ✓; `cargo test --all-targets` → 4 passed (3 CLI unit + 1 boundary); `maestro --help` lists `--no-tui`; `maestro --no-tui version` prints `maestro 0.1.0`.
* **AC2:** four DDD layers scaffolded with module docs; `tests/domain_boundary.rs::domain_does_not_depend_on_outer_layers` passes (rejects `crate::infrastructure`/`crate::presentation` in `src/domain`, ignoring comments).
* **AC3:** `tatui 0.1.1` resolved via `nimble`; `nimble build` produces `maestro_tui`; `nimble test` → 3 passed (headless protocol framing, no TTY).
* **AC4:** `scripts/quality-gate.sh` → "Quality gate OK" (Rust fmt/clippy/test + Nim `nimble test`). `nph` format check skipped locally (not installed); CI installs it.
* **Risk/rollback:** pure additive scaffolding; rollback = delete `Cargo.toml`, `Cargo.lock`, `src/`, `tests/`, `frontend/`.

