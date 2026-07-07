---
applyTo: "src/**/*.rs"
description: "Use when implementing or reviewing Two-Towers persona↔skill routing: deterministic matcher, scoring, and reproducible selection in Maestro's Rust core."
---

# Two-Towers Routing Rules

## Purpose
Select the best sub-agent (persona) for a micro-project by scoring a **persona tower** against a
**skill/task tower**. Selection must be deterministic and testable — never an ad-hoc heuristic
scattered across adapters.

## Design rules
- Keep the matcher in `domain/`/`application/`; embedding *providers* are ports implemented in
  `infrastructure/`.
- Given identical inputs, the ranked output must be identical (stable sort with an explicit
  tie-breaker; no reliance on hash-map iteration order).
- Return a ranked list with scores, not just the top pick, so decisions are auditable.
- Log the chosen persona, runner-up, and score margin via `tracing`.

## Safety rules
- Fall back to a documented default persona when no candidate clears the minimum score threshold;
  never route to an empty/undefined persona.
- Thresholds and weights are named constants or config — no magic numbers inline.

## Verification
- Unit-test determinism (same input → same ranking), tie-breaking, and the fallback path.
- When scoring or weighting changes, compare baseline vs new ranking on a fixed fixture set.
