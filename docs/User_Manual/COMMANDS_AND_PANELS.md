# Maestro Commands and Panels

> Updated for the three-mode Workspace (ADR
> [0002](../adr/0002-three-mode-workspace-and-interview-removal.md)). Interview / onboarding mode has
> been removed; setup is a plain-CLI `maestro init` bootstrap.

## CLI Commands
- `maestro init [<project-name>]`: Plain-CLI bootstrap (no LLM). Prompts for project **name**
  (required), **primary scope** (required), **type** (optional: library/Web/Desktop/Mobile), and
  **layout reference image paths** (optional, repeated until you answer `No`). Scaffolds the
  governance defaults + `maestro/config.yml`, writes the answers into the primary scope file, then
  opens the Workspace on **Maestro Mode**.
- `maestro tui`: Launches the Nim/Tatui Workspace TUI (which spawns the headless core).
- `maestro run`: Runs the headless duplex core — reads `TuiCommand` frames on stdin and writes
  `CoreEvent` frames on stdout (protocol v2). This is what the TUI drives; also useful for scripting.
- `maestro validate-config`: Validates `maestro/config.yml` and its cross-references.
- `maestro list-agents`: Lists the registered default personas.
- `maestro doctor`: Checks config and governance-scaffold readiness.
- `maestro scaffold-markdown`: Creates the `scopes` / `personas` / `skills` governance folders.
- `maestro init-config`: Writes a starter `maestro/config.yml`.
- `maestro --no-tui …`: Global flag to stay headless.

## Workspace Modes
The Workspace is a single application with **three modes**, switched from the `Tabs` header
(`F1`/`F2`/`F3`, or `Tab` to cycle). `Esc` quits.

### Config Mode (F1)
The single governance surface. View, create, edit, update, and **archive** both **defaults** and
**customs** of everything Maestro reads:
- `maestro/config.yml` (system, providers, models, agent bindings),
- personas (instructions) under `maestro/personas/`,
- skills under `maestro/skills/`,
- project scopes under `maestro/scopes/`.

Layout: a governance **navigator** (left) listing entries with their `origin` (default/custom) and
archive state, and an **editor** (right) for the selected entry. Archiving is a soft delete
(entries move under `maestro/archive/`); the immutable Maestro orchestrator persona can never be
edited or archived.

### Maestro Mode (F2)
The orchestration monitor. Submit a demand and the **Maestro** persona plans it, delegates to the
project's default and custom personas, audits their work, and delivers — while you watch.
Layout: a **Personas** panel (left, per-persona cognitive state: `idle`/`observe`/`think`/`act`/
`error`) and a **Maestro** panel (right, the current FSM stage plus live narration: plan →
delegate → audit → deliver, with heartbeats during long work). Approval gates surface here.

### Product Mode (F3)
The live demo. For each shipped release, Maestro runs the built artifact and streams its output.
Layout: **Releases & Notes** (left, the release list plus the selected release's changelog) and
**Live Demo** (right, the running artifact's stdout/stderr and exit status).

## Command Footer
Every mode shows a command footer. Type a demand and press `Enter` to send it as `user_input`;
prefix with `/` to send a slash command (e.g. `/quit`). Mode switches also notify the core via
`switch_mode` so it stays in sync.

## Persona source of truth
Personas live as canonical markdown under `maestro/personas/`. Maestro Mode loads its agent set from
these governed files, so editing a persona in Config Mode changes the live team. Missing/invalid
files fall back safely to the built-in default catalog. `maestro scaffold-markdown` emits the schema
and the immutable Maestro orchestrator is always present.
