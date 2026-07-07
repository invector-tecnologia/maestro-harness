---
description: "Author the mandatory rollback plan before a micro-project's Execution gate."
name: "Plan Rollback"
argument-hint: "The micro-project cascade to protect"
agent: "Rollback Planner"
---
Author the rollback plan for the named micro-project.

- Enumerate the forward cascade steps in order.
- For each, write the inverse action and its verification.
- Flag irreversible steps and require explicit user acknowledgement.
- Order the rollback as the reverse of the forward cascade. No rollback plan ⇒ no execution.
