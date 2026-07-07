# 0002. Three-mode Workspace (Config · Maestro · Product) and removal of interview mode

- Status: accepted
- Date: 2026-07-07
- Deciders: Maestro maintainers

## Context
The 0.1.0 core is complete and green, but the interactive experience was only ever *planned*, not
built. That plan (tasks 017–032 onboarding, 043–045 LLM-driven interview, 035/040 "Architect Mode")
centred on a conversational **interview mode** that adaptively interrogates the user before work can
begin. This conflicts with Maestro's doctrine: the user is a **validator of plans**, not a subject of
a Q&A funnel. The interview machinery is large (≈19 tasks, an onboarding state machine, resume,
telemetry) and delays the product's identity — orchestrating micro-projects.

We need a single, predictable interactive surface that (a) lets the user govern configuration and
personas, (b) lets the user watch Maestro orchestrate work, and (c) lets the user see shipped results.

## Decision
Replace all interview/onboarding UX with one **Workspace** application exposing exactly three modes,
switchable via a Tatui `Tabs` header:

- **Config Mode** — the single governance surface. View, create, edit, update, and **archive** both
  **defaults** and **customs** of `maestro/config.yml` and the governance markdown (personas /
  instructions, skills, project scopes). This absorbs the former "Architect Mode".
- **Maestro Mode** — the user submits a demand to the **Maestro** persona, which plans and delegates
  to the project's default and custom personas; the user monitors what Maestro demands and what the
  personas build (narration, per-persona activity, FSM stage, approval gates).
- **Product Mode** — Maestro presents a live demo of the shipped project: it runs the built artifact
  and streams its output while showing the release notes/changelog for each shipped release.

The only remaining bootstrap is a **plain-CLI, non-LLM** `maestro init [<name>]` questionnaire
(project name, primary scope, optional type, optional layout-reference image paths). It scaffolds
defaults, writes the answers into the default files, then opens the Workspace on Maestro Mode.

## Consequences
- **Positive:** one coherent mental model; no conversational funnel; faster path to the orchestration
  core; Config Mode centralises all governance CRUD; Product Mode gives visible, shippable value.
- **Negative:** tasks 017–032 and 043–045 are retired and their partial doctrine (onboarding state
  machine, telemetry) is dropped; `maestro interview`/`onboarding` commands never ship.
- **Testable invariant:** the Workspace exposes exactly the three modes above and no interview flow;
  `maestro init` performs no LLM calls; the only interactive setup is the plain-CLI questionnaire.

## Alternatives considered
- **Keep the dual-engine interview (043–045):** rejected — contradicts the "validator, not subject"
  doctrine and front-loads cost before the orchestration engine exists.
- **Separate Architect Mode alongside Config Mode:** rejected — two governance surfaces confuse
  ownership; a single Config Mode with defaults/customs and archive covers the need.
- **Two modes (fold Product into Maestro):** rejected — presenting shipped releases is a distinct
  audience/task (validation of outcomes) and deserves its own mode.
