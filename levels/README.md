# Level files

Each `levelN.txt` describes one of the nine levels. They are compiled into the
game (`src/game/levels.rs`), so after editing one, rebuild with `cargo run`.
A mistake in a file stops the game with the file name and line number, and
`cargo test` checks that every file reads correctly.

A file is a few settings followed by a picture of the board:

    // Level 2: ...

    diamonds 35
    random-draws 160

    creature 10,9 hidden
    creature 14,9 hidden
    creature 20,9 hidden

    map
    ###############################
    #M.....=.*....=.......=.......#
    ...

Anything after `//` on a settings line is a comment. Blank lines are ignored.

## Settings

| Line | Meaning |
|---|---|
| `diamonds N` | How many diamonds to collect before the door opens. Required. |
| `random-draws N` | How many numbers the original drew from Turbo Pascal's `Random` while scattering this level's diamonds and boulders (two per item). Every level restarts the generator from `$FFFF`, and the game's later random numbers (which way things roll, the morph) carry on from this point. Leave it out for 0. |
| `creature COL,ROW` | A creature, on an `@` in the map. Creatures are numbered in the order they are listed, which is the order they move in. Odd and even numbers follow the walls in opposite directions, and the number also picks the starting animation frame. |
| `creature COL,ROW hidden` | A creature that isn't drawn until it first moves. The original built some levels on top of their creatures, so they start under whatever the map shows at that cell (earth on level 3, empty space on levels 2 and 9). |

Columns run 1 to 31 from the left and rows 1 to 20 from the top, the same
numbering as the original Pascal.

## The map

The `map` line is followed by exactly 20 rows of exactly 31 characters, one per
cell. The outer ring must be `#` or `E`.

| Character | Cell |
|---|---|
| `#` | The steel border (`Bounds`) |
| `E` | The exit: a border cell where the door appears once you have enough diamonds. Exactly one. |
| `=` | Brick wall |
| `.` | Earth |
| (space) | Empty |
| `O` | Boulder |
| `*` | Diamond |
| `M` | The man. Exactly one. |
| `@` | A creature (needs a `creature` line) |
| `%` | The morph. At most one. |

The nine files reproduce the original's levels exactly, including the
scattered diamonds and boulders on levels 1 and 2. The gameplay fingerprint
test in `src/game/tests.rs` fails if any of that changes.
