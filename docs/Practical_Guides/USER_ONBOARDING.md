# Getting Started

> The interview/onboarding flow was removed (ADR
> [0002](../adr/0002-three-mode-workspace-and-interview-removal.md)). Setup is a short plain-CLI
> bootstrap; day-to-day work happens in the three-mode Workspace.

## 1. Bootstrap a project
```bash
maestro init my-project
```
`maestro init` is a plain-CLI questionnaire (no LLM). It asks for:
- **Project name** (required)
- **Primary scope** (required)
- **Type** (optional: library / Web / Desktop / Mobile)
- **Layout reference image paths** (optional — it keeps asking until you answer `No`)

It then scaffolds the governance defaults and `maestro/config.yml`, writes your answers into the
primary scope file, and opens the Workspace on **Maestro Mode**.

Prefer to configure by hand? `maestro init-config` + `maestro scaffold-markdown` create the same
files without prompts, and `maestro doctor` checks readiness.

## 2. Use the Workspace
Launch it any time with:
```bash
maestro tui
```
Switch modes from the tab header — `F1` Config, `F2` Maestro, `F3` Product (or `Tab` to cycle,
`Esc` to quit):

- **Config Mode** — view, create, edit, update, and archive config plus personas, skills, and
  scopes (defaults and customs).
- **Maestro Mode** — type a demand in the footer and press `Enter`; watch Maestro plan, delegate to
  the personas, audit, and deliver.
- **Product Mode** — pick a shipped release and watch Maestro run the built artifact live alongside
  its changelog.

## Useful commands
- `maestro list-agents` — the default persona catalog.
- `maestro validate-config` / `maestro doctor` — configuration and governance checks.
- `maestro run` — the headless core (used by the TUI; handy for scripting/CI).
