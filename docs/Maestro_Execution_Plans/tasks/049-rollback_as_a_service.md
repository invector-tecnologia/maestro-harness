# TASK 049: Rollback-as-a-Service

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** Cascade executor, micro-project step list.
* **Context Anchors:** #file:.github/instructions/rollback-cascade.instructions.md, #file:.github/agents/rollback-planner.agent.md
* **Expected Output:** A rollback planner and executor invoked before the Execution gate.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* No rollback plan ⇒ no execution.
* Every forward step has an explicit inverse; irreversible steps are flagged.
* Rollback ordering is the reverse of the forward cascade.

## 3. EXECUTION PROMPT (Paste into Copilot Chat)
"""
Act as a Rust Application Engineer.
Goal: Implement rollback-as-a-service: produce and, on failure, apply the inverse of the cascade.

Before generating code, open a `<reasoning>` block and model the forward→inverse mapping and irreversible-step handling.

Execute:
1. Generate a rollback plan (inverse steps + per-step verification) before the Execution gate.
2. Require explicit user acknowledgement for irreversible steps over IPC.
3. On mid-cascade failure, apply the rollback for already-applied steps in reverse order.
4. Add tests: rollback required before execution, reverse ordering, irreversible-step warning.

[Cohesion Mechanism]:
- Confirm the Execution gate cannot open without a rollback plan.

Return ONLY the modified code blocks in Markdown. No introduction.
"""

## 4. Acceptance Criteria
* **AC1:** Execution is impossible without a rollback plan (tested).
* **AC2:** Rollback applies inverse steps in reverse order.
* **AC3:** Irreversible steps require explicit acknowledgement.
