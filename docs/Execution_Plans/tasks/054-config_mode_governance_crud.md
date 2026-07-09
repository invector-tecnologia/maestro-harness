# TASK 054: Config Mode — governance CRUD + archive (absorbs Architect Mode)

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** The v2 IPC config messages (`src/presentation/ipc/mod.rs`), the duplex server
  (`src/presentation/ipc/server.rs`), the governance scaffold (`maestro/{personas,skills,scopes}`),
  the default persona catalog (`src/domain/models/persona.rs`), and the Config panel
  (`frontend/src/panels/config.nim`).
* **Context Anchors:** #file:docs/adr/0002-three-mode-workspace-and-interview-removal.md,
  #file:docs/User_Manual/COMMANDS_AND_PANELS.md
* **Expected Output:** Config Mode is the single governance surface. It lists, reads, creates,
  edits/updates, validates, and **archives** both **defaults** and **customs** of `config.yml`,
  personas, skills, and project scopes — with the Maestro orchestrator persona immutable.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* The Maestro persona (`personas/maestro`) can never be edited or archived at any layer.
* Archive is a soft delete: entries move under `maestro/archive/<kind>/`, never destroyed.
* Domain classification (kind, origin, immutability, slug) is pure; filesystem access lives in the
  application governance service; the server only marshals IPC.
* No `unwrap`/`expect`/`panic!` in normal Rust paths; `thiserror` errors; `tracing` logging.

## 3. ACCEPTANCE CRITERIA
* AC1: The domain exposes `GovernanceKind`, `Origin`, `GovernanceEntry`, `slug`, `is_immutable`,
  `default_persona_ids`, and `kind_of`, each unit-tested and pure.
* AC2: The application `governance` service provides `list`, `read`, `create`, `save`, `archive`,
  and `validate`; `read` synthesizes a default persona body when no override file exists.
* AC3: `list` returns `config.yml`, the built-in default personas (origin `default`), on-disk custom
  personas/skills/scopes (origin `custom`), and archived entries (marked `archived`).
* AC4: `save`/`archive` on `personas/maestro` fail with an immutable error and change no files.
* AC5: `archive` moves the entry to `maestro/archive/<kind>/` and subsequent `list` marks it archived.
* AC6: `validate` parses+cross-checks `config.yml` and rejects empty markdown bodies.
* AC7: The server serves `config_list`/`config_open`/`config_create`/`config_save`/`config_archive`/
  `config_validate` against the project root and refreshes the tree after mutations.
* AC8: The Nim Config panel is navigable (Up/Down selection, Enter opens the selected entry) and the
  shell requests `config_list` on entering Config Mode.
* AC9: All quality gates pass with added tests (Rust domain + application + server; Nim navigation).

## 4. VALIDATION COMMANDS
* `cargo fmt --all -- --check` · `cargo clippy --all-targets -- -D warnings` · `cargo test --all-targets`
* `scripts/quality-gate.sh` · `cd frontend && nimble test`

## 5. INCREMENT MAP
* INC1 (done): Domain `GovernanceKind`/`Origin`/`GovernanceEntry` + pure helpers. Covers AC1.
* INC2 (done): Application `governance` CRUD + archive + validate + `GovernanceError`. Covers AC2–AC6.
* INC3 (done): Server `config_*` handlers rooted at the project dir; tree refresh. Covers AC7.
* INC4 (done): Nim Config navigation + `config_list` on mode entry. Covers AC8.
* INC5 (done): Quality gates + E2E create/open/archive smoke. Covers AC9.

## 5b. VALIDATION EVIDENCE
* Rust: 80 unit + 1 boundary pass; clippy clean; `scripts/quality-gate.sh` OK.
* Nim: 19 tests pass (protocol + workspace navigation snapshots).
* E2E: `config_create → config_open → config_archive` moves the file to `maestro/archive/personas/`.

## 6. RESIDUAL RISKS
* Skill/scope schemas are not yet strictly validated (only non-empty); tighten with tasks 037/038.
* Config editing in the TUI is read/navigate today; in-place body editing lands with the Config editor
  keybindings pass.
