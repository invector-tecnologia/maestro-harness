# 065. Merge `scaffold-markdown` into `init-config`

## Objective
Merge the standalone `maestro scaffold-markdown` command into `maestro init-config` as a `--governance` flag to streamline project setup and reduce cognitive load.

## Implementation Steps
1. **Providers Module (`src/presentation/cli/providers.rs`)**:
   - Update `InitConfigResult` to include `governance_created: Vec<String>`.
   - Update `init_config_with_provider` to take a `governance: bool` argument.
   - If `governance` is true, call `crate::presentation::cli::scaffold_markdown(root)` and store the result.

2. **CLI & Dispatch (`src/presentation/cli/mod.rs`)**:
   - Remove `ScaffoldMarkdown` variant from `Command`.
   - Add `governance: bool` argument to `Command::InitConfig` with `--governance` flag.
   - Remove `Command::ScaffoldMarkdown` match arm in `dispatch`.
   - Update `Command::InitConfig` match arm to pass `governance` to `init_config_with_provider` and print governance directories if created.
   - Update interactive wizard (choice "5") to call `init_config_with_provider(root, None, None, None, true)` instead of just `scaffold_markdown`.

3. **Documentation (`docs/Product_Engineering/FEATURE_MAP.md`)**:
   - Update item 1.6 to reflect it has been merged into 1.2.

## Acceptance Criteria
- [ ] `maestro scaffold-markdown` command is removed.
- [ ] `maestro init-config --governance` creates both config and governance folders.
- [ ] `maestro init` wizard choice 5 creates both config and governance folders.
- [ ] All tests pass.
