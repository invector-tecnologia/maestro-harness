# TASK 058: Providers, accessibility, and release candidate 0.3.0

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** The provider registry (`src/infrastructure/llm/registry.rs`), the Ollama adapter
  pattern (`src/infrastructure/llm/ollama.rs`), the Nim Workspace panels (`frontend/src/panels/`),
  and the config template (`src/presentation/cli/mod.rs`).
* **Context Anchors:** #file:docs/Maestro_Execution_Plans/RELEASE_CANDIDATE_0.3.0.md,
  #file:docs/adr/0002-three-mode-workspace-and-interview-removal.md
* **Expected Output:** An optional OpenAI-compatible cloud provider, an accessibility ASCII fallback,
  and a documented release candidate — with Ollama remaining the local-first default.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* Cloud credentials come from the environment (`OPENAI_API_KEY`), never config files.
* Unknown provider kinds still fail fast.
* The ASCII fallback changes only presentation; the protocol and core are untouched.
* No `unwrap`/`expect`/`panic!` in normal paths; `thiserror`; `tracing` logging.

## 3. ACCEPTANCE CRITERIA
* AC1: `OpenAiProvider` implements `LlmProvider` against an OpenAI-compatible API (`/models` probe,
  `/chat/completions`); an empty key degrades to `Unauthorized` rather than calling out.
* AC2: The registry builds the `openai` kind with the key read from `OPENAI_API_KEY`; `ollama`
  remains the default and unknown kinds still fail fast.
* AC3: `MAESTRO_ASCII_ONLY` renders the Workspace borderless with `[Title]` headers and no Unicode
  box-drawing.
* AC4: The config template documents the optional `openai` provider (commented, key via env).
* AC5: A release-candidate document records scope and test evidence.
* AC6: All quality gates pass with tests for the adapter (pure helpers + unauthorized path), the
  registry kind, and the ASCII fallback.

## 4. VALIDATION COMMANDS
* `cargo fmt --all -- --check` · `cargo clippy --all-targets -- -D warnings` · `cargo test --all-targets`
* `scripts/quality-gate.sh` · `cd frontend && nimble test`

## 5. INCREMENT MAP
* INC1 (done): `infrastructure/llm/openai.rs` OpenAI-compatible adapter + tests. AC1.
* INC2 (done): registry `openai` kind with env key + test. AC2.
* INC3 (done): `frontend/src/theme.nim` + panel wiring for ASCII fallback + test. AC3.
* INC4 (done): config template optional `openai` block. AC4.
* INC5 (done): `RELEASE_CANDIDATE_0.3.0.md`. AC5.
* INC6 (done): quality gates green. AC6.

## 5b. VALIDATION EVIDENCE
* Rust: 116 unit + 1 boundary pass; clippy clean; `scripts/quality-gate.sh` OK. Nim: 21 tests.

## 6. RESIDUAL RISKS
* Native Anthropic/Gemini adapters remain future work (their APIs differ from OpenAI's).
* Live cloud completions are untested here (no network/keys in CI); pure helpers + the unauthorized
  path are covered.
