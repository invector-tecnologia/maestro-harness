#!/usr/bin/env bash
# Maestro Harness — frictionless installer.
#
# One command to go from a fresh checkout to an installed `maestro` binary.
# It detects (and optionally installs) the toolchain, builds the Rust core,
# optionally builds the Nim/Niobium TUI, and copies the binary onto your PATH.
#
# No project knowledge required. Just run:
#
#   ./scripts/install.sh
#
# Common options:
#   --no-tui           Build only the headless Rust core (skip the Nim TUI).
#   --prefix DIR       Install the binary into DIR (default: ~/.local/bin).
#   --auto-deps        Install missing toolchains (Rust/Nim) without prompting.
#   --no-auto-deps     Never install toolchains; fail if something is missing.
#   -y, --yes          Assume "yes" for every prompt (non-interactive).
#   -h, --help         Show this help.
#
# Environment overrides:
#   PREFIX             Same as --prefix.
#   MAESTRO_NO_TUI=1   Same as --no-tui.
set -euo pipefail

# ----------------------------------------------------------------------------
# Presentation helpers
# ----------------------------------------------------------------------------
if [[ -t 1 ]]; then
  BOLD="$(printf '\033[1m')"; DIM="$(printf '\033[2m')"; RED="$(printf '\033[31m')"
  GREEN="$(printf '\033[32m')"; YELLOW="$(printf '\033[33m')"; BLUE="$(printf '\033[34m')"
  RESET="$(printf '\033[0m')"
else
  BOLD=""; DIM=""; RED=""; GREEN=""; YELLOW=""; BLUE=""; RESET=""
fi

step() { printf '%s==>%s %s\n' "$BLUE$BOLD" "$RESET" "$*"; }
ok()   { printf '%s  ✓%s %s\n' "$GREEN" "$RESET" "$*"; }
warn() { printf '%s  !%s %s\n' "$YELLOW" "$RESET" "$*" >&2; }
die()  { printf '%s  ✗ %s%s\n' "$RED$BOLD" "$*" "$RESET" >&2; exit 1; }

# ----------------------------------------------------------------------------
# Defaults & argument parsing
# ----------------------------------------------------------------------------
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PREFIX="${PREFIX:-$HOME/.local/bin}"
BUILD_TUI=1
[[ "${MAESTRO_NO_TUI:-0}" == "1" ]] && BUILD_TUI=0
AUTO_DEPS="prompt"   # prompt | always | never
ASSUME_YES=0

usage() { sed -n '2,22p' "$0" | sed 's/^#\{1,\} \{0,1\}//'; }

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-tui) BUILD_TUI=0 ;;
    --tui) BUILD_TUI=1 ;;
    --prefix) shift; PREFIX="${1:?--prefix needs a directory}" ;;
    --prefix=*) PREFIX="${1#*=}" ;;
    --auto-deps) AUTO_DEPS="always" ;;
    --no-auto-deps) AUTO_DEPS="never" ;;
    -y|--yes) ASSUME_YES=1 ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown option: $1 (try --help)" ;;
  esac
  shift
done

# ----------------------------------------------------------------------------
# Small utilities
# ----------------------------------------------------------------------------
have() { command -v "$1" >/dev/null 2>&1; }

confirm() {
  # confirm "question" -> returns 0 for yes
  [[ "$ASSUME_YES" == "1" ]] && return 0
  [[ ! -t 0 ]] && return 1   # non-interactive without --yes: default no
  local reply
  printf '%s  ? %s [y/N] %s' "$YELLOW" "$*" "$RESET"
  read -r reply || true
  [[ "$reply" =~ ^[Yy] ]]
}

OS="$(uname -s)"
case "$OS" in
  Linux) PLATFORM="linux" ;;
  Darwin) PLATFORM="macos" ;;
  *) die "unsupported OS: $OS (Maestro targets Linux and macOS)" ;;
esac

# ----------------------------------------------------------------------------
# Toolchain: Rust
# ----------------------------------------------------------------------------
ensure_rust() {
  if have cargo; then
    ok "Rust toolchain: $(cargo --version)"
    return 0
  fi
  warn "Rust (cargo) not found — required to build the Maestro core."
  if [[ "$AUTO_DEPS" == "never" ]]; then
    die "install Rust from https://rustup.rs and re-run."
  fi
  if [[ "$AUTO_DEPS" == "always" ]] || confirm "Install Rust now via rustup?"; then
    have curl || die "curl is required to install rustup. Please install curl first."
    step "Installing Rust via rustup (non-interactive)"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
    have cargo || die "Rust install did not expose cargo on PATH."
    ok "Rust installed: $(cargo --version)"
  else
    die "Rust is required. Install it from https://rustup.rs and re-run."
  fi
}

