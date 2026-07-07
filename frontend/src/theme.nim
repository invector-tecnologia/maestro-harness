## Accessibility theming — an ASCII-only fallback for terminals that cannot render
## Unicode box-drawing. Enabled with the `MAESTRO_ASCII_ONLY` environment variable.
##
## Tatui only ships Unicode border glyphs, so in ASCII mode panels are drawn
## borderless with a plain `[Title]` header line instead of a boxed title.

import std/os
import tatui

proc asciiOnly*(): bool =
  ## Whether the ASCII-only accessibility mode is active.
  getEnv("MAESTRO_ASCII_ONLY").len > 0

proc panelBorders*(): Borders =
  ## The border set for a panel: none in ASCII mode, full box otherwise.
  if asciiOnly(): {} else: AllBorders

proc panelTitle*(title: string): string =
  ## The block title — empty in ASCII mode (rendered as a text header instead).
  if asciiOnly(): "" else: " " & title & " "

proc asciiHeader*(title: string): string =
  ## A plain-text header line used in ASCII mode; empty in normal mode.
  if asciiOnly(): "[" & title & "]\n" else: ""
