#!/usr/bin/env bash
# Install the exact Tatui commit behind the v0.1.2 tag.
#
# Tatui is not published to the nimble registry yet, so the Nim frontend depends
# on it by bare name (`requires "tatui"`) and this script provides the exact,
# immutable source. Pinning the commit (not the movable tag) makes builds reproducible.
set -euo pipefail

# Commit behind tatui tag v0.1.2.
TATUI_REV="${TATUI_REV:-493d9fc0b32ad3505c927bbe288b65a5e4a5f704}"

echo "==> Installing tatui @ ${TATUI_REV}"
nimble install -y "https://github.com/invector-tecnologia/tatui@#${TATUI_REV}"
