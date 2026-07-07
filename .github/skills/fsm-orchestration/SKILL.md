---
name: fsm-orchestration
description: "Use when designing or reviewing Maestro's micro-project finite state machine: the six stages, legal transitions, approval/rollback gates, and instrumentation."
---

# FSM Orchestration Skill

## Use When
- Implementing or reviewing the micro-project lifecycle in the Rust core.
- Adding a stage, gate, or transition, or wiring approvals to the TUI.

## The machine
`Ideation → Planning → Approval → Instrumentation → Execution → Verification`

| Stage | Purpose | Gate |
|---|---|---|
| Ideation | Capture the user request | — |
| Planning | Produce the strategic plan | — |
| Approval | User validates the plan | explicit user approval (IPC) |
| Instrumentation | Generate personas + inject prompts/skills/RAG/specs | — |
| Execution | Serial cascade of actions | rollback plan + fresh approval before env actions |
| Verification | Validate outcome, package + persist | — |

## Workflow
1. Model stages/transitions as explicit domain types; reject illegal transitions with typed errors.
2. Emit a `tracing` event on every transition (id, from, to).
3. Block at gated stages until the matching IPC response arrives; never assume consent.
4. Record instrumented artifacts so the run is reproducible.
5. Cover the full transition table + rejected-illegal-transition cases in tests.

## Outputs
- A deterministic, fully-logged FSM with tested legal/illegal transitions and an ADR for any change
  to stages or gates.
