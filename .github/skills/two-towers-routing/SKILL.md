---
name: two-towers-routing
description: "Use when implementing or tuning Maestro's Two-Towers persona↔skill matcher: deterministic scoring, ranking, tie-breaking, and fallback selection."
---

# Two-Towers Routing Skill

## Use When
- Selecting the best persona for a micro-project.
- Changing scoring, weighting, or the fallback policy of the matcher.

## Concept
Two encoders produce embeddings — a **persona tower** and a **skill/task tower**. The matcher scores
personas against the task and returns a ranked, reproducible selection.

## Workflow
1. Keep the matcher in `domain`/`application`; embeddings come through an `infrastructure` port.
2. Score all candidates; sort stably with an explicit tie-breaker (never rely on map order).
3. Return a ranked list with scores + margin; log the pick and runner-up via `tracing`.
4. Apply a documented default persona when nothing clears the minimum threshold.
5. Keep thresholds/weights in named constants or config — no inline magic numbers.

## Verification
- Determinism test: identical input → identical ranking.
- Tie-break and fallback tests.
- Baseline-vs-new ranking comparison on a fixed fixture set when scoring changes.

## Outputs
- An auditable, deterministic routing decision with tests and (for scoring changes) a before/after
  ranking delta.
