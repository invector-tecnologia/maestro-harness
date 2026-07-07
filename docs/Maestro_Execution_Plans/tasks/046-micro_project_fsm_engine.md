# TASK 046: Micro-Project FSM Engine

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** Approved architecture (hexagonal core), event bus.
* **Context Anchors:** #file:docs/Maestro_Manifesto/ARCHITECTURE.md, #file:.github/instructions/fsm-orchestration.instructions.md, #file:docs/adr/0001-rust-core-nim-tatui-stdio-protocol.md
* **Expected Output:** A typed finite state machine driving every micro-project.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* Stages are explicit domain types, never bare strings.
* Only legal transitions succeed; illegal transitions return a typed error.
* Every transition emits a `tracing` event (id, from, to).

## 3. EXECUTION PROMPT (Paste into Copilot Chat)
"""
Act as a Rust Domain Engineer.
Goal: Implement the micro-project FSM: Ideation → Planning → Approval → Instrumentation → Execution → Verification.

Before generating code, open a `<reasoning>` block and model the legal transition table and the gated stages (Approval, pre-Execution rollback gate).

Execute:
1. Model stages and transitions as domain types in `src/domain/`.
2. Drive transitions from `src/application/`, delegating I/O through ports.
3. Block gated stages until an IPC approval response arrives.
4. Add tests for the full legal transition table plus one rejected-illegal-transition case per stage.

[Cohesion Mechanism]:
- Confirm no illegal transition can silently no-op.

Return ONLY the modified code blocks in Markdown. No introduction.
"""

## 4. Acceptance Criteria
* **AC1:** All legal transitions are covered by passing unit tests.
* **AC2:** Each stage rejects at least one illegal transition with a typed error.
* **AC3:** Gated stages block until an approval response is received over IPC.
