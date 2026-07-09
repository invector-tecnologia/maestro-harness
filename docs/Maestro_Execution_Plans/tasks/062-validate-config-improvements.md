# 062. Validate Config Improvements

## Objective
Enhance `maestro validate-config` with inline error suggestions, an auto-repair `--fix` flag, and active remote provider probing.

## Implementation Steps
1. **Domain (`src/domain/models/config.rs`)**
   - Add `.suggestion()` to `ConfigError` to provide helpful hints.
   - Add `.repair()` to `MaestroConfig` to auto-fix dangling defaults and invalid agents.

2. **Infrastructure (`src/infrastructure/config.rs`)**
   - Add `save_to` to save `MaestroConfig` back to `config.yml`.

3. **Presentation (`src/presentation/cli/mod.rs`)**
   - Add `--fix` to `Command::ValidateConfig`.
   - Update `validate_config` to attempt repair if `--fix` is passed and `config.validate()` fails.
   - Run active network probes for all declared providers using `tokio::runtime::Runtime::new()`.

## Acceptance Criteria
- [ ] Broken config fails validation and prints a helpful suggestion.
- [ ] `validate-config --fix` repairs a dangling default provider (if only 1 exists) or model.
- [ ] Config rewritten by `--fix` passes validation on the next run.
- [ ] Command outputs the active network probe status for all configured providers.
- [ ] No `unwrap` in normal paths.
