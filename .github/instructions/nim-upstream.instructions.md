---
applyTo: "frontend/**/*.nim"
description: "Reference the upstream Nim language/std (github.com/nim-lang/Nim) as ground-truth for std modules, --mm:orc semantics, and syntax when local sources are insufficient."
---

# Nim Upstream Reference

- Authoritative source: https://github.com/nim-lang/Nim (`git@github.com:nim-lang/Nim.git`).
- Confirm `std/*` APIs, pragmas, and `--mm:orc` semantics upstream before relying on memory; cite
  exact permalinks.
- Target Nim ≥ 2.0 with `--mm:orc`; ignore pre-2.0 or GC-specific upstream guidance.
- Local doctrine wins: the `tatui-frontend` skill and `nim-frontend.instructions.md` take
  precedence. Keep the TUI a thin, pure renderer of core state.
