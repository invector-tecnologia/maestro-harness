#!/usr/bin/env bash
# Backward-compatible shim for workflows/scripts that still call the old name.
# The TUI dependency was migrated from Niobium to Tatui.
set -euo pipefail

exec "$(cd "$(dirname "$0")" && pwd)/install-tatui.sh" "$@"
