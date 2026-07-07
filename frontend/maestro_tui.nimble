# Package
version = "0.1.0"
author = "Invector Tecnologia"
description =
  "Maestro Nim/Niobium TUI frontend — renders core state over a stdio JSON protocol."
license = "GPL-3.0-only"
srcDir = "src"
bin = @["maestro_tui"]

# Dependencies
requires "nim >= 2.0.0"
# Niobium is not on the nimble registry yet. Resolve it by bare name; install the
# exact immutable commit behind the v0.1.0 tag before building (see scripts/install-niobium.sh):
#   nimble install -y "https://github.com/invector-tecnologia/niobium@#0051e11235280f4a235e573141ca2f40310d2e60"
requires "niobium"
