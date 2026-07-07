---
applyTo: "src/**/*.rs"
description: "Use when implementing or reviewing the micro-project finite state machine: stage transitions, approval gates, and instrumentation in Maestro's Rust core."
---

# FSM Orchestration Rules

## The canonical machine
Every micro-project advances through exactly these stages, in order:

`Ideation → Planning → Approval → Instrumentation → Execution → Verification`

## Transition rules
- Model stages and transitions as explicit domain types; never encode stage as a bare string.
- Only legal transitions are allowed. Reject illegal transitions with a typed error (`thiserror`) —
  never silently no-op.
- Every transition emits a `tracing` event carrying the micro-project id, from-stage, and to-stage.
- The FSM lives in `application/`; it is driven by pure `domain/` state and never performs I/O
  directly (delegate through ports).

## Gated stages
- **Approval** and the pre-**Execution** rollback gate require an explicit user decision relayed over
  the IPC boundary. The core blocks until an `approval_response` arrives; it never assumes consent.
- **Instrumentation** generates personas and injects their system prompt, skills, RAG context, and
  `.spec`/`.json` files. Instrumented artifacts are recorded so the run is reproducible.

## Verification
- Cover the full legal transition table with unit tests, plus at least one rejected-illegal-transition
  test per stage.
- Any new stage, gate, or transition semantics requires an ADR in `docs/adr/`.
