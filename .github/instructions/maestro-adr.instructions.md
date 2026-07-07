---
applyTo: "docs/adr/**"
description: "Architecture Decision Record format (MADR-lite) for Maestro."
---

# ADR format — Maestro (MADR-lite)

## File
- Location: `docs/adr/`. Name: `NNNN-kebab-title.md` (zero-padded, sequential).
- Status is one of: `proposed`, `accepted`, `superseded by NNNN`, `deprecated`.

## Required sections
```
# NNNN. <Title>

- Status: <proposed | accepted | ...>
- Date: <YYYY-MM-DD>
- Deciders: <who>

## Context
Why this decision is needed; the forces and constraints.

## Decision
The choice made, stated in active voice.

## Consequences
Positive, negative, and any **testable invariant** introduced.

## Alternatives considered
Each option with a one-line reason it was not chosen.
```

## Rules
- One decision per ADR. Do not edit an accepted ADR's meaning; supersede it with a new one.
- Link the relevant plan task in `docs/Maestro_Execution_Plans/tasks/`.
- Any change to the Rust↔Nim IPC contract, the FSM stages, or the governance gates requires an ADR.
