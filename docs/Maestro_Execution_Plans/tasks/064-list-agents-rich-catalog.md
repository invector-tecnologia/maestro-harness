# 064. List Agents Rich Catalog

## Objective
Enhance `maestro list-agents` to show a rich table with per-agent responsibility, model binding, provider, role (orchestrator vs operational), and handoff targets. Add `--json` for scripting.

## Implementation Steps
1. **Presentation (`src/presentation/cli/mod.rs`)**
   - Add `--json` flag to `Command::ListAgents`.
   - Replace name-only listing with a formatted table showing: Name, Role, Responsibility, Provider/Model, Handoffs.
   - If config exists, resolve per-agent bindings from `config.agents` (falling back to `system.default_provider/default_model`).
   - If `--json` is passed, serialize the agent catalog as JSON to stdout.

## Acceptance Criteria
- [ ] `maestro list-agents` prints a table with Name, Role, Provider/Model, Responsibility, Handoffs.
- [ ] `maestro list-agents --json` prints machine-readable JSON.
- [ ] Agents with explicit config bindings show their pinned provider/model.
- [ ] Agents without bindings show the system default.
- [ ] No `unwrap` in normal paths.
