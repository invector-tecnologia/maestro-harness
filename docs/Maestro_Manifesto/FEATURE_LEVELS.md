# Feature Levels

This document separates Maestro capabilities into feature maturity levels.

## Foundational Level
Target: first-time users and initial project setup.

Features:
- `maestro init`, `maestro init-config`, and `maestro validate-config`.
- Readiness checks and baseline doctor validations.
- Initial markdown governance scaffold.
- Single-provider local setup and config loading.

## Core Level
Target: daily multi-agent operation.

Features:
- Multi-agent runtime (`observe`, `think`, `act`).
- TUI dashboard and command center.
- Persona model with interaction matrix and handoff rules.
- Provider registry and default provider resolution.
- Mandatory quality gates and structured tracing.

## Advanced Level
Target: teams scaling adoption and governance.

Features:
- Three-mode Workspace (Config · Maestro · Product) over the stdio protocol v2.
- Governed execution: approval gates, rollback, and git-standalone release persistence.
- Accessibility controls (ASCII fallback via `MAESTRO_ASCII_ONLY`).
- Cross-platform packaging and smoke-test workflows.

## Enterprise Level
Target: broad organizational rollout.

Features (roadmap-oriented):
- Compliance reporting and policy extension.
- Broader provider ecosystem and operational governance exports.
- Expanded reliability automation and audit analytics.

## Traceability
- Runtime and personas: `src/application/agent_runtime.rs`, `src/application/orchestrator.rs`.
- TUI: `frontend/src/` (Nim/Niobium); IPC: `src/presentation/ipc/`.
- CLI operations: `src/presentation/cli/mod.rs`.
- Packaging and quality gates: `scripts/quality-gate.sh`, `scripts/build-*.sh`.
