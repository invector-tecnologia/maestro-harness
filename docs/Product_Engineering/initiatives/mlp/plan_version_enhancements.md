# Implementation Plan: Enhance `maestro version` (Item 1.7)

## Goal

Today `maestro version` prints a bare version string (`maestro 0.1.0`). Competitors like Claude Code and Aider surface build metadata, active model info, and package versions. This plan enhances `version` to include:

1. **Build metadata** — git commit hash and build timestamp baked in at compile time via `build.rs`.
2. **Runtime context** — default provider and model from `maestro/config.yml` (if present).
3. **Toolchain** — the Rust edition declared in `Cargo.toml`.
4. **`--json` flag** — machine-readable output for CI/scripting.

### Expected output (human)

```
maestro 0.1.0 (c9b777f 2026-07-09)
  edition:  2021
  provider: ollama
  model:    llama3.1:8b-instruct-q8_0
```

### Expected output (`--json`)

```json
{
  "version": "0.1.0",
  "commit": "c9b777f",
  "build_date": "2026-07-09",
  "edition": "2021",
  "provider": "ollama",
  "model": "llama3.1:8b-instruct-q8_0"
}
```

When no config is present, the `provider` and `model` fields show `"(none)"`.

> [!IMPORTANT]
> **Model Recommendation:** Gemini 3.1 Pro (Low). Straightforward new file (`build.rs`) + a focused CLI edit. No complex refactoring.

---

## User Review Required

> [!IMPORTANT]
> **New `build.rs`:** This adds a Cargo build script that runs `git rev-parse --short HEAD` and captures the build date. This is the standard Rust approach and has zero runtime cost — the values are compile-time constants.

---

## Proposed Changes

### Build Script (compile-time metadata)

#### [NEW] build.rs

A new Cargo build script that emits two environment variables consumed via `env!()` at compile time:

- `MAESTRO_COMMIT` — short git commit hash (or `"unknown"` if not in a git repo).
- `MAESTRO_BUILD_DATE` — ISO 8601 date (`YYYY-MM-DD`).

```rust
//! Build script — bake git commit and build date into the binary.

use std::process::Command;

fn main() {
    // Git short hash
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=MAESTRO_COMMIT={commit}");

    // Build date (UTC, YYYY-MM-DD)
    let date = Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=MAESTRO_BUILD_DATE={date}");

    // Only re-run when HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
}
```

---

### CLI Enum & Dispatch

#### [MODIFY] src/presentation/cli/mod.rs

**1. Add `--json` flag to `Command::Version`:**

```diff
-    /// Print version information.
-    Version,
+    /// Print version and build information.
+    Version {
+        /// Output as JSON for scripting.
+        #[arg(long)]
+        json: bool,
+    },
```

**2. Replace the one-liner dispatch with a `print_version(root, json)` call:**

```diff
-        Some(Command::Version) => print_line(&format!("maestro {}", env!("CARGO_PKG_VERSION"))),
+        Some(Command::Version { json }) => print_version(&root, json)?,
```

**3. Add the `print_version` function:**

```rust
/// Print rich version information including build metadata and active provider/model.
fn print_version(root: &Path, json: bool) -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let commit = env!("MAESTRO_COMMIT");
    let build_date = env!("MAESTRO_BUILD_DATE");
    let edition = "2021"; // mirrors Cargo.toml edition

    // Attempt to read provider/model from config
    let (provider, model) = match crate::infrastructure::config::load_from(root) {
        Ok(cfg) => (cfg.system.default_provider, cfg.system.default_model),
        Err(_) => ("(none)".to_string(), "(none)".to_string()),
    };

    if json {
        #[derive(serde::Serialize)]
        struct VersionInfo {
            version: String,
            commit: String,
            build_date: String,
            edition: String,
            provider: String,
            model: String,
        }
        let info = VersionInfo {
            version: version.to_string(),
            commit: commit.to_string(),
            build_date: build_date.to_string(),
            edition: edition.to_string(),
            provider,
            model,
        };
        print_line(&serde_json::to_string_pretty(&info)?);
    } else {
        print_line(&format!("maestro {} ({} {})", version, commit, build_date));
        print_line(&format!("  edition:  {}", edition));
        print_line(&format!("  provider: {}", provider));
        print_line(&format!("  model:    {}", model));
    }

    Ok(())
}
```

**4. Update existing test to match new variant shape:**

```diff
-        assert!(matches!(cli.command, Some(Command::Version)));
+        assert!(matches!(cli.command, Some(Command::Version { .. })));
```

---

### Documentation

#### [MODIFY] docs/Product_Engineering/FEATURE_MAP.md

Update item 1.7 to reflect the enhanced implementation:

```diff
 ### 1.7 `maestro version` — Version Info

-- **Status:** ✅ Implemented
+- **Status:** ✅ Implemented (enhanced)
   ...
-- **What It Does Today:** Prints version string.
+- **What It Does Today:** Prints version, commit hash, build date, Rust edition, and active
+  provider/model from config. Supports `--json` for CI scripting.
-- **Gap:** Plain version only.
+- **Gap:** No opt-in update check (future work).
```

---

## Summary of All Changes

| File | Change |
|------|--------|
| `build.rs` | **[NEW]** Bake git commit hash and build date into env vars |
| `src/presentation/cli/mod.rs` | Add `--json` to `Version`, new `print_version()` fn, update test |
| `docs/Product_Engineering/FEATURE_MAP.md` | Update item 1.7 status and description |

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
# Human-readable version
maestro version
# Expected: maestro 0.1.0 (c9b777f 2026-07-09) + edition/provider/model lines

# JSON version
maestro version --json
# Expected: JSON object with version, commit, build_date, edition, provider, model

# Without config (no maestro/ dir)
cd /tmp && maestro version
# Expected: provider and model show "(none)"
```
