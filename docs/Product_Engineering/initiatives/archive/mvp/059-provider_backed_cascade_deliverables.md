# TASK 059: Provider-backed cascade deliverables (real LLM work)

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** The gated `Session` (`src/application/orchestrator.rs`), the provider registry
  (`src/infrastructure/llm/registry.rs`), the governed persona catalog
  (`application::governance::load_personas`), and the duplex server (`src/presentation/ipc/server.rs`).
* **Context Anchors:** #skill:two-towers-routing, #file:docs/adr/0002-three-mode-workspace-and-interview-removal.md
* **Expected Output:** During Execution, each persona's deliverable is produced by the configured LLM
  provider when a model is reachable, and by a deterministic placeholder otherwise.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* The orchestrator `Session` stays pure/deterministic over an injected `Deliverer`; no provider types
  cross into `domain`/`application` orchestration logic.
* The provider is used only when its probe returns `Available`; any failure falls back deterministically.
* No `unwrap`/`expect`/`panic!` in normal paths; provider errors degrade, never crash.

## 3. ACCEPTANCE CRITERIA
* AC1: `Session::execute` takes a `Deliverer` (`Fn(persona, model, demand) -> String`); `orchestrate`
  and tests use the deterministic `placeholder_deliverable`.
* AC2: An injected deliverer's output flows into both the `Deliverable` signals and the release
  deliverables (verified by test).
* AC3: The server builds a provider-backed deliverer only at the execution gate, and only when the
  default provider probes `Available`; otherwise it uses the placeholder.
* AC4: Offline/CI runs remain deterministic (no config or unreachable provider → placeholder).
* AC5: All quality gates pass.

## 4. VALIDATION COMMANDS
* `cargo fmt --all -- --check` · `cargo clippy --all-targets -- -D warnings` · `cargo test --all-targets`
* `scripts/quality-gate.sh`

## 5. INCREMENT MAP
* INC1 (done): `Deliverer` type + `placeholder_deliverable`; thread through `resume`/`execute`. AC1, AC2.
* INC2 (done): server `build_completer`/`build_deliverer`; `Session::awaiting_execution`. AC3, AC4.
* INC3 (done): quality gates. AC5.

## 5b. VALIDATION EVIDENCE
* Rust: 119 unit + 1 boundary pass; clippy clean; `scripts/quality-gate.sh` OK.
* The `injected_deliverer_output_flows_into_deliverables` test proves the seam; offline E2E still
  yields deterministic placeholder deliverables.

## 6. RESIDUAL RISKS
* Live provider completions are not exercised in CI (no model/keys); the seam and fallback are tested.
* Persona instructions/skills are not yet injected into the prompt (only name + demand); richer prompt
  construction (RAG, skills, scope) is future work.