# ----------------------------------------------------------------------------
# Toolchain: Nim (only for the TUI)
# ----------------------------------------------------------------------------
ensure_nim() {
  if have nimble && have nim; then
    ok "Nim toolchain: $(nim --version 2>/dev/null | head -1)"
    return 0
  fi
  warn "Nim (nim/nimble) not found — required only for the interactive TUI."
  if [[ "$AUTO_DEPS" == "never" ]]; then
    warn "Skipping TUI. Install Nim >= 2.0 from https://nim-lang.org to enable it."
    return 1
  fi
  if [[ "$AUTO_DEPS" == "always" ]] || confirm "Install Nim now via choosenim?"; then
    have curl || { warn "curl needed for choosenim; skipping TUI."; return 1; }
    step "Installing Nim via choosenim (non-interactive)"
    CHOOSENIM_NO_ANALYTICS=1 curl -sSf https://nim-lang.org/choosenim/init.sh | sh -s -- -y
    export PATH="$HOME/.nimble/bin:$PATH"
    have nimble || { warn "Nim install did not expose nimble on PATH; skipping TUI."; return 1; }
    ok "Nim installed: $(nim --version 2>/dev/null | head -1)"
  else
    warn "Skipping TUI build (Nim not installed)."
    return 1
  fi
  return 0
}

# ----------------------------------------------------------------------------
# Build steps
# ----------------------------------------------------------------------------
build_core() {
  step "Building the Maestro core (release)"
  cargo build --release --locked
  [[ -x target/release/maestro ]] || die "expected binary target/release/maestro was not produced."
  ok "Core binary built: target/release/maestro"
}

install_core() {
  step "Installing binary into ${PREFIX}"
  mkdir -p "$PREFIX"
  install -m 0755 target/release/maestro "$PREFIX/maestro"
  ok "Installed: ${PREFIX}/maestro"
}

build_tui() {
  step "Building the Nim/Niobium TUI"
  export PATH="$HOME/.nimble/bin:$PATH"
  "$ROOT/scripts/install-niobium.sh"
  ( cd frontend && nimble install -y --depsOnly && nimble build )
  if [[ -x frontend/maestro_tui ]]; then
    install -m 0755 frontend/maestro_tui "$PREFIX/maestro_tui"
    ok "Installed TUI: ${PREFIX}/maestro_tui"
  else
    warn "TUI build finished but no frontend/maestro_tui binary found; skipping install."
  fi
}

# ----------------------------------------------------------------------------
# PATH advisory
# ----------------------------------------------------------------------------
check_path() {
  case ":$PATH:" in
    *":$PREFIX:"*) ok "${PREFIX} is already on your PATH." ;;
    *)
      warn "${PREFIX} is not on your PATH."
      printf '%s    Add it with:%s\n' "$DIM" "$RESET"
      printf '      echo '\''export PATH="%s:$PATH"'\'' >> ~/.bashrc && source ~/.bashrc\n' "$PREFIX"
      ;;
  esac
}

# ----------------------------------------------------------------------------
# Main
# ----------------------------------------------------------------------------
printf '%s\n' "${BOLD}⚡ Maestro Harness installer${RESET} ${DIM}(${PLATFORM})${RESET}"
printf '%s\n' "${DIM}   prefix=${PREFIX}  tui=$([[ $BUILD_TUI == 1 ]] && echo yes || echo no)  auto-deps=${AUTO_DEPS}${RESET}"

ensure_rust
build_core
install_core

if [[ "$BUILD_TUI" == "1" ]]; then
  if ensure_nim; then
    build_tui
  else
    warn "Continuing with the headless core only. Re-run with Nim installed to add the TUI."
  fi
else
  step "Skipping TUI (--no-tui). Core runs headless."
fi

check_path

printf '\n%s✔ Maestro is ready.%s\n' "$GREEN$BOLD" "$RESET"
printf '  Try: %smaestro doctor%s   %smaestro list-agents%s\n' "$BOLD" "$RESET" "$BOLD" "$RESET"
[[ "$BUILD_TUI" == "1" ]] && printf '  Or launch the deck: %smaestro%s (headless: %smaestro --no-tui%s)\n' "$BOLD" "$RESET" "$BOLD" "$RESET"
