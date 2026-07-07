# TASK 048: Serial Cascade Executor

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** FSM engine, approved plan, rollback plan.
* **Context Anchors:** #file:.github/instructions/rollback-cascade.instructions.md, #file:docs/Maestro_Manifesto/ARCHITECTURE.md
* **Expected Output:** A serial executor that runs micro-project steps in cascade.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* Steps run strictly serial; no parallel environment-affecting steps.
* A failed step halts the cascade; downstream steps do not run.
* No environment action runs without a rollback plan and a fresh approval.

## 3. EXECUTION PROMPT (Paste into Copilot Chat)
"""
Act as a Rust Application Engineer.
Goal: Implement the serial cascade executor with halt-on-failure and pre-execution gating.

Before generating code, open a `<reasoning>` block and model the step sequence, the halt behavior, and the approval/rollback gate.

Execute:
1. Implement the executor in `src/application/`, delegating I/O through ports.
2. Require a rollback plan and a fresh IPC approval before any environment action.
3. Halt the cascade on the first failure and surface the applied-steps rollback.
4. Add tests for ordering, halt-on-failure, and required-rollback-before-execution.

[Cohesion Mechanism]:
- Confirm no `tokio::spawn` is used for environment-affecting steps.

Return ONLY the modified code blocks in Markdown. No introduction.
"""

## 4. Acceptance Criteria
* **AC1:** Tests prove steps execute in declared order.
* **AC2:** A failing step halts the cascade; downstream steps do not run.
* **AC3:** Execution is blocked until a rollback plan and approval exist.
