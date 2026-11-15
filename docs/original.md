# The original source

[Back to the README](../README.md)

The port is based entirely on the original 1993 program, whose files are
kept unchanged in [`Org Pascal/`](../Org%20Pascal/) in this repository. The game rules were ported line by
line from the Pascal units, every image came out of the resource file, and the
random-number behaviour was checked against the compiled program.

| File | Date | What it is |
|---|---|---|
| `ROCK.PAS` | 25 Apr 1993 | The main program: the ObjectWindows application and window, the 200 ms timer, keyboard and mouse handlers, painting, and the About and Level menu commands. Its `{$D}` line reads "Dale O'Brien, Beta Verison of Rock, 1993, Jan, Coded using Borland TPW 1.5". |
| `ROCK_FUC.PAS` | 24 Apr 1993 | The `Rock_Fuc` unit: the game logic (level setup, falling and rolling, the creatures, the morph, scoring, drawing cells). Ported to `src/game/`. |
| `ROCK_VAR.PAS` | 24 Apr 1993 | The `Rock_Var` unit: grid size, constants, types and the global game state. Ported to `src/game/`. |
| `ROCK20.RES` | 25 Apr 1993 | The Windows 3.x resource file linked in with `{$R rock20.res}`: 31 bitmaps, the icon and the menu. Extracted into `resources/`. |
| `ROCK.EXE` | 25 Apr 1993 | The compiled 16-bit Windows program. Used to confirm the resources and Turbo Pascal's `Random`. |
| `ROCK_FUC.TPU`, `ROCK_VAR.TPU` | 24 Apr 1993 | Compiled Turbo Pascal units for the two `.PAS` units. |
| `ROCK.RTF` | 15 Jan 1993 | The start of a WinHelp help file that was never finished or hooked up. |

To build the original you would need Borland Turbo Pascal for Windows 1.5
(its `WObjects`, `WinTypes`, `WinProcs`, `Strings`, `StdDlgs` and `WinCrt`
units) and Windows 3.x to run it.

## What was in ROCK20.RES

All of it is also embedded byte-for-byte in `ROCK.EXE`, so the `.RES` file matches the
shipped program.

* **31 bitmaps**, 20 x 20 except the 2 x 20 score-bar strips, in 16- and
  256-colour DIB format → `resources/bitmaps/*.png`. Overview:
  `resources/contact_sheet.png`. `CHERRY` and `DIA256` are in the file but the
  program never loads them, and `MORPH4` is loaded but never shown.
* **1 icon** (32 x 32, 16 colours) → `resources/icons/icon.ico` and `.png`.
  The port uses it as its window icon.
* **1 menu** `MENU_1`: `&About` (100), `&Level` (200) → `resources/menus/menu_1.json`
* There are no dialog, string table or cursor resources. The Level dialog
  came from Borland's `StdDlgs` unit, not from this file.
* `resources/manifest.json` lists every resource in the file.

## Re-extracting the resources

Point the extractor at `ROCK20.RES` in the original source folder. On Dale's
Mac:

    python3 tools/extract_res.py "Org Pascal/ROCK20.RES" resources
