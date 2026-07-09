# Plan: 2.3 — Custom Personas (Rich Schema & End-to-End Wiring)

## Goal

Close the gap between **default personas** (which now carry system_prompt, expertise_keywords,
skill_tags) and **custom personas** authored in `maestro/personas/*.md`. Today, custom persona
markdown files only extract `# Name` and `## Responsibility`. This plan enriches the parser to
extract all fields and wires the curated `system_prompt` through the deliverer pipeline, so that
both built-in and user-authored personas benefit from the enriched schema.

## Background

Item 2.2 expanded the `Persona` struct and the 8 built-in personas with:
- `system_prompt` — curated LLM identity instruction
- `expertise_keywords` — tokens fed into Two-Towers routing
- `skill_tags` — future skill binding

But `parse_custom_persona()` in `governance.rs` still creates personas with empty strings/vecs
for those three fields. And `build_deliverer()` in `server.rs` still ignores the persona's
curated `system_prompt`, using a hardcoded generic one instead.

## FEATURE_MAP Entry (Before)

```
- **Gap:** Schema is minimal (name + responsibility). No system prompts or tool bindings.
```

## FEATURE_MAP Entry (After — proposed revalidation)

```
- **What It Does Today:** YAML persona files in `maestro/personas/`, loaded and merged with defaults
  for Two-Towers routing. Config Mode governance navigator + editor in TUI. Custom persona
  markdown supports `## System Prompt`, `## Expertise Keywords`, and `## Skill Tags` sections.
  Curated system prompts are wired end-to-end into the deliverer pipeline.
- **Gap:** No tool whitelists, temperature settings, or context window strategy per persona. No
  visual persona editor.
```

---

## Proposed Changes

### 1. Application — `governance.rs` (enrich `parse_custom_persona`)

#### [MODIFY] governance.rs

Extend the markdown section parser to extract three new sections:

```diff
 fn parse_custom_persona(body: &str, maestro: &AgentId) -> Option<Persona> {
     let mut name = String::new();
     let mut responsibility = String::new();
+    let mut system_prompt = String::new();
+    let mut expertise_keywords = Vec::new();
+    let mut skill_tags = Vec::new();
     let mut in_responsibility = false;
+    let mut in_system_prompt = false;
+    let mut in_keywords = false;
+    let mut in_skill_tags = false;
     for line in body.lines() {
         let trimmed = line.trim();
         if let Some(heading) = trimmed.strip_prefix("# ") {
             if name.is_empty() {
                 name = heading.trim().to_string();
             }
         } else if trimmed.eq_ignore_ascii_case("## responsibility") {
-            in_responsibility = true;
+            in_responsibility = true; in_system_prompt = false; in_keywords = false; in_skill_tags = false;
+        } else if trimmed.eq_ignore_ascii_case("## system prompt") {
+            in_system_prompt = true; in_responsibility = false; in_keywords = false; in_skill_tags = false;
+        } else if trimmed.eq_ignore_ascii_case("## expertise keywords") {
+            in_keywords = true; in_responsibility = false; in_system_prompt = false; in_skill_tags = false;
+        } else if trimmed.eq_ignore_ascii_case("## skill tags") {
+            in_skill_tags = true; in_responsibility = false; in_system_prompt = false; in_keywords = false;
         } else if trimmed.starts_with("## ") {
-            in_responsibility = false;
+            in_responsibility = false; in_system_prompt = false; in_keywords = false; in_skill_tags = false;
         } else if in_responsibility && responsibility.is_empty() && !trimmed.is_empty() {
             responsibility = trimmed.to_string();
+        } else if in_system_prompt && !trimmed.is_empty() {
+            // Accumulate multi-line system prompt
+            if !system_prompt.is_empty() { system_prompt.push('\n'); }
+            system_prompt.push_str(trimmed);
+        } else if in_keywords && !trimmed.is_empty() {
+            // Comma-separated keywords on one or more lines
+            expertise_keywords.extend(trimmed.split(',').map(|k| k.trim().to_lowercase()).filter(|k| !k.is_empty()));
+        } else if in_skill_tags && !trimmed.is_empty() {
+            skill_tags.extend(trimmed.split(',').map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty()));
         }
     }
     ...
     Persona::new(id, responsibility, vec![maestro.clone()], false,
-                  "", vec![], vec![])
+                  system_prompt, expertise_keywords, skill_tags)
     .ok()
 }
