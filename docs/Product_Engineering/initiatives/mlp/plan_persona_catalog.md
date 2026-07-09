# Plan: Domain 2.2 — Default Persona Catalog Enrichment

## Goal

Expand the default persona catalog from 5 to 8 roles and enrich the `Persona` model
with three new fields: **system prompt**, **skill tags**, and **expertise keywords**.
This closes the gap identified in the FEATURE_MAP: *"Personas are name+responsibility
text only. No system prompts, no tool bindings."*

## Background

Today each `Persona` carries only `id`, `responsibility`, `can_handoff_to`, and
`orchestrator`. The system prompt injected by `PersonaAgent` is auto-generated at
runtime from the bare responsibility text. The Two-Towers router scores only on
lexical overlap with `id + responsibility`.

Competitors like MetaGPT ship full SOP-per-role and CrewAI attaches backstory, goal,
and tools. To compete, Maestro's personas need richer metadata that:

1. Gives the LLM a proper identity via a **curated system prompt**.
2. Improves routing precision with **expertise keywords** (more signal than bare
   responsibility text).
3. Prepares for skill binding with a **skill tags** list the scheduler can reference.

## Proposed Changes

### 1. Enrich the `Persona` model

#### [MODIFY] `src/domain/models/persona.rs`

Add three new fields to `Persona`:

```diff
 pub struct Persona {
     pub id: AgentId,
     pub responsibility: String,
     pub can_handoff_to: Vec<AgentId>,
     #[serde(default)]
     pub orchestrator: bool,
+    /// Multi-line system prompt template. Injected as the first message in act().
+    #[serde(default)]
+    pub system_prompt: String,
+    /// Expertise keywords for Two-Towers routing (enriches scoring signal).
+    #[serde(default)]
+    pub expertise_keywords: Vec<String>,
+    /// Skill tags this persona is associated with (future skill binding).
+    #[serde(default)]
+    pub skill_tags: Vec<String>,
 }
```

- `Persona::new()` gains three new parameters (with defaults for backwards compat).
- The helper `fn persona(...)` in the default catalog is updated.

---

### 2. Expand the default catalog to 8 personas

#### [MODIFY] `src/domain/models/persona.rs` — `default_personas()`

Three new operational personas:

| Persona | Responsibility | Keywords |
|---|---|---|
| **DevOps Engineer** | Automate infrastructure, CI/CD, and environment provisioning. | ci, cd, pipeline, docker, container, deploy, infra, terraform, kubernetes |
| **Security Analyst** | Identify threats, review access controls, and enforce security policy. | security, threat, vulnerability, auth, access, policy, encryption, audit |
| **Technical Writer** | Produce clear documentation, READMEs, and user guides. | documentation, readme, guide, writing, docs, api-docs, changelog |

Each persona gets:
- A curated multi-line **system prompt** (3-5 sentences defining behavior).
- An **expertise keywords** list (8-12 tokens).
- **Skill tags** (2-3 tags like `["infra", "ci-cd"]`).
- `can_handoff_to: [Maestro]` (same as existing operational personas).

The existing 4 operational personas are also enriched with system prompts, keywords,
and skill tags.

Maestro (orchestrator) gets a system prompt and keywords but no skill tags (it
delegates, not executes).

---

### 3. Use curated system prompt in `PersonaAgent`

#### [MODIFY] `src/application/persona_agent.rs`

```diff
     fn system_prompt(&self) -> String {
-        format!(
-            "You are '{}'. Your responsibility: {}\n\n\
-             Follow a structured approach:\n\
-             ...",
-            self.persona.id, self.persona.responsibility
-        )
+        if !self.persona.system_prompt.is_empty() {
+            self.persona.system_prompt.clone()
+        } else {
+            // Fallback for custom personas without a curated prompt
+            format!(
+                "You are '{}'. Your responsibility: {}\n\n\
+                 Follow a structured approach:\n\
+                 1. Interpret the task in terms of your specific role.\n\
+                 2. Apply your expertise to produce a focused, actionable contribution.\n\
+                 3. Flag any risks or concerns within your domain.\n\
+                 4. Stay within your responsibility boundary — delegate what is outside it.",
+                self.persona.id, self.persona.responsibility
+            )
+        }
     }
```

