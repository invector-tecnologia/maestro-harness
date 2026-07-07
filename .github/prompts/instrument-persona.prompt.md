---
description: "Instrument a persona for a micro-project during the FSM Instrumentation stage."
name: "Instrument Persona"
argument-hint: "The micro-project and the routed persona"
agent: "Persona Instrumenter"
---
Instrument the persona for the named micro-project.

- Use the persona chosen by the Two-Towers matcher; do not pick ad-hoc.
- Draft the scoped system prompt, attach minimal skills and RAG context (cite sources).
- Emit the `.spec`/`.json` files that make the run reproducible.
- Stay within the Instrumentation stage; never trigger Execution.
