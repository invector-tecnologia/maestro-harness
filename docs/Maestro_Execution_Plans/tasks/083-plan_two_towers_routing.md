# Implementation Plan: Domain 2.6 — Two-Towers Persona↔Skill Routing Improvements

## Goal

Upgrade the deterministic Two-Towers routing from a naive word-overlap scorer to a
**weighted, multi-signal scorer** that produces auditable, narrated routing decisions —
closing the gap identified in [FEATURE_MAP.md](file:///home/bro/projects/maestro-harness/docs/Product_Engineering/FEATURE_MAP.md)
item 2.6 while staying within the MLP (Minimum Lovable Product) boundary.

### Current State

The routing module ([routing.rs](file:///home/bro/projects/maestro-harness/src/domain/models/routing.rs))
implements a simple lexical token-overlap scorer:

- Tokenises the demand and each persona's `id + responsibility + expertise_keywords`
- Counts exact-match overlapping tokens (case-insensitive, len > 2)
- Ranks by score descending, then id ascending (deterministic tie-break)
- Falls back to `"Software Engineer"` when nothing clears `MIN_SCORE = 1`

**Limitations identified in the FEATURE_MAP:**
- Lexical-only — misses semantic matches (e.g., "CI pipeline" vs "deploy container")
- `skill_tags` are defined on personas but **not used** in routing at all
- No narration/tracing of routing decisions (chosen persona, runner-up, score margin)
- No confidence weighting or routing explanation surfaced to the user

### What This Plan Delivers (MLP Scope)

1. **Weighted multi-signal scoring** — differentiated weights for id, responsibility, keywords, and skill_tags
2. **Skill-tags integration** — `skill_tags` finally participate in routing
3. **Routing narration via tracing** — chosen persona, runner-up, and margin logged
4. **Routing explanation in `Routing` struct** — surfaced through the `Signal::Plan` to the user
5. **Updated FEATURE_MAP.md** — reflecting the new state

### What Is Explicitly Deferred

> [!IMPORTANT]
> The following are deferred to keep MLP scope manageable:
> - Embedding-based scoring (requires an `EmbeddingProvider` port + Ollama `/api/embed`)
> - Hybrid lexical+semantic scoring (blocked on embeddings)
> - Learned routing from historical success data (requires persistence + metrics pipeline)
> - Confidence-weighted delegation (meaningful only with semantic scores)

---

## User Review Required

> [!IMPORTANT]
> **Scoring weights are configurable constants.** The proposed defaults below are based
> on the intuition that keywords and skill_tags carry more signal than the persona's name.
> Review these weights and override if you disagree:
>
> | Signal         | Weight | Rationale                                    |
> |----------------|--------|----------------------------------------------|
> | `id` tokens    | 1      | Baseline — persona name is a weak signal     |
> | `responsibility` tokens | 1 | The role description                    |
> | `expertise_keywords`    | 2 | Curated routing signal — should be worth more |
> | `skill_tags`            | 2 | Curated routing signal — should be worth more |

---

## Proposed Changes

### 1. Domain — Enhanced Scoring in `routing.rs`

#### [MODIFY] [routing.rs](file:///home/bro/projects/maestro-harness/src/domain/models/routing.rs)

**Add named weight constants:**

```rust
/// Scoring weights per signal source (named constants, not magic numbers).
pub const WEIGHT_ID: u32 = 1;
pub const WEIGHT_RESPONSIBILITY: u32 = 1;
pub const WEIGHT_KEYWORDS: u32 = 2;
pub const WEIGHT_SKILL_TAGS: u32 = 2;
```

**Rewrite `score()` to apply weighted scoring across all four signal sources:**

```rust
/// Weighted lexical overlap between demand tokens and persona signals.
fn score(demand_tokens: &[String], persona: &Persona) -> u32 {
    let id_tokens = tokens(&persona.id.to_string());
    let resp_tokens = tokens(&persona.responsibility);
    let kw_tokens: Vec<String> = persona.expertise_keywords.iter()
        .flat_map(|kw| tokens(kw))
        .collect();
    let tag_tokens: Vec<String> = persona.skill_tags.iter()
        .flat_map(|tag| tokens(tag))
        .collect();

    let id_hits = count_hits(demand_tokens, &id_tokens);
    let resp_hits = count_hits(demand_tokens, &resp_tokens);
    let kw_hits = count_hits(demand_tokens, &kw_tokens);
    let tag_hits = count_hits(demand_tokens, &tag_tokens);

    id_hits * WEIGHT_ID
        + resp_hits * WEIGHT_RESPONSIBILITY
        + kw_hits * WEIGHT_KEYWORDS
        + tag_hits * WEIGHT_SKILL_TAGS
}

/// Count how many demand tokens appear in the haystack.
fn count_hits(demand: &[String], haystack: &[String]) -> u32 {
    demand.iter()
        .filter(|t| haystack.iter().any(|h| h == *t))
        .count() as u32
}
```

**Add a `reason` field to `Routing`:**

```rust
pub struct Routing {
    pub ranked: Vec<PersonaMatch>,
    pub selected: Vec<String>,
    pub used_fallback: bool,
    /// Human-readable routing explanation.
    pub reason: String,
}
```

**Generate `reason` in `route()`:**

```rust
let reason = if used_fallback {
    format!("No persona cleared threshold ({}); fell back to {}", MIN_SCORE, FALLBACK_PERSONA)
} else {
    let top = &ranked[0];
    let runner_up = ranked.get(1);
    match runner_up {
        Some(ru) => format!(
            "Selected '{}' (score {}) over '{}' (score {}), margin {}",
            top.persona, top.score, ru.persona, ru.score, top.score.saturating_sub(ru.score)
        ),
        None => format!("Selected '{}' (score {}, sole candidate)", top.persona, top.score),
    }
};
```

---

### 2. Domain — Tracing Narration

#### [MODIFY] [routing.rs](file:///home/bro/projects/maestro-harness/src/domain/models/routing.rs)

> [!NOTE]
> The routing module lives in `domain/models/`. Strictly, domain should be pure and
> I/O-free. However, `tracing` is an **observability primitive**, not I/O — it's a
> structured log that doesn't affect control flow. This is consistent with how the
> FSM module already emits `tracing` events from `domain/models/fsm.rs`.

Add `tracing::info!` at the end of `route()`:

```rust
tracing::info!(
    selected = %ranked[0].persona,
    score = ranked[0].score,
    runner_up = ranked.get(1).map(|r| r.persona.as_str()).unwrap_or("none"),
    runner_up_score = ranked.get(1).map(|r| r.score).unwrap_or(0),
    margin = ranked[0].score.saturating_sub(ranked.get(1).map(|r| r.score).unwrap_or(0)),
    used_fallback,
    "two-towers routing decision"
);
```

---

### 3. Application — Surface Routing Explanation in `orchestrator.rs`

#### [MODIFY] [orchestrator.rs](file:///home/bro/projects/maestro-harness/src/application/orchestrator.rs)

The `Session::start()` method already includes a plan line `"route N persona(s): ..."`.
Extend this to include the routing reason:

```diff
 let mut plan = vec![
     format!("understand: {demand}"),
     format!(
         "route {} persona(s): {}",
         selected.len(),
         routing.selected.join(", ")
     ),
+    format!("routing: {}", routing.reason),
     "delegate in serial cascade".to_string(),
     "audit deliverables".to_string(),
     "deliver".to_string(),
 ];
```

This means the TUI and headless JSON output automatically show the routing decision
explanation to the user — no IPC protocol changes needed since it flows through the
existing `Signal::Plan(Vec<String>)`.

---

### 4. Tests — Verification Plan

#### [MODIFY] [routing.rs](file:///home/bro/projects/maestro-harness/src/domain/models/routing.rs) tests

Add new tests:

| Test | What It Validates |
|------|------------------|
| `weighted_keywords_beat_id_match` | A keyword match at weight 2 outscores a bare id match at weight 1 |
| `skill_tags_contribute_to_score` | Demand matching a `skill_tag` increases the score |
| `reason_explains_selection` | `Routing.reason` contains the chosen persona name and score |
| `reason_explains_fallback` | When fallback fires, `reason` mentions it |
| `tracing_emits_routing_decision` | (implicit via existing narrate tests — tracing doesn't need explicit test) |

**Baseline comparison** (required by the Two-Towers Routing skill):

The existing 5 tests form the baseline fixture set. All must continue passing with
identical outcomes — the weighted scoring is designed to be **backward-compatible**
(same winners, higher scores) because the weight for `id` and `responsibility` stays
at 1, which preserves the old counting behavior.

---

### 5. Documentation — FEATURE_MAP.md Update

#### [MODIFY] [FEATURE_MAP.md](file:///home/bro/projects/maestro-harness/docs/Product_Engineering/FEATURE_MAP.md)

Update item 2.6 to reflect the new state:

```diff
 ### 2.6 Two-Towers Persona↔Skill Routing
 
-- **Status:** ✅ Implemented
-+ **Source:** `src/domain/models/routing.rs`
-+ **Business Value:** 🟠 High
-+ **What It Does Today:** Lexical token-overlap scorer. Deterministic ranking with stable sort and
-+   tie-breaking by ID. Fallback to Software Engineer. Min score threshold of 1.
-+ **What It Should Do:** Embedding-based scoring (local embeddings via Ollama). Hybrid
-+   lexical+semantic scoring. Learned routing from historical success data. Confidence-weighted
-+   delegation. Routing explanation in narration.
-+ **Gap:** Lexical-only — misses semantic matches. No learning from outcomes.
++ **Status:** ✅ Implemented (enhanced)
++ **Source:** `src/domain/models/routing.rs`
++ **Business Value:** 🟠 High
++ **What It Does Today:** Weighted multi-signal lexical scorer (id, responsibility, keywords,
++   skill_tags with configurable weights). Deterministic ranking. Routing explanation in plan
++   narration. tracing-based decision logging with chosen/runner-up/margin.
++ **What It Should Do:** Embedding-based scoring (local embeddings via Ollama). Hybrid
++   lexical+semantic scoring. Learned routing from historical success data.
++ **Gap:** Lexical-only — misses semantic matches. No learning from outcomes.
```

---

## Verification Plan

### Automated Tests

```bash
# 1. Format
cargo fmt --all --check

# 2. Lint
cargo clippy --all-targets -- -D warnings

# 3. Unit + integration tests
cargo test --all-targets

# 4. Full quality gate
scripts/quality-gate.sh
```

### Manual Verification

1. Review the tracing output for the routing decision in a test run
2. Confirm all 5 existing routing tests still pass with the same outcomes (backward compatibility)
3. Confirm the new tests exercise weighted scoring and routing explanation

---

## Model & Category Recommendation

> [!NOTE]
> **Recommended model:** Gemini 3.1 Pro (Low)
>
> This is a focused, well-scoped change to a single domain module with a few touch points
> in the orchestrator. No complex async patterns, no infrastructure adapters. Low tier is
> appropriate.
