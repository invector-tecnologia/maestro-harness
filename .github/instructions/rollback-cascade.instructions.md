---
applyTo: "src/**/*.rs"
description: "Use when implementing serial cascade execution, rollback-as-a-service, or git-standalone persistence of micro-projects in Maestro's Rust core."
---

# Cascade & Rollback Rules

## Serial cascade
- Coordinated actions run **strictly in cascade (serial)** — never `tokio::spawn` parallel
  environment-affecting steps. Concurrency is allowed only for read-only observation.
- A failed step halts the cascade; downstream steps do not run.

## Rollback-as-a-service
- Before **any** environment-affecting action, produce a concrete rollback plan and request a fresh
  user approval over IPC. No rollback plan ⇒ no execution.
- Rollback steps are the inverse of forward steps and are themselves recorded and testable.
- On failure mid-cascade, surface the rollback plan for the steps already applied.

## Git-standalone persistence
- On successful completion, package the micro-project artifacts **and** the rollback state, then
  persist them to a standalone git repository for later reuse.
- Persistence goes through a port; `domain/` never touches git directly.

## Verification
- Test: cascade halts on first failure; rollback plan is required before execution; packaged output
  contains both artifacts and rollback state.
- All state transitions and approvals are logged via `tracing`.
