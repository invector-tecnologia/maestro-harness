# TASK 047: Two-Towers Persona↔Skill Routing

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** Persona catalog, skill/task descriptors, embedding provider port.
* **Context Anchors:** #file:.github/instructions/two-towers-routing.instructions.md, #file:docs/Maestro_Manifesto/ARCHITECTURE.md
* **Expected Output:** A deterministic matcher that ranks personas for a micro-project.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* Identical inputs must produce an identical ranking.
* Thresholds and weights are named constants or config, never inline magic numbers.
* A documented default persona is used when nothing clears the minimum score.

## 3. EXECUTION PROMPT (Paste into Copilot Chat)
"""
Act as a Rust Domain Engineer.
Goal: Implement Two-Towers routing that scores a persona tower against a skill/task tower and returns a ranked selection.

Before generating code, open a `<reasoning>` block and model the scoring, the stable tie-breaker, and the fallback path.

Execute:
1. Keep the matcher in `src/domain`/`src/application`; embeddings via an `infrastructure` port.
2. Return a ranked list with scores and margin; log the pick and runner-up via `tracing`.
3. Apply the default persona below the minimum threshold.
4. Add tests for determinism, tie-breaking, and fallback.

[Cohesion Mechanism]:
- Confirm ranking does not depend on hash-map iteration order.

Return ONLY the modified code blocks in Markdown. No introduction.
"""

## 4. Acceptance Criteria
* **AC1:** A determinism test proves identical input → identical ranking.
* **AC2:** Tie-breaking and fallback are covered by tests.
* **AC3:** Scoring changes ship with a baseline-vs-new ranking delta on a fixed fixture set.
