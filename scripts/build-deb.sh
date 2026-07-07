#!/usr/bin/env bash
# Build a Debian .deb for the Maestro core binary (TASK 015).
# Usage: scripts/build-deb.sh [version]
set -euo pipefail

version="${1:-$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')}"
arch="$(dpkg --print-architecture 2>/dev/null || echo amd64)"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

pkg="maestro-ai_${version}_${arch}"
stage="target/deb/${pkg}"

echo "==> Building release binary"
cargo build --release --locked

echo "==> Staging package tree at ${stage}"
rm -rf "$stage"
mkdir -p "$stage/DEBIAN" "$stage/usr/bin"
install -m 0755 target/release/maestro "$stage/usr/bin/maestro"

cat > "$stage/DEBIAN/control" <<EOF
Package: maestro-ai
Version: ${version}
Section: utils
Priority: optional
Architecture: ${arch}
Maintainer: Invector Tecnologia <dev@invector.tec>
Description: Maestro — local-first tactical agentic workflow orchestrator.
 Headless Rust core for planning and executing disposable micro-projects.
EOF

# Lifecycle hooks: purge removes any user-local state.
cat > "$stage/DEBIAN/postrm" <<'EOF'
#!/bin/sh
set -e
if [ "$1" = "purge" ]; then
  rm -rf "${HOME:-/root}/.config/maestro" 2>/dev/null || true
fi
exit 0
EOF
chmod 0755 "$stage/DEBIAN/postrm"

echo "==> Building .deb"
dpkg-deb --build --root-owner-group "$stage" "target/deb/${pkg}.deb"
echo "==> Wrote target/deb/${pkg}.deb"
