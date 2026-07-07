# AGENTS.md — Maestro agent operating contract

This file defines how automated agents (and humans) must work in this repository. It complements
[.github/copilot-instructions.md](.github/copilot-instructions.md), which holds the architectural
invariants.

## What Maestro is

A local-first **tactical agentic orchestrator** (an "AI harness") for disposable **micro-projects**.
Two stacks, one process boundary:

- **Rust core** — FSM, scheduler, Two-Towers routing, multi-model runtime, git/rollback governance,
  hexagonal domain. Headless; drives everything; usable with `--no-tui`.
- **Nim/Niobium TUI** — a separate process under `frontend/` that consumes
  [Niobium](https://github.com/invector-tecnologia/niobium) and renders core events over a
  line-delimited JSON stdio protocol.

## Development philosophy

**Infra-first, then spec-first.** The AI operating model (instructions, skills, agents, ADRs, plan
tasks, CI) is established before product code. Each unit of product code is driven by a task spec in
`docs/Maestro_Execution_Plans/tasks/` and a failing test before implementation.

## The reasoning loop (follow for every task)

1. **Plan** — read the relevant plan task, the ADRs in `docs/adr/`, and the manifesto docs. State
   the intended change and which FSM stage / architecture layer it touches.
2. **Red** — write or update a failing test for the right reason (Rust `#[cfg(test)]` unit test, or
   a Nim TUI golden snapshot via Niobium's test backend).
3. **Implement** — write the minimal code to pass. Respect the hard invariants and layer boundaries.
4. **Verify** — run the quality gates below, cheap → expensive. Fix and repeat on failure.
5. **Self-review** — re-check against the restrictions in `.github/instructions/`.
6. **Record** — update the plan task's evidence and any relevant ADR.

## Quality gates

### Rust core (run in this order)

```
cargo fmt --all --check                       # 1. formatting
cargo clippy --all-targets -- -D warnings     # 2. lint (no warnings)
cargo test --all-targets                      # 3. unit + integration
scripts/quality-gate.sh                        # 4. aggregate project gate
```

### Nim / Niobium TUI (run in this order)

```
scripts/install-niobium.sh                    # 1. resolve the TUI dependency (pinned niobium commit; not on the nimble registry yet)
nph --check frontend                          # 2. formatting
nimble test                                   # 3. TUI tests (Niobium test backend, no TTY)
```

## Definition of Done

- All applicable quality gates pass (Rust and/or Nim, depending on scope).
- Change maps to exactly one plan task; acceptance criteria validated with evidence.
- Architecture boundaries respected; the Rust↔Nim coupling stays only in the IPC protocol.
- No `unwrap`/`expect`/`panic!` in normal Rust paths; all logging via `tracing`.
- Behavior-changing work documents its rollback and risk notes.

## Toolchain

- Rust: stable toolchain, `cargo fmt` + `cargo clippy`, Tokio async runtime.
- Nim ≥ 2.0.0 (`--mm:orc`), formatter `nph`, TUI via Niobium pinned to the exact v0.1.0 commit (`scripts/install-niobium.sh`).
- Never bypass gates (no skipping clippy/fmt, no `--no-verify`).
