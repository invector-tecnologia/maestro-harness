# TASK 050: Git-Standalone Micro-Project Persistence

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** Completed micro-project artifacts, rollback state.
* **Context Anchors:** #file:.github/instructions/rollback-cascade.instructions.md, #file:docs/Maestro_Manifesto/ARCHITECTURE.md
* **Expected Output:** Completed micro-projects packaged into a standalone git repository.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* Persistence runs only after successful Verification.
* Packaged output contains both artifacts and rollback state.
* `domain/` never touches git; persistence goes through a port.

## 3. EXECUTION PROMPT (Paste into Copilot Chat)
"""
Act as a Rust Infrastructure Engineer.
Goal: Package a completed micro-project and its rollback state into a standalone git repository for reuse.

Before generating code, open a `<reasoning>` block and model the git-as-a-service port and the package contents.

Execute:
1. Define a persistence port in `src/domain/ports/` and implement it in `src/infrastructure/`.
2. Package artifacts + rollback state; initialize/commit to a standalone repo.
3. Make packages retrievable for later reuse.
4. Add tests for package contents and the post-Verification trigger.

[Cohesion Mechanism]:
- Confirm no git calls exist in `domain/`.

Return ONLY the modified code blocks in Markdown. No introduction.
"""

## 4. Acceptance Criteria
* **AC1:** Packaging triggers only after successful Verification.
* **AC2:** Packages contain both artifacts and rollback state.
* **AC3:** No git dependency is imported in `domain/`.
