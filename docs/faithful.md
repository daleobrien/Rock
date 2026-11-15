# How faithful is it

[Back to the README](../README.md)

The game logic was first ported statement by statement and then reorganised
into idiomatic Rust, with a test checking that it still plays exactly the same.
The screen is drawn the same way as the original. The original never kept a picture of the screen: it `BitBlt`ed each
cell straight to the window as it changed and flashed the exit with
`InvertRect`. The port keeps a 620 x 400 framebuffer and does the same blits
into it, so things like the flashing exit and the stretched score bar look the
same.

## Bugs in the original that the port fixes

* **Dialogs pause the game.** In the original, the 200 ms timer kept running
  behind message boxes and the Level dialog, so creatures kept moving and you
  could even die while reading the About box.
* **The Level dialog does what it says.** It opens showing the current level
  and only accepts a single digit from 1 to 9. **Cancel** leaves the game
  alone. The original remembered the last thing typed, because it wrote into
  the string constant `'1'`, and it changed level even on Cancel. It also took
  any character, so `0` or an empty box loaded the level 1 layout under the
  wrong level number, and `:` jumped straight to "All done".
* **Falling things roll either way.** The original's `Random(1) < 0.5` is
  always true, so boulders and diamonds always tried to roll right first.
  It is now a real 50/50.
* **The morph stays on level 6.** The original never cleared `Morphic[0]`,
  so once you had played level 6, a morph kept appearing at column 15, row 14
  on every later level. On earlier levels it wrote to `Map[0,0]`, outside the
  map, every tick.
* **Only the arrow keys do anything.** In the original any key made the man
  blink, and pressing Shift while holding an arrow stopped the arrow
  repeating.

## Kept as they were

* There is no sound. The original has `MessageBeep` calls, but its `Sound`
  flag is always False.
* Mouse clicks on the board do nothing. The original works out the cell and
  then ignores it.
* The level layouts, including the scattered diamonds and boulders, are the
  same as the original's.

## What couldn't be reproduced exactly

* **Window size.** The original asked for a 620 x 440 window *including* the
  caption, menu and dialog frame, so its client area was a little smaller
  than the 620 x 400 board. The right-hand column and bottom row were partly
  cut off. The port shows the whole board, and the window can be resized, with
  whole-number pixel scaling where it fits (it opens at 2x).
* **Windows 3.1 look.** The menu bar, message boxes and input dialog are
  redrawn to look like Windows 3.1, but they use macroquad's built-in font
  instead of the System font, and the sizes are close, not exact. There is
  no keyboard menu navigation (F10 / Alt alone); Alt+A and Alt+L work.
* **Colours.** The bitmaps are shown in full colour. On a typical 1993
  16-colour VGA driver, the 256-colour bitmaps (man, creatures, door, falling
  boulder) would have been mapped to the nearest of the 16 system colours, so
  the original probably looked harsher. The exit flash inverts RGB values,
  which matches what the 16 standard colours did.
* **Keyboard repeat.** The original moved at whatever repeat rate Windows was
  set to. The port copies the Windows defaults (about 500 ms delay, then 30
  repeats a second).
* **Out-of-range memory access.** Besides the morph write above, the
  original paint routine read row and column 0, outside the map. That read
  never changed anything visible, and the port doesn't do it. The map array
  also has a border of padding cells, so any stray access stays harmless.
* **Repainting after a dialog.** Windows repainted only the area a closed
  dialog had covered. The port repaints the whole board, which can change
  which phase the exit flash is in.
* `ROCK.RTF` is the start of a WinHelp help file that was never hooked up, so
  there is nothing to port. The Mac files in the `ROCK` folder
  (`ROCK.xcodeproj`, `Start.pas`, `MainUnit.pas`) are a separate Free
  Pascal/Carbon project and were left alone.
