# Project Setup & Development Guide

> Setup no longer uses an interview flow (ADR
> [0002](../adr/0002-three-mode-workspace-and-interview-removal.md)). This guide covers creating a
> Maestro project and the developer environment for contributing to Maestro itself.

## Create a project
```bash
maestro init my-project
```
The plain-CLI bootstrap collects the project **name** (required), **primary scope** (required),
**type** (optional), and any **layout reference image paths** (optional), then scaffolds:
- `maestro/config.yml`
- `maestro/scopes/` (with your primary scope written from the answers)
- `maestro/personas/` and `maestro/skills/` (default catalog)

It opens the Workspace on **Maestro Mode**. Manage everything afterward in **Config Mode** (`F1`):
create/edit/update/archive personas, skills, and scopes — defaults and customs alike. The immutable
Maestro orchestrator persona cannot be edited or archived.

## Developer environment (contributing to Maestro)
Two stacks behind one process boundary:

### Rust core
```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
scripts/quality-gate.sh
```

### Nim / Niobium TUI
```bash
scripts/install-niobium.sh      # resolve the pinned Niobium commit
cd frontend
nph --check .                   # formatting
nimble test                     # headless Niobium test-backend snapshots
```

## How the two processes connect
- `maestro run` is the headless core: it reads `TuiCommand` frames on stdin and writes `CoreEvent`
  frames on stdout (line-delimited JSON, protocol v2 — ADR
  [0003](../adr/0003-ipc-v2-mode-scoped-protocol.md)).
- The Nim `maestro_tui` binary owns the terminal, spawns `maestro run`, and renders the three-mode
  Workspace. Point it at a specific core with `MAESTRO_CORE`, or point `maestro tui` at a specific
  frontend with `MAESTRO_TUI`.

## Troubleshooting
- `maestro doctor` — verifies config load + governance scaffold.
- `maestro validate-config` — validates `config.yml` cross-references.
- Niobium is not on the nimble registry; run `scripts/install-niobium.sh` before building the TUI.