This means all 8 default personas get high-quality curated prompts, while custom
personas created by users in Config Mode still get the auto-generated fallback.

---

### 4. Enrich Two-Towers routing with expertise keywords

#### [MODIFY] `src/domain/models/routing.rs`

The `score()` function currently builds its haystack from `id + responsibility`.
Add `expertise_keywords`:

```diff
 fn score(demand_tokens: &[String], persona: &Persona) -> u32 {
     let mut haystack = tokens(&persona.id.to_string());
     haystack.extend(tokens(&persona.responsibility));
+    for kw in &persona.expertise_keywords {
+        haystack.extend(tokens(kw));
+    }
     demand_tokens
         .iter()
         .filter(|t| haystack.iter().any(|h| h == *t))
         .count() as u32
 }
```

This means routing now considers the richer vocabulary, improving routing precision.
For example, "deploy the container" would now match DevOps Engineer via keywords
even if those terms aren't in the responsibility text.

---

### 5. Update governance reader

#### [MODIFY] `src/application/governance.rs`

The `read()` function synthesizes a markdown body for default personas. Update it to
include the new fields:

```diff
-            return Ok(format!(
-                "# {}\n\n## Responsibility\n{}\n",
-                persona.id, persona.responsibility
-            ));
+            let mut body = format!(
+                "# {}\n\n## Responsibility\n{}\n",
+                persona.id, persona.responsibility
+            );
+            if !persona.system_prompt.is_empty() {
+                body.push_str(&format!("\n## System Prompt\n{}\n", persona.system_prompt));
+            }
+            if !persona.expertise_keywords.is_empty() {
+                body.push_str(&format!(
+                    "\n## Expertise Keywords\n{}\n",
+                    persona.expertise_keywords.join(", ")
+                ));
+            }
+            if !persona.skill_tags.is_empty() {
+                body.push_str(&format!(
+                    "\n## Skill Tags\n{}\n",
+                    persona.skill_tags.join(", ")
+                ));
+            }
+            return Ok(body);
```

---

### 6. Update tests

All touched files need test updates:

| Test file | Changes |
|---|---|
| `persona.rs` tests | `default_catalog_has_five_personas` → `default_catalog_has_eight_personas`. New test: `all_default_personas_have_system_prompts`. New test: `all_operational_personas_have_expertise_keywords`. |
| `routing.rs` tests | New test: `keywords_improve_routing_precision` (demand "deploy container" routes to DevOps). Update existing tests for 7 operational personas (was 4). |
| `persona_agent.rs` tests | New test: `uses_curated_system_prompt_when_available`. |
| `governance.rs` tests | `read_synthesizes_default_persona_body` verifies system prompt section appears. |

---

### 7. Update documentation

#### [MODIFY] `docs/Product_Engineering/FEATURE_MAP.md`

Update Item 2.2 status and gap:

```diff
-- **What It Does Today:** 5 personas: Maestro (orchestrator), Project Manager, QA, UX, Software
-  Engineer. Each has responsibility text and interaction (handoff) matrix.
+- **What It Does Today:** 8 personas: Maestro (orchestrator), Project Manager, QA, UX, Software
+  Engineer, DevOps Engineer, Security Analyst, Technical Writer. Each has responsibility text,
+  curated system prompt, expertise keywords, skill tags, and interaction matrix.
-- **Gap:** Personas are name+responsibility text only. No system prompts, no tool bindings.
+- **Gap:** No temperature settings or example outputs yet. Skill tags are defined but
+  not yet wired to runtime skill selection. Tool access lists are not implemented.
```

## Recommended Model

**Gemini 3.1 Pro (Low)** — This is mostly mechanical enrichment of data structures and
constants, with straightforward test updates. No complex architecture or design work.

## Verification Plan

### Automated Tests

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

### Manual Verification

- Confirm all 8 personas appear in `maestro list-personas`.
- Confirm routing for "deploy the container" selects DevOps Engineer.
- Confirm `read()` on a default persona shows system prompt in synthesized body.
