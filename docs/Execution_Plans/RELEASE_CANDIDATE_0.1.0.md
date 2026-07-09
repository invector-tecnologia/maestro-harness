# Maestro MLP 0.1.0 - Release Candidate Checklist

## Scope
This checklist consolidates evidence from Task 016 for MLP 0.1.0 validation.

## Required gates
- [x] `cargo fmt --all --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --all-targets`
- [x] `nimble test` (Nim/Tatui TUI, headless test backend)

### Consolidated gate evidence
- Script executed: `./scripts/quality-gate.sh`
- Result: `Quality gate OK`
- Rust suite: `57 passed; 0 failed` (unit) + `1 passed` (architecture boundary integration test)
- Nim suite: `6 passed` (dashboard snapshot + stdio protocol framing, no TTY)

## MLP requirements coverage
- [x] Complete multi-agent flow (runtime + failure isolation)
  - Evidence: `application::agent_runtime::tests::{runs_cycle_and_collects_outputs, failing_agent_is_isolated}`
- [x] Default operational personas (Project Manager / Quality Assurance / User Experience / Software Engineer)
  - Evidence: `application::persona_agent::tests::activates_four_operational_personas`, `domain::models::persona::tests::*`
- [x] TUI with agent panel, logs, and command input (Nim/Tatui)
  - Evidence: `frontend/tests/test_dashboard.nim` (headless test-backend snapshot of the dashboard panels)
- [x] Required persona/scope/skill creation wizards with required-field blocking
  - Evidence: unit tests `application::wizard::tests::*`
- [x] External configuration validated (schema, type, and cross references)
  - Evidence: unit tests `domain::models::config::tests::*` and `infrastructure::config::tests::*`
- [x] Provider registry + reference Ollama adapter
  - Evidence: unit tests `infrastructure::llm::registry::tests::*` and `infrastructure::llm::ollama::tests::*`
- [x] Rust↔Nim IPC stdio protocol (versioned, schema-checked)
  - Evidence: unit tests `presentation::ipc::tests::*` and `frontend/tests/test_protocol.nim`
- [x] Operational CLI (`version`, `validate-config`, `list-agents`, `doctor`, `scaffold-markdown`, `init-config`, `--no-tui`)
  - Evidence: unit tests `presentation::cli::tests::*`
- [x] Debian packaging prepared (`.deb`) with remove/purge lifecycle
  - Evidence: `scripts/build-deb.sh` (control + `postrm purge` hook)

## Debian validation status
- [ ] Debian smoke test executed in a clean environment with `dpkg`/`dpkg-deb`
  - Expected command:
    - `./scripts/build-deb.sh 0.1.0`
    - `./scripts/smoke-test-debian.sh target/deb/maestro-ai_0.1.0_$(dpkg --print-architecture).deb`
  - Note: in the current automation environment, `dpkg` was not available.

## Release readiness criterion
Release Candidate 0.1.0 is approved when:
1. All required gates pass.
2. Coverage checklist remains 100% checked.
3. Debian smoke test is successfully executed in a clean environment.

## Current RC status
- Quality gates: APPROVED
- MLP functional coverage: APPROVED
- Debian smoke test in clean environment: PENDING (depends on host with `dpkg`/`dpkg-deb`)

## Single gate command
```bash
./scripts/quality-gate.sh
```
