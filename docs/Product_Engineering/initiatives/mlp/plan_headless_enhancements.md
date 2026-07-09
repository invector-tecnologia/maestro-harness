# Implementation Plan: Enhance `--no-tui` Headless Mode (Item 1.8)

## Goal

Today `maestro run` starts a full duplex IPC server that expects structured `TuiCommand` frames on stdin — designed for the Nim TUI, not for human or CI use. A CI pipeline or script has no practical way to send a demand, auto-approve it, and get structured output.

This plan adds a **`--message` flag** to `maestro run` that enables a fully non-interactive, fire-and-forget headless execution:

```bash
maestro run --message "build a cli tool"
```

This will:
1. Boot the core, auto-detect the provider, send the demand.
2. **Auto-approve** both the plan and execution gates (no human in the loop).
3. Stream structured JSON events to stdout (one per line, the existing IPC protocol).
4. Exit with a **meaningful exit code**: 0 = success, 1 = error, 2 = plan rejected/aborted.

### Why This Closes the Competitive Gap

| Competitor | Feature | Maestro After |
|---|---|---|
| Claude Code | `--json` structured output, `--print` non-interactive | `--message` + existing JSON protocol |
| Aider | `--message` for non-interactive, exit codes for CI | `--message` + exit codes 0/1/2 |
| OpenCode | Headless mode with structured JSON events | Already has structured events; now usable without TUI |

> [!IMPORTANT]
> **Model Recommendation:** Gemini 3.1 Pro (Low). The implementation is a new function in `server.rs` + a CLI variant update. No deep refactoring.

---

## User Review Required

> [!IMPORTANT]
> **Auto-approval in `--message` mode.** When `--message` is used, both the plan-approval and execution-approval gates are automatically approved. This is intentional for CI/CD pipelines. The existing `maestro run` (no `--message`) retains the full interactive IPC protocol and does **not** auto-approve.

> [!IMPORTANT]
> **Exit codes.** The new convention:
> - `0` — demand completed successfully, release persisted.
> - `1` — runtime error (config missing, provider unreachable, etc.).
> - `2` — orchestration aborted (e.g., no personas found, FSM failure).
>
> This is consistent with Unix conventions (0 = ok, 1 = general error, 2 = misuse/abort).

---

## Proposed Changes

### CLI Enum & Dispatch

#### [MODIFY] src/presentation/cli/mod.rs

**1. Add `--message` flag to `Command::Run`:**

```diff
-    /// Run the headless duplex IPC core (reads commands on stdin, writes events on stdout).
-    Run,
+    /// Run the headless duplex IPC core (reads commands on stdin, writes events on stdout).
+    Run {
+        /// Non-interactive: send a demand, auto-approve, stream JSON events, then exit.
+        #[arg(long)]
+        message: Option<String>,
+    },
```

**2. Update dispatch:**

```diff
-        Some(Command::Run) => run_core()?,
+        Some(Command::Run { message }) => run_core(message)?,
```

**3. Update `run_core()` to accept the message and delegate:**

```rust
fn run_core(message: Option<String>) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    if let Some(demand) = message {
        let code = crate::presentation::ipc::server::run_headless(&root, &demand)?;
        std::process::exit(code);
    } else {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        crate::presentation::ipc::server::run_core(&root, stdin.lock(), stdout.lock())?;
    }
    Ok(())
}
```

---

### IPC Server — Headless Runner

#### [MODIFY] src/presentation/ipc/server.rs

Add a new public function `run_headless` that drives the FSM non-interactively:

```rust
/// Run a fully non-interactive headless session for CI/CD.
///
/// Sends the demand, auto-approves all gates, streams JSON events to stdout,
/// and returns an exit code: 0 = success, 1 = error, 2 = aborted.
pub fn run_headless(root: &Path, demand: &str) -> std::io::Result<i32> {
    let mut out = std::io::stdout().lock();

    // Boot greeting
    emit(&mut out, &CoreEvent::ModeChanged { mode: Mode::Maestro })?;
    emit(&mut out, &CoreEvent::Log {
        level: "info".to_string(),
        message: format!("headless demand: {demand}"),
    })?;

    // Begin orchestration
    let pending = begin_run(root, demand, &mut out)?;
    let Some(mut run) = pending else {
        // No pending gates — unlikely but handle gracefully
        return Ok(2);
    };

    // Auto-approve plan gate
    let finished = resume_run(root, &mut run, true, &mut out)?;
    if finished {
        return Ok(0);
    }

    // Auto-approve execution gate
    if run.session.is_pending() {
        let finished = resume_run(root, &mut run, true, &mut out)?;
        if finished && run.session.is_done() {
            return Ok(0);
        }
    }

    // If still pending somehow, it's an abort
    if run.session.is_done() { Ok(0) } else { Ok(2) }
}
```

This function:
- Reuses `begin_run`, `resume_run`, and the existing `emit` functions — no duplication.
- Auto-approves both gates (plan + execution).
- Returns exit code `0` on success, `2` on abort.
- If `begin_run` or `resume_run` returns an `Err`, the `?` propagates it and `main.rs` surfaces it as exit code `1` via `anyhow`.

---

### Documentation

#### [MODIFY] docs/Product_Engineering/FEATURE_MAP.md

Update item 1.8:

```diff
-- **Status:** ✅ Implemented
+- **Status:** ✅ Implemented (enhanced)
-- **What It Does Today:** Runs core without TUI. Full duplex IPC server over stdin/stdout.
-  Human-readable `tracing` output.
+- **What It Does Today:** Runs core without TUI. Full duplex IPC server over stdin/stdout.
+  `--message` flag for fully non-interactive CI/CD: sends demand, auto-approves gates,
+  streams structured JSON events, exits with code 0/1/2. Human-readable `tracing` on stderr.
-- **Gap:** Output is human-readable tracing, not machine-parseable.
+- **Gap:** No JUnit XML test report output (future work).
```

---

## Summary of All Changes

| File | Change |
|------|--------|
| `src/presentation/cli/mod.rs` | Add `--message` to `Command::Run`, update dispatch and `run_core()` |
| `src/presentation/ipc/server.rs` | Add `run_headless()` function |
| `docs/Product_Engineering/FEATURE_MAP.md` | Update item 1.8 status and description |

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
# Test 1: Non-interactive headless run
maestro run --message "build a hello world script"
# Expected: Structured JSON events on stdout, exit code 0

# Test 2: Verify exit code
maestro run --message "build something" ; echo "exit: $?"
# Expected: exit: 0

# Test 3: Interactive IPC mode still works
echo '{"v":2,"kind":"command","name":"quit"}' | maestro run
# Expected: JSON events, then clean exit

# Test 4: Error case (no config, unreachable provider)
cd /tmp && maestro run --message "test" ; echo "exit: $?"
# Expected: JSON events with fallback, exit 0 (deterministic fallback)
```
