---
name: nim-upstream
description: >
  Authoritative reference to the upstream Nim language, compiler, and standard
  library (github.com/nim-lang/Nim). Use when you need ground-truth on Nim std
  modules, --mm:orc semantics, syntax, or pragmas for Maestro's TUI frontend.
---

# Nim Upstream Skill

Ground-truth reference for the Nim language and standard library used by Maestro's `frontend/`.

## Authoritative source
- Repo: `git@github.com:nim-lang/Nim.git` — https://github.com/nim-lang/Nim
- Use it (via `github_repo` / `fetch_webpage`) to confirm `std/*` module APIs (`json`, `strutils`,
  `os`, `streams`, `unittest`), pragma behavior, and `--mm:orc` memory semantics.

## When to apply
- Verifying a Nim `std` API signature or a language construct (templates, macros, `proc`/`func`).
- Questions about ORC memory management, value vs ref types, or `defer` lifecycle.
- Formatting/build questions: `nph`, `nimble`, Nim ≥ 2.0 features.

## How to use it in Maestro
- Prefer local conventions first: the `tatui-frontend` skill and `.github/instructions/nim-frontend`.
- The frontend targets Nim ≥ 2.0 with `--mm:orc`; ignore pre-2.0 or GC-specific upstream guidance.
- Cite exact upstream paths/permalinks when relying on upstream behavior.

## Guardrails
- The TUI is a thin renderer: no orchestration logic in Nim; the draw is a pure function of state.
- Frontend Nim only *consumes* Tatui widgets; it does not reimplement TUI primitives.
