---
description: "Create a new Architecture Decision Record for Maestro following the MADR-lite format."
name: "New ADR"
argument-hint: "The decision title and short context"
agent: "agent"
---
Create a new ADR under `docs/adr/` following [the ADR format](../instructions/maestro-adr.instructions.md).

- Pick the next `NNNN` number after the highest existing ADR in `docs/adr/`.
- File name: `NNNN-kebab-title.md`.
- Fill Context, Decision, Consequences (including any testable invariant introduced), and
  Alternatives considered.
- Link the relevant plan task in `docs/Maestro_Execution_Plans/tasks/`.
- Set status to `proposed` unless told otherwise.
