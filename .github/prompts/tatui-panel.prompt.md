---
description: "Scaffold a Maestro TUI panel as a Niobium consumer, test-first."
name: "Niobium Panel"
argument-hint: "The panel name (e.g. chat, agents, fsm, logs, projects, metrics)"
agent: "Niobium TUI"
---
Build the named TUI panel under `frontend/src/panels/`.

- Read the `niobium-frontend` skill for the panel → widget map and the Niobium API.
- Write failing golden-snapshot tests first using Niobium's test backend (no TTY).
- Compose only shipped Niobium widgets with constraint layout; keep the draw pure.
- Drive the panel from the latest core snapshot via the stdio protocol; no business logic in the TUI.
- Run the Nim quality gates from [AGENTS.md](../../AGENTS.md) until green.
