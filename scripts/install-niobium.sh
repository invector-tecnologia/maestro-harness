#!/usr/bin/env bash
# Install the exact Niobium commit behind the v0.1.0 tag.
#
# Niobium is not published to the nimble registry yet, so the Nim frontend depends
# on it by bare name (`requires "niobium"`) and this script provides the exact,
# immutable source. Pinning the commit (not the movable tag) makes builds reproducible.
set -euo pipefail

# Commit behind niobium tag v0.1.0.
NIOBIUM_REV="${NIOBIUM_REV:-0051e11235280f4a235e573141ca2f40310d2e60}"

echo "==> Installing niobium @ ${NIOBIUM_REV}"
nimble install -y "https://github.com/invector-tecnologia/niobium@#${NIOBIUM_REV}"
