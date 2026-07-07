## Maestro TUI binary entry point.
##
## Thin wrapper that launches the application shell. Panels live under `panels/`
## (delivered by TASK 012) and consume only shipped Tatui widgets.

import ./app

when isMainModule:
  run()
