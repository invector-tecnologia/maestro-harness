---
name: rust-upstream
description: >
  Authoritative reference to the upstream Rust language and standard library
  (github.com/rust-lang/rust). Use when you need ground-truth on std APIs,
  language semantics, edition behavior, Tokio-adjacent patterns, or when a
  Maestro Rust question cannot be answered from local code alone.
---

# Rust Upstream Skill

Ground-truth reference for the Rust language and standard library used by Maestro's core.

## Authoritative source
- Repo: `git@github.com:rust-lang/rust.git` — https://github.com/rust-lang/rust
- Use it (via `github_repo` / `fetch_webpage`) to confirm std API signatures, stabilization status,
  edition semantics, and idiomatic patterns before relying on memory.

## When to apply
- Verifying a `std` / `core` / `alloc` API shape, trait bound, or MSRV/edition behavior.
- Resolving borrow-checker, lifetime, async-trait, or `?`-propagation questions.
- Choosing idiomatic error types (`thiserror` in domain/application, `anyhow` at the CLI boundary).

## How to use it in Maestro
- Prefer local conventions first: `docs/Maestro_Manifesto/CONVENTIONS.md` and the `rust` skill.
- Only reach upstream when local sources are insufficient; cite the exact upstream path/permalink.
- Never copy nightly-only or unstable APIs into Maestro — stable toolchain only.

## Guardrails
- No `unwrap`/`expect`/`panic!` in normal paths; `Arc<tokio::sync::RwLock<T>>` for shared async state.
- Keep `domain/` free of I/O and provider SDKs regardless of upstream examples.
