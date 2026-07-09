# Implementation Plan: Complete Project Templates (Item 1.9)

## Goal

The FEATURE_MAP calls for a "library of templates (web app, API service, CLI tool, data pipeline, infra module) with pre-configured personas, skills, and scope documents." Today we have 4 templates (`web-app`, `cli-tool`, `library`, `infra`) that ship a scope document and a task spec, but they're missing:

1. **Two templates explicitly called out**: `api-service` and `data-pipeline`.
2. **Per-template persona hints** — a brief system-prompt fragment that tailors the default personas to the project type (e.g., for `web-app`, the UX persona gets extra emphasis; for `infra`, security/idempotency is emphasized).
3. **Per-template skill content** — a starter skill file that seeds the `maestro/skills/` directory with domain-relevant prompts.
4. **`--json` output for `list-templates`** — machine-readable catalog for CI tooling.

This plan **does not** implement a community template registry (that's a future network feature). All content remains baked into the binary, consistent with the local-first philosophy.

> [!IMPORTANT]
> **Model Recommendation:** Gemini 3.1 Pro (Low). This is primarily additive string content in `templates.rs` plus a small CLI flag change. No architectural refactoring.

---

## User Review Required

> [!IMPORTANT]
> **Template content is opinionated.** The persona hints and skill starters embed best-practice guidance for each project type. Please review the content in the "Template Content" section below.

---

## Proposed Changes

### Template Data Model

#### [MODIFY] src/presentation/cli/templates.rs

**1. Extend `ProjectTemplate` with two new fields:**

```diff
 pub struct ProjectTemplate {
     pub key: &'static str,
     pub description: &'static str,
     pub scope_content: &'static str,
     pub task_spec: &'static str,
+    /// Optional persona hint (written to `maestro/scopes/_persona_hints.md`).
+    pub persona_hints: &'static str,
+    /// Optional starter skill content (written to `maestro/skills/<key>.md`).
+    pub skill_content: &'static str,
 }
```

**2. Add two new templates** (`api-service` and `data-pipeline`) to the `TEMPLATES` array.

**3. Enrich existing templates** with `persona_hints` and `skill_content`.

#### Template Content

| Template | Persona Hints (summary) | Skill (summary) |
|---|---|---|
| `web-app` | UX persona: responsive design, Lighthouse, a11y. Engineer: frontend+backend separation. | Skill: Frontend Testing (Lighthouse, visual regression) |
| `cli-tool` | Engineer: argument parsing, exit codes, cross-platform. QA: edge-case CLI inputs. | Skill: CLI UX (help text, error messages, shell completion) |
| `library` | Engineer: public API surface, semver, documentation. QA: coverage ≥ 80%. | Skill: API Design (backward compat, naming, documentation) |
| `infra` | Engineer: idempotency, dry-run, secrets handling. QA: drift detection. | Skill: Infrastructure Safety (plan-then-apply, rollback, audit) |
| `api-service` | Engineer: REST/gRPC, auth, rate limiting. QA: contract testing. UX: developer experience (SDK, docs). | Skill: API Contract Testing (OpenAPI, schema validation) |
| `data-pipeline` | Engineer: ETL stages, idempotency, schema evolution. QA: data quality checks. | Skill: Data Quality (schema validation, null checks, dedup) |

---

### Scaffold Integration

#### [MODIFY] src/presentation/cli/mod.rs

Update `scaffold_project` to write the new template fields:

```diff
     if let Some(tmpl) = template {
         // Write template-enriched scope
         std::fs::write(&scope_path, tmpl.scope_content)?;
         created.push(format!("scopes/{slug}.md"));

         // Write starter task spec
         let tasks_dir = root.join("maestro").join("tasks");
         std::fs::create_dir_all(&tasks_dir)?;
         let task_path = tasks_dir.join("001_initial_setup.md");
         std::fs::write(&task_path, tmpl.task_spec)?;
         created.push("tasks/001_initial_setup.md".to_string());
+
+        // Write persona hints if provided
+        if !tmpl.persona_hints.is_empty() {
+            let hints_path = root.join("maestro").join("scopes").join("_persona_hints.md");
+            std::fs::write(&hints_path, tmpl.persona_hints)?;
+            created.push("scopes/_persona_hints.md".to_string());
+        }
+
+        // Write starter skill if provided
+        if !tmpl.skill_content.is_empty() {
+            let skills_dir = root.join("maestro").join("skills");
+            std::fs::create_dir_all(&skills_dir)?;
+            let skill_path = skills_dir.join(format!("{}.md", tmpl.key));
+            std::fs::write(&skill_path, tmpl.skill_content)?;
+            created.push(format!("skills/{}.md", tmpl.key));
+        }
     }
```

---

### CLI — `list-templates --json`

#### [MODIFY] src/presentation/cli/mod.rs

**1. Add `--json` flag to `Command::ListTemplates`:**

```diff
-    /// List available project templates.
-    ListTemplates,
+    /// List available project templates.
+    ListTemplates {
+        /// Output as JSON for CI scripting.
+        #[arg(long)]
+        json: bool,
+    },
```

**2. Update dispatch:**

```diff
-        Some(Command::ListTemplates) => {
-            print_line("Available templates:");
-            for (key, desc) in templates::list() {
-                print_line(&format!("  {key:<12} — {desc}"));
-            }
-        }
+        Some(Command::ListTemplates { json }) => {
+            if json {
+                let items: Vec<serde_json::Value> = templates::list()
+                    .into_iter()
+                    .map(|(k, d)| serde_json::json!({"key": k, "description": d}))
+                    .collect();
+                print_line(&serde_json::to_string_pretty(&items).unwrap_or_default());
+            } else {
+                print_line("Available templates:");
+                for (key, desc) in templates::list() {
+                    print_line(&format!("  {key:<12} — {desc}"));
+                }
+            }
+        }
```

---

### Documentation

#### [MODIFY] docs/Product_Engineering/FEATURE_MAP.md

Update item 1.9:

```diff
-- **Status:** 📋 Planned
-- **Source:** Not implemented
+- **Status:** ✅ Implemented
+- **Source:** `src/presentation/cli/templates.rs`, `src/presentation/cli/mod.rs`
-- **What It Does Today:** Nothing.
+- **What It Does Today:** 6 built-in templates (web-app, cli-tool, library, infra, api-service,
+  data-pipeline) with pre-configured scope docs, task specs, persona hints, and starter skills.
+  `maestro list-templates --json` for CI scripting.
-- **Gap:** Full feature gap.
+- **Gap:** No community template registry (future network feature).
```

---

## Summary of All Changes

| File | Change |
|------|--------|
| `src/presentation/cli/templates.rs` | Add `persona_hints` and `skill_content` fields. Add `api-service` and `data-pipeline` templates. Enrich all 6 templates. |
| `src/presentation/cli/mod.rs` | Write persona hints + skill files during scaffold. Add `--json` to `ListTemplates`. |
| `docs/Product_Engineering/FEATURE_MAP.md` | Update item 1.9 status and description. |

---

## Verification Plan

### Automated Tests

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Existing tests in `templates.rs` will catch regressions (all templates must have non-empty content). We'll also extend the tests:
- `find` returns the two new templates (`api-service`, `data-pipeline`).
- `list` returns 6 templates.
- All templates have non-empty `persona_hints` and `skill_content`.

### Manual Verification

```bash
# Test 1: Init with new template
maestro init my-api --template api-service --no-tui
ls my-api/maestro/scopes/ my-api/maestro/skills/ my-api/maestro/tasks/
# Expected: scope file, _persona_hints.md, api-service.md skill, 001_initial_setup.md

# Test 2: Init with data-pipeline template
maestro init my-pipeline --template data-pipeline --no-tui
ls my-pipeline/maestro/scopes/ my-pipeline/maestro/skills/ my-pipeline/maestro/tasks/

# Test 3: JSON output
maestro list-templates --json
# Expected: JSON array with 6 objects

# Test 4: Human-readable output
maestro list-templates
# Expected: 6 rows
```