```

**New tests to add:**

- `parse_custom_persona_extracts_system_prompt` — a custom markdown with `## System Prompt` section, assert it arrives in the persona.
- `parse_custom_persona_extracts_keywords_and_tags` — comma-separated values under `## Expertise Keywords` and `## Skill Tags`.
- `custom_persona_enriches_routing` — a custom persona with keywords that match a demand should score higher than one without.

---

### 2. Application — `governance.rs` (validate persona markdown)

#### [MODIFY] governance.rs

Enhance the `validate()` function to perform structural validation on persona markdown:

```diff
 pub fn validate(id: &str, body: &str) -> (bool, Vec<String>) {
     if id == "config.yml" {
         // ... existing YAML validation ...
+    } else if id.starts_with("personas/") {
+        let mut issues = Vec::new();
+        if !body.lines().any(|l| l.trim().starts_with("# ")) {
+            issues.push("persona must have a `# Name` heading".to_string());
+        }
+        if !body.to_lowercase().contains("## responsibility") {
+            issues.push("persona should have a `## Responsibility` section".to_string());
+        }
+        (issues.is_empty(), issues)
     } else if body.trim().is_empty() {
```

---

### 3. Presentation — `server.rs` (wire curated system prompt into `build_deliverer`)

#### [MODIFY] server.rs

The `build_deliverer` currently uses a hardcoded system prompt: `"You are the {persona} persona..."`.
We need to look up the persona's curated `system_prompt` and use it when available.

The change requires passing the loaded personas into `build_deliverer` so it can look up the
`system_prompt` by persona name:

```diff
-fn build_deliverer(root: &Path) -> impl Fn(&str, &str, &str) -> String {
+fn build_deliverer(root: &Path) -> impl Fn(&str, &str, &str) -> String {
     let completer = build_completer(root);
+    let personas = gov::load_personas(root);
     move |persona: &str, model: &str, demand: &str| -> String {
         if let Some((runtime, provider)) = &completer {
             let mut messages = Vec::new();
-            if let Ok(system) = Message::system(format!(
-                "You are the {persona} persona on a software team. Deliver your concise contribution to the task."
-            )) {
+            // Use the persona's curated system prompt if available
+            let sys_text = personas.iter()
+                .find(|p| p.id.to_string() == persona)
+                .filter(|p| !p.system_prompt.is_empty())
+                .map(|p| p.system_prompt.clone())
+                .unwrap_or_else(|| format!(
+                    "You are the {persona} persona on a software team. Deliver your concise contribution to the task."
+                ));
+            if let Ok(system) = Message::system(sys_text) {
                 messages.push(system);
             }
```

---

### 4. Documentation — `FEATURE_MAP.md`

#### [MODIFY] FEATURE_MAP.md

Update item 2.3 to reflect the narrowed gap:

```diff
-- **What It Does Today:** YAML persona files in `maestro/personas/`, loaded and merged with defaults
-  for Two-Towers routing. Config Mode governance navigator + editor in TUI.
-- **Gap:** Schema is minimal (name + responsibility). No system prompts or tool bindings.
+- **What It Does Today:** Markdown persona files in `maestro/personas/`, loaded and merged with
+  defaults for Two-Towers routing. Config Mode governance navigator + editor in TUI. Custom
+  persona markdown supports `## System Prompt`, `## Expertise Keywords`, and `## Skill Tags`
+  sections. Curated system prompts are wired end-to-end into the deliverer pipeline. Persona
+  markdown is structurally validated on save.
+- **Gap:** No tool whitelists, temperature settings, or context window strategy per persona.
+  No visual persona editor.
```

---

## Verification Plan

### Automated Tests

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Specific new tests:
| Test | File | Validates |
|------|------|-----------|
| `parse_custom_persona_extracts_system_prompt` | `governance.rs` | Markdown → `system_prompt` field |
| `parse_custom_persona_extracts_keywords_and_tags` | `governance.rs` | Comma-separated → `Vec<String>` |
| `custom_persona_enriches_routing` | `governance.rs` | Keywords boost Two-Towers score |
| `validate_persona_requires_heading` | `governance.rs` | Missing `#` heading is an issue |
| `build_deliverer_uses_curated_prompt` | N/A (integration via existing tests) | End-to-end prompt injection |

### Manual Verification

1. Create a custom persona file `maestro/personas/data_engineer.md` with all sections
2. Run `maestro list-agents` and confirm the persona appears
3. Run `maestro run --message "build a data pipeline"` and confirm routing picks the Data Engineer

---

## Model Recommendation

**Gemini 3.1 Pro (Low)** — This is mechanical parsing enrichment and plumbing changes with clear test assertions. No architectural decisions required.
