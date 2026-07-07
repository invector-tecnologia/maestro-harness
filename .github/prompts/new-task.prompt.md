---
description: "Create a new Maestro execution-plan task following the spec-first task template."
name: "New Task"
argument-hint: "The task title and short context"
agent: "Plan Author"
---
Create a new task under `docs/Maestro_Execution_Plans/tasks/` following the existing task format.

- Pick the next `NNN` number after the highest existing task.
- File name: `NNN-kebab_title.md`.
- Use the TASK SIGNATURE / ABSOLUTE CONSTRAINTS / EXECUTION PROMPT structure of neighbouring tasks.
- Add numbered acceptance criteria (AC1, AC2, …) that a test or scripted check can validate.
- State the FSM stage and architecture layer touched; note rollback/risk for behavior changes.
