#!/usr/bin/env bash
# Aggregate quality gate for Maestro. Runs the cheap → expensive checks for whichever
# stacks are present (Rust core and/or Nim/Tatui TUI). Skips a stack that is not yet scaffolded.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

fail=0
run() {
  echo "==> $*"
  if ! "$@"; then
    echo "FAILED: $*" >&2
    fail=1
  fi
}

if [[ -f Cargo.toml ]]; then
  echo "== Rust core =="
  run cargo fmt --all --check
  run cargo clippy --all-targets -- -D warnings
  run cargo test --all-targets
else
  echo "== Rust core: no Cargo.toml, skipping =="
fi

if ls frontend/*.nimble >/dev/null 2>&1; then
  echo "== Nim/Tatui TUI =="
  if command -v nph >/dev/null 2>&1; then
    run nph --check frontend
  else
    echo "nph not installed, skipping format check"
  fi
  ( cd frontend && run nimble test )
else
  echo "== Nim/Tatui TUI: no frontend/*.nimble, skipping =="
fi

if [[ "$fail" -ne 0 ]]; then
  echo "Quality gate FAILED." >&2
  exit 1
fi
echo "Quality gate OK."
