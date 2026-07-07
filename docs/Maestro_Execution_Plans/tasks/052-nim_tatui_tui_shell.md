# TASK 052: Nim/Niobium TUI Shell

## 1. TASK SIGNATURE (DSPy Architecture)
* **Inputs:** IPC stdio protocol, Niobium dependency.
* **Context Anchors:** #file:.github/skills/niobium-frontend/SKILL.md, #file:.github/instructions/nim-frontend.instructions.md
* **Expected Output:** A `frontend/` Nim process rendering Maestro state via Niobium.

## 2. ABSOLUTE CONSTRAINTS (1.58-bit Constraint)
* Consume Niobium only (`requires "niobium >= 0.1.0"`); never re-implement TUI primitives.
* The draw function is a pure function of the latest core snapshot.
* The TUI talks to the core only over the stdio protocol; terminal state restored via `defer`.

## 3. EXECUTION PROMPT (Paste into Copilot Chat)
"""
Act as a Terminal-First Product Engineer.
Goal: Build the Nim/Niobium TUI shell: tick loop, protocol client, and the base panel layout.

Before generating code, open a `<reasoning>` block and model the event flow: read core events → update snapshot → draw frame → forward input.

Execute:
1. Scaffold `frontend/maestro_tui.nimble` requiring `niobium >= 0.1.0`.
2. Implement `frontend/src/app.nim` (tick loop) and `frontend/src/protocol.nim` (stdio client).
3. Compose the base layout with the panel → widget map (chat, agents, FSM stepper, logs, projects, metrics).
4. Add golden-snapshot tests using Niobium's test backend (no TTY).

[Cohesion Mechanism]:
- Confirm the TUI holds no orchestration logic.

Return ONLY the modified code blocks in Markdown. No introduction.
"""

## 4. Acceptance Criteria
* **AC1:** The TUI builds against `niobium >= 0.1.0` and runs headless in tests.
* **AC2:** Panels render from a core snapshot and are covered by test-backend snapshots.
* **AC3:** All core coupling flows through the stdio protocol client.
