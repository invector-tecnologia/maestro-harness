---
applyTo: "src/**/*.rs"
description: "Reference the upstream Rust language/std (github.com/rust-lang/rust) as ground-truth for std APIs, editions, and language semantics when local sources are insufficient."
---

# Rust Upstream Reference

- Authoritative source: https://github.com/rust-lang/rust (`git@github.com:rust-lang/rust.git`).
- Confirm `std`/`core`/`alloc` API shapes, stabilization, and edition semantics upstream before
  relying on memory; cite exact permalinks.
- Stable toolchain only — never adopt nightly/unstable APIs into Maestro.
- Local doctrine wins: `docs/Maestro_Manifesto/CONVENTIONS.md`, the `rust` skill, and
  `rust-companion.instructions.md` take precedence over upstream examples.
