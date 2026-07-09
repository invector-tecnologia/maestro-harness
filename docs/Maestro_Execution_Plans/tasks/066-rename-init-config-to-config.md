# 066. Rename `init-config` to `config`

## Objective
Replace the `maestro init-config [--governance]` command with a simpler `maestro config` that always writes `config.yml` and scaffolds governance folders (`scopes/`, `personas/`, `skills/`) in a single step.

## Implementation Steps
1. **Providers Module (`src/presentation/cli/providers.rs`)**:
   - Remove `governance: bool` argument from `init_config_with_provider`.
   - Update `init_config_with_provider` to always call `crate::presentation::cli::scaffold_markdown(root)` unconditionally.
   - Update doc comments referencing `init-config` to `config`.

2. **CLI & Dispatch (`src/presentation/cli/mod.rs`)**:
   - Rename `Command::InitConfig` to `Command::Config`.
   - Remove the `governance: bool` argument from `Command::Config`.
   - Update the `dispatch` match arm from `Some(Command::InitConfig { ... })` to `Some(Command::Config { ... })` and remove `governance` propagation.
   - Update interactive wizard choice "5" label to "Config (setup)".
   - Update interactive wizard choice "5" action to remove the boolean argument from `init_config_with_provider`.

3. **Documentation (`docs/Product_Engineering/FEATURE_MAP.md`)**:
   - Update item 1.2 to reflect the new command name `maestro config`.
   - Update item 1.6 to reflect the changes.

## Acceptance Criteria
- [ ] `maestro config` creates both config and governance folders.
- [ ] `maestro init-config` command is removed and errors as unrecognized.
- [ ] All tests pass.
