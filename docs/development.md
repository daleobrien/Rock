# Building and the code

[Back to the README](../README.md)

## Building

You need a Rust toolchain (https://rustup.rs).

    make run        # or cargo run --release
    make test
    make           # the rest

It uses [macroquad](https://macroquad.rs), so it builds as-is on Windows,
macOS and Linux (on Linux it needs the usual X11/GL development libraries).
All the bitmaps are compiled into the binary, so `target/release/rock` runs on
its own.

`cargo test` runs the tests. One of them plays every level for 3,000 random
moves and checks the result against a fingerprint recorded from the first,
line-by-line version of the port, so any accidental change to how the game
plays or looks is caught.

## In a web browser

`make web` builds the same program for WebAssembly into `web/dist`:
`rock.wasm`, the page (`web/index.html`) and macroquad's JavaScript loader
(`mq_js_bundle.js`, fetched into `web/` the first time). It adds the
`wasm32-unknown-unknown` target the first time too. `make serve` builds it
and serves it at http://localhost:8000 (the page won't load from
`file://`), and `make deploy` builds it and deploys it to Cloudflare, where
rock.do.id.au is a Worker serving `web/dist` (`wrangler.jsonc`). Pushes to
`main` deploy it through `.github/workflows/web.yml`. In the
browser the level isn't remembered between visits: it starts at level 1.

## Layout

| Path | What it is |
|---|---|
| `Org Pascal/` | The original 1993 Turbo Pascal source, resources and compiled program |
| `src/game/mod.rs` | The `Game` type the window drives: timer, keys, menu commands and message boxes |
| `src/game/tile.rs` | What a cell can hold (boulder, diamond, creature frame...) and what each kind does |
| `src/game/board.rs` | The 31 x 20 board, positions and directions |
| `src/game/physics.rs` | Falling and rolling, and the morph |
| `src/game/creatures.rs` | The wall-following creatures |
| `src/game/player.rs` | Moving, digging, pushing and dying |
| `levels/` | The nine levels, one text file each: settings plus a picture of the board (format in [`levels/README.md`](../levels/README.md)) |
| `src/game/levels.rs` | Reads the level files, which are compiled into the binary |
| `src/game/render.rs` | The framebuffer and bitmaps it draws into |
| `src/game/tests.rs` | Tests |
| `src/rng.rs` | Turbo Pascal's `Random`, bit for bit |
| `src/main.rs` | Window, 200 ms timer, keyboard, scaling |
| `src/ui.rs` | Windows 3.1-style menu bar, message boxes and the Level input dialog |
| `src/assets.rs` | Embeds the PNGs from `resources/` into the binary |
| `src/save.rs` | Saves the current level between runs |
| `src/script.rs` | Scripted input for automated screenshots (only used if `ROCK_SCRIPT` is set; scripted runs start at level 1 and don't save) |
| `resources/` | Everything extracted from `ROCK20.RES` (see [What was in ROCK20.RES](original.md#what-was-in-rock20res)); the game builds its images in from here |
| `tools/extract_res.py` | The extractor for 16-bit `.RES` files (needs Pillow) |
| `screenshots/` | The port running |
