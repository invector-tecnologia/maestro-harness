# TASK 060: Project Templates & Auto-Detection for `maestro init`

## 1. TASK SIGNATURE
* **Inputs:** `src/presentation/cli/mod.rs` (existing init flow)
* **Context Anchors:** #file:docs/FEATURE_MAP.md (Domain 1, item 1.1)
* **Expected Output:** Enhanced `maestro init` with template gallery,
  project auto-detection, starter task specs, and `--template <name>` flag.

## 2. ABSOLUTE CONSTRAINTS
* Templates are baked into the binary (no network fetch).
* `maestro init` still makes NO LLM calls.
* `--template` bypasses interactive prompts entirely.
* Auto-detection is best-effort; never blocks or errors on failure.
* Existing tests continue to pass unchanged.

## 3. ACCEPTANCE CRITERIA
* AC1: `maestro list-templates` prints ≥ 4 templates with descriptions.
* AC2: `maestro init --template web-app` creates governance folders + scope
  file + starter task spec non-interactively.
* AC3: Running `maestro init` inside a directory with `Cargo.toml` auto-
  suggests the directory name as project name and prints detection info.
* AC4: Template scope files contain meaningful boundary/constraint content
  (not placeholder text).
* AC5: All new functions have unit tests. All existing tests pass.

## 4. RISKS
* Adding templates increases binary size (negligible — ~5KB of embedded strings).
* Template content may not match all project variations (mitigated: templates
  are starting points, not constraints).

## 5. ROLLBACK
* Revert the three new/modified files. No database or external state.
