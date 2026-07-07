---
description: "Scaffold a Maestro TUI panel as a Tatui consumer, test-first."
name: "Tatui Panel"
argument-hint: "The panel name (e.g. chat, agents, fsm, logs, projects, metrics)"
agent: "Tatui TUI"
---
Build the named TUI panel under `frontend/src/panels/`.

- Read the `tatui-frontend` skill for the panel → widget map and the Tatui API.
- Write failing golden-snapshot tests first using Tatui's test backend (no TTY).
- Compose only shipped Tatui widgets with constraint layout; keep the draw pure.
- Drive the panel from the latest core snapshot via the stdio protocol; no business logic in the TUI.
- Run the Nim quality gates from [AGENTS.md](../../AGENTS.md) until green.
