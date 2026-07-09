# 067. Enhance `maestro version` (Item 1.7)

## Objective
Enhance `maestro version` to include build metadata (commit hash, build date, toolchain) and runtime context (default provider/model) with a `--json` output option, matching competitor capabilities (Claude Code, Aider).

## Implementation Steps
1. **Build Metadata (`build.rs`)**:
   - Create a `build.rs` script to inject `MAESTRO_COMMIT` and `MAESTRO_BUILD_DATE` environment variables at compile time.

2. **CLI & Dispatch (`src/presentation/cli/mod.rs`)**:
   - Add a `json: bool` flag to the `Command::Version` variant.
   - Create a `print_version(root: &Path, json: bool)` function that reads compile-time env vars and the current config (if present).
   - Format output nicely for humans, or as structured JSON if `--json` is passed.
   - Update `cli::tests::defaults_to_no_subcommand` to match `Command::Version { .. }`.

3. **Documentation (`docs/Product_Engineering/FEATURE_MAP.md`)**:
   - Update item 1.7 status to "Implemented (enhanced)".
   - Note the inclusion of commit hash, build date, Rust edition, active provider/model, and JSON output.

## Acceptance Criteria
- [ ] `maestro version` prints the detailed human-readable summary.
- [ ] `maestro version --json` prints structured JSON.
- [ ] Output falls back gracefully when `maestro/config.yml` is missing.
- [ ] All tests pass.
