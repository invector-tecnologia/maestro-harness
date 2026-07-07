# Package
version = "0.1.0"
author = "Invector Tecnologia"
description =
  "Maestro Nim/Tatui TUI frontend — renders core state over a stdio JSON protocol."
license = "GPL-3.0-only"
srcDir = "src"
bin = @["maestro_tui"]

# Dependencies
requires "nim >= 2.0.0"
# Tatui is not on the nimble registry yet. Resolve it by bare name; install the
# exact immutable commit behind the v0.1.2 tag before building (see scripts/install-tatui.sh):
#   nimble install -y "https://github.com/invector-tecnologia/tatui@#493d9fc0b32ad3505c927bbe288b65a5e4a5f704"
requires "tatui"
