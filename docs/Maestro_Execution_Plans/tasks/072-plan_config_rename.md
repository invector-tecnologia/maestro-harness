# Implementation Plan: Replace `init-config` with `maestro config`

## Goal

Replace the `maestro init-config [--governance]` command with a simpler `maestro config` that **always** writes `config.yml` and scaffolds governance folders (`scopes/`, `personas/`, `skills/`) in a single step. The `--governance` flag becomes unnecessary because governance scaffolding is now the default behavior.

### Why

- **Simpler mental model:** One command, one purpose — "set up Maestro in this directory."
- **Governance-first by design:** Maestro's governed execution model means governance folders are always required. Making them optional was an unnecessary indirection.
- **Shorter command name:** `maestro config` is more natural than `maestro init-config`.

> [!IMPORTANT]
> **Model Recommendation:** Claude Opus 4.6 (current). Code-editing-heavy refactor touching the CLI enum, dispatch, providers module, interactive wizard, tests, and documentation.

---

## User Review Required

> [!WARNING]
> **Breaking Change:** `maestro init-config` will no longer be recognized as a valid subcommand. All scripts/docs referencing it must switch to `maestro config`.

> [!IMPORTANT]
> **`maestro init` (full wizard) still works.** It calls `scaffold_project()` internally, which uses the underlying `scaffold_markdown()` and `init_config()` helpers directly. This path is **not** affected by the CLI rename.

---

## Proposed Changes

### CLI Enum & Dispatch

#### [MODIFY] src/presentation/cli/mod.rs

**1. Rename `InitConfig` → `Config` in the `Command` enum, remove `--governance` flag:**

```diff
 pub enum Command {
     Version,
     ValidateConfig { ... },
     ListAgents { ... },
-    /// Generate `maestro/config.yml` from a template.
-    InitConfig {
+    /// Set up Maestro: generate config and governance scaffold.
+    Config {
         #[arg(long)]
         provider: Option<String>,
         #[arg(long)]
         endpoint: Option<String>,
         #[arg(long)]
         model: Option<String>,
-        #[arg(long)]
-        governance: bool,
     },
     Doctor,
     ...
 }
```

**2. Update dispatch match arm:**

```diff
-        Some(Command::InitConfig {
+        Some(Command::Config {
             provider,
             endpoint,
             model,
-            governance,
         }) => {
-            let result = providers::init_config_with_provider(&root, provider, endpoint, model, governance)?;
+            let result = providers::init_config_with_provider(&root, provider, endpoint, model)?;
             print_line(&format!("wrote {}", result.path.display()));
             if !result.governance_created.is_empty() {
                 print_line(&format!(
                     "scaffolded governance: {}",
                     result.governance_created.join(", ")
                 ));
             }
             ...
         }
```

**3. Update interactive wizard (choice "5") label from "Scaffold Governance" to "Config":**

```diff
-            let _ = writeln!(out, "5) Scaffold Governance");
+            let _ = writeln!(out, "5) Config (setup)");
```

The wizard dispatch at choice "5" already calls `init_config_with_provider(..., true)` — this just needs the `governance` argument removed since governance is now always-on.

---

### Providers Module

#### [MODIFY] src/presentation/cli/providers.rs

**1. Remove `governance: bool` parameter — governance is now always-on:**

```diff
 pub fn init_config_with_provider(
     root: &Path,
     provider: Option<String>,
     endpoint: Option<String>,
     model: Option<String>,
-    governance: bool,
 ) -> anyhow::Result<InitConfigResult> {
     ...
-    let governance_created = if governance {
-        crate::presentation::cli::scaffold_markdown(root)?
-    } else {
-        Vec::new()
-    };
+    let governance_created = crate::presentation::cli::scaffold_markdown(root)?;
```

**2. Update doc comments to reference `maestro config` instead of `maestro init-config`.**

---

### Tests

#### [MODIFY] src/presentation/cli/mod.rs (test section)

The existing `init_config_writes_template` test uses the internal `init_config()` helper and is unaffected.

No test currently exercises the CLI `InitConfig` variant directly (it's tested manually), so no test renames needed.

---

### Documentation

#### [MODIFY] docs/Product_Engineering/FEATURE_MAP.md

- Item 1.2: Rename from `maestro init-config` to `maestro config`.
- Item 1.6: Already merged; update source reference.

---

## Summary of All Changes

| File | Change |
|------|--------|
| `src/presentation/cli/mod.rs` | Rename `InitConfig` → `Config`, remove `--governance` flag, update dispatch and wizard |
| `src/presentation/cli/providers.rs` | Remove `governance: bool` param, always scaffold |
| `docs/Product_Engineering/FEATURE_MAP.md` | Update items 1.2 and 1.6 |

---

## Verification Plan

### Automated Tests

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

### Manual Verification

```bash
# Test 1: `maestro config` creates config + governance
tmp=$(mktemp -d) && cd $tmp && maestro config && ls -l maestro && rm -rf $tmp
# Expected: config.yml, scopes/, personas/, skills/ all present

# Test 2: `maestro init-config` is no longer valid
maestro init-config
# Expected: error about unrecognized subcommand

# Test 3: `maestro config --provider ollama` works
tmp=$(mktemp -d) && cd $tmp && maestro config --provider ollama && ls -l maestro && rm -rf $tmp
```
