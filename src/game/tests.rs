use super::board::{Board, Pos, COLS, ROWS};
use super::render::{Bitmap, Framebuffer};
use super::*;

fn game() -> Game {
    Game::new(Sprites::blank())
}

fn at(g: &Game, col: i32, row: i32) -> Tile {
    g.board[Pos::new(col, row)]
}

fn set(g: &mut Game, col: i32, row: i32, t: Tile) {
    g.board.set(Pos::new(col, row), t);
}

fn count(g: &Game, t: Tile) -> usize {
    Board::positions().filter(|&p| g.board[p] == t).count()
}

fn go_to_level(g: &mut Game, n: i32) {
    while g.modal().is_some() {
        g.close_modal(None);
    }
    g.menu_level();
    g.close_modal(Some(&n.to_string()));
    assert_eq!(g.level, n);
}

/// Move the man to an empty spot, clearing his old one.
fn move_man_to(g: &mut Game, col: i32, row: i32) {
    g.board.set(g.man, Tile::Empty);
    g.place_man(col, row);
}

// ----- gameplay is unchanged ----------------------------------------------

fn fnv(h: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *h ^= u64::from(b);
        *h = h.wrapping_mul(0x100_0000_01b3);
    }
}

/// Plays every level for 3,000 random moves and timer ticks (dying a couple
/// of hundred times along the way) and fingerprints the board, the screen
/// and the generator.  The expected values were recorded from the original
/// statement-by-statement port, so any change to the game's behaviour or
/// drawing shows up here.
#[test]
fn gameplay_matches_recorded_fingerprints() {
    const EXPECTED: [u64; 9] = [
        0x79dbcbc400e3207e,
        0x0e85fcedfc2c7e61,
        0x52de15f27b471a28,
        0x28a8ee5bd127c59c,
        0xfa639a5061c67223,
        0x487fe5eb7e889764,
        0x54a798fe689cd57a,
        0xc9d435c5398cf0e4,
        0xdf222ebf475164e2,
    ];
    let mut g = Game::new(crate::assets::sprites());
    for (lvl, expected) in (1..).zip(EXPECTED) {
        go_to_level(&mut g, lvl);
        let mut r = TpRandom::new(42 + lvl as u32);
        for _ in 0..3000 {
            if g.modal().is_some() {
                g.close_modal(None);
            }
            if r.random(4) == 0 {
                g.timer();
            } else {
                let k = r.random(5) as usize;
                let shift = r.random(4) == 0;
                if k > 0 {
                    g.key_down(Dir::ALL[k - 1], shift);
                }
            }
        }
        let mut h = 0xcbf2_9ce4_8422_2325;
        for p in Board::positions() {
            fnv(&mut h, format!("{:?}", g.board[p]).as_bytes());
        }
        for px in g.pixels().chunks_exact(4) {
            fnv(&mut h, &px[..3]);
        }
        fnv(&mut h, &g.level.to_le_bytes());
        fnv(&mut h, &g.diamonds.to_le_bytes());
        fnv(&mut h, &g.rng.seed.to_le_bytes());
        assert_eq!(h, expected, "level {lvl}");
    }
}

// ----- levels ---------------------------------------------------------------

#[test]
fn levels_are_deterministic() {
    let mut a = game();
    let mut b = game();
    for lvl in 1..=9 {
        a.init_level(lvl);
        b.init_level(lvl);
        assert_eq!(a.board, b.board, "level {lvl}");
    }
}

#[test]
fn level1_layout() {
    let g = game();
    assert_eq!(at(&g, 2, 2), Tile::Man1);
    assert_eq!(g.exit, Pos::new(31, 19));
    assert_eq!(g.diamonds_needed, 67);
    // Walls at columns 8, 15 and 23 with an earth gap at rows 9..=12.
    assert_eq!(at(&g, 8, 5), Tile::Wall);
    assert_eq!(at(&g, 8, 10), Tile::Earth);
    assert!(count(&g, Tile::Diamond) > 67);
    for p in Board::positions() {
        if p.col == 1 || p.col == COLS || p.row == 1 || p.row == ROWS {
            assert_eq!(g.board[p], Tile::Bounds, "{p:?}");
        }
    }
}

#[test]
fn every_level_has_a_man_and_its_creatures() {
    let mut g = game();
    let creatures = [0, 3, 8, 14, 4, 0, 30, 50, 12];
    for (lvl, n) in (1..).zip(creatures) {
        g.init_level(lvl);
        assert_eq!(g.board[g.man], Tile::Man1, "level {lvl}");
        assert_eq!(g.creature_count, n, "level {lvl}");
        // Some levels clear space over their creatures after placing them;
        // they reappear when they first move, as in the original.
        g.move_creatures();
        let on_board = Board::positions()
            .filter(|&p| matches!(g.board[p], Tile::Life(_)))
            .count();
        assert_eq!(on_board, n, "level {lvl}");
    }
}

#[test]
fn out_of_range_level_gets_level_1() {
    let mut g = game();
    let level1 = g.board.clone();
    g.init_level(0);
    assert_eq!(g.board, level1);
    g.init_level(12);
    assert_eq!(g.board, level1);
}

// ----- falling ----------------------------------------------------------------

#[test]
fn boulder_falls_and_kills() {
    let mut g = game();
    for p in Board::positions().filter(|p| (2..ROWS).contains(&p.row)) {
        g.board.set(p, Tile::Earth);
    }
    g.place_man(2, 6);
    set(&mut g, 2, 3, Tile::Boulder);
    set(&mut g, 2, 4, Tile::Empty);
    set(&mut g, 2, 5, Tile::Empty);
    g.timer(); // starts falling into the gap
    assert_eq!(at(&g, 2, 4), Tile::FnBoulder);
    assert_eq!(g.modal(), None);
    g.timer(); // directly above the man
    assert_eq!(g.modal(), Some(Modal::Death));
    assert!(!g.alive);
    assert_eq!(at(&g, 2, 6), Tile::FnBoulder, "the boulder takes his place");
    g.close_modal(None);
    assert!(g.alive);
    assert_eq!(g.level, 1);
    assert_eq!(at(&g, 2, 2), Tile::Man1, "level restarted");
}

#[test]
fn falling_object_lands_on_earth() {
    let mut g = game();
    set(&mut g, 5, 5, Tile::FnDiamond);
    set(&mut g, 5, 6, Tile::Earth);
    g.drop_all();
    assert_eq!(at(&g, 5, 5), Tile::FDiamond);
}

#[test]
fn rolls_both_ways() {
    // A boulder on a wall with space on both sides should sometimes roll
    // left and sometimes right.
    let (mut left, mut right) = (0, 0);
    let mut g = game();
    for seed in 0..200 {
        for p in
            Board::positions().filter(|p| (2..COLS).contains(&p.col) && (2..ROWS).contains(&p.row))
        {
            g.board.set(p, Tile::Empty);
        }
        set(&mut g, 10, 19, Tile::Wall);
        set(&mut g, 10, 18, Tile::Boulder);
        g.rng.seed = seed * 7919;
        g.drop_all();
        // (It may fall further within the same sweep.)
        if (17..=19).any(|r| at(&g, 9, r) == Tile::FnBoulder) {
            left += 1;
        }
        if (17..=19).any(|r| at(&g, 11, r) == Tile::FnBoulder) {
            right += 1;
        }
    }
    assert!(left > 20 && right > 20, "left {left} right {right}");
}

#[test]
fn tile_states_round_trip() {
    for t in [Tile::Boulder, Tile::FBoulder, Tile::Diamond, Tile::FDiamond] {
        assert!(t.is_resting());
        assert!(t.falling().is_falling());
        assert!(t.falling().landed().is_resting());
    }
    assert_eq!(Tile::Earth.falling(), Tile::Earth);
    assert_eq!(Tile::Outside.sprite(), None);
    assert_eq!(Tile::Life(9).sprite(), Some(Tile::SPRITES - 1));
}

// ----- the man ----------------------------------------------------------------

#[test]
fn eat_diamond_scores() {
    let mut g = game();
    let Pos { col, row } = g.man;
    set(&mut g, col + 1, row, Tile::Diamond);
    g.key_down(Dir::Right, false);
    assert_eq!(g.diamonds, 1);
    assert_eq!(g.man, Pos::new(col + 1, row));
    assert!(at(&g, col + 1, row).is_man());
    assert_eq!(at(&g, col, row), Tile::Empty);
}

#[test]
fn shift_digs_without_moving() {
    let mut g = game();
    let Pos { col, row } = g.man;
    set(&mut g, col + 1, row, Tile::Earth);
    g.key_down(Dir::Right, true);
    assert_eq!(g.man, Pos::new(col, row));
    assert_eq!(at(&g, col + 1, row), Tile::Empty);
}

#[test]
fn push_boulder() {
    let mut g = game();
    let Pos { col, row } = g.man;
    set(&mut g, col + 1, row, Tile::Boulder);
    set(&mut g, col + 2, row, Tile::Empty);
    g.key_down(Dir::Right, false);
    assert_eq!(g.man.col, col + 1);
    assert_eq!(at(&g, col + 2, row), Tile::FBoulder);
}

#[test]
fn shift_pushes_without_moving() {
    let mut g = game();
    let Pos { col, row } = g.man;
    set(&mut g, col + 1, row, Tile::Boulder);
    set(&mut g, col + 2, row, Tile::Empty);
    g.key_down(Dir::Right, true);
    assert_eq!(g.man, Pos::new(col, row));
    assert!(at(&g, col, row).is_man());
    assert_eq!(at(&g, col + 1, row), Tile::Empty);
    assert_eq!(at(&g, col + 2, row), Tile::FBoulder);
}

#[test]
fn cannot_push_up_or_into_things() {
    let mut g = game();
    move_man_to(&mut g, 5, 5);
    set(&mut g, 5, 4, Tile::Boulder);
    set(&mut g, 5, 3, Tile::Empty);
    g.key_down(Dir::Up, false);
    assert_eq!(g.man, Pos::new(5, 5));
    set(&mut g, 6, 5, Tile::Boulder);
    set(&mut g, 7, 5, Tile::Wall);
    g.key_down(Dir::Right, false);
    assert_eq!(g.man, Pos::new(5, 5));
}

#[test]
fn reaching_exit_advances() {
    let mut g = game();
    g.diamonds = g.diamonds_needed - 1;
    g.display_score(1);
    assert!(g.exit_open);
    assert_eq!(at(&g, 31, 19), Tile::Door);
    move_man_to(&mut g, 30, 19);
    g.key_down(Dir::Right, false);
    assert_eq!(g.level, 2);
    assert_eq!(g.creature_count, 3);
}

#[test]
fn finishing_level_9_wraps_to_level_1() {
    let mut g = game();
    go_to_level(&mut g, 9);
    g.diamonds = g.diamonds_needed - 1;
    g.display_score(1);
    move_man_to(&mut g, 30, 10);
    g.key_down(Dir::Right, false);
    assert_eq!(g.modal(), Some(Modal::AllDone));
    g.timer(); // paused
    assert_eq!(g.modal(), Some(Modal::AllDone));
    g.close_modal(None);
    assert_eq!(g.level, 1);
    assert_eq!(at(&g, 2, 2), Tile::Man1);
}

// ----- creatures ----------------------------------------------------------------

#[test]
fn creature_kills_man() {
    let mut g = game();
    go_to_level(&mut g, 2);
    let c = g.creatures[0].pos;
    // Box the creature in except for the man next to it.
    for d in Dir::ALL {
        g.board.set(c + d.delta(), Tile::Earth);
    }
    move_man_to(&mut g, c.col, c.row - 1);
    for _ in 0..4 {
        g.timer();
        if g.modal().is_some() {
            break;
        }
    }
    assert_eq!(g.modal(), Some(Modal::Death));
}

#[test]
fn crushed_creature_becomes_diamonds() {
    let mut g = game();
    go_to_level(&mut g, 2);
    let c = g.creatures[0].pos;
    g.board.set(c.up(), Tile::FnBoulder);
    g.move_creatures();
    assert!(g.creatures[0].dead);
    for dc in -1..=1 {
        for dr in -1..=1 {
            assert_eq!(g.board[c + (dc, dr)], Tile::FDiamond);
        }
    }
}

#[test]
fn trapped_creature_gives_up() {
    let mut g = game();
    go_to_level(&mut g, 2);
    let c = g.creatures[0].pos;
    for d in Dir::ALL {
        g.board.set(c + d.delta(), Tile::Earth);
    }
    for _ in 0..29 {
        g.move_creatures();
    }
    assert!(!g.creatures[0].dead);
    g.move_creatures();
    assert!(g.creatures[0].dead);
    assert_eq!(g.board[c], Tile::FDiamond);
}

// ----- the morph ------------------------------------------------------------------

#[test]
fn morph_only_on_level_6() {
    let mut g = game();
    go_to_level(&mut g, 6);
    g.timer();
    assert!(matches!(at(&g, 15, 14), Tile::Morph(_)));
    go_to_level(&mut g, 7);
    for _ in 0..5 {
        g.timer();
    }
    assert!(!matches!(at(&g, 15, 14), Tile::Morph(_)));
    assert_eq!(at(&g, 0, 0), Tile::Outside);
}

#[test]
fn morph_turns_boulder_into_diamond() {
    let mut g = game();
    go_to_level(&mut g, 6);
    let m = Pos::new(15, 14);
    g.board.set(m.up(), Tile::FBoulder);
    g.board.set(m.down(), Tile::Empty);
    g.drop_all();
    assert_eq!(g.board[m.up()], Tile::Empty);
    assert!(g.board[m.down()].is_diamond());
}

// ----- dialogs ----------------------------------------------------------------------

#[test]
fn level_dialog() {
    let mut g = game();
    g.menu_level();
    assert_eq!(g.modal(), Some(Modal::LevelInput));
    g.close_modal(Some("4"));
    assert_eq!(g.level(), 4);
    assert_eq!(g.creature_count, 14);
    // Cancel and bad input change nothing.
    g.key_down(Dir::Right, false);
    assert_eq!(g.man.col, 3);
    g.menu_level();
    g.close_modal(None);
    assert_eq!(g.modal(), None);
    assert_eq!(g.level(), 4);
    assert_eq!(g.man.col, 3, "level was not restarted");
    for bad in ["", "0", ":", "10", "x"] {
        g.menu_level();
        g.close_modal(Some(bad));
        assert_eq!(g.modal(), None);
        assert_eq!(g.level(), 4, "input {bad:?}");
    }
    g.menu_level();
    g.close_modal(Some(" 9 "));
    assert_eq!(g.level(), 9);
}

#[test]
fn paused_under_modal() {
    let mut g = game();
    g.menu_about();
    let before = (g.rng.seed, g.board.clone());
    g.timer();
    g.key_down(Dir::Right, false);
    assert_eq!((g.rng.seed, g.board.clone()), before);
    assert_eq!(g.modal(), Some(Modal::About));
    g.menu_level(); // only one box at a time
    assert_eq!(g.modal(), Some(Modal::About));
}

#[test]
fn soak_all_levels() {
    // Run every level for a while with random input; must not panic.
    let mut g = game();
    let mut r = TpRandom::new(1234);
    for lvl in 1..=9 {
        go_to_level(&mut g, lvl);
        for _ in 0..2000 {
            if g.modal().is_some() {
                g.close_modal(None);
            }
            if r.random(3) == 0 {
                g.timer();
            } else {
                let d = Dir::ALL[r.random(4) as usize];
                g.key_down(d, r.random(5) == 0);
            }
        }
    }
}

// ----- drawing ------------------------------------------------------------------------

#[test]
fn redraws_only_when_something_changes() {
    let mut g = game();
    assert!(g.take_redraw());
    assert!(!g.take_redraw());
    g.timer(); // the exit flashes every tick
    assert!(g.take_redraw());
}

#[test]
fn blit_clips_at_the_edges() {
    let mut fb = Framebuffer::new();
    let red = Bitmap::solid(CELL, CELL, [255, 0, 0]);
    fb.blit(-10, -10, &red);
    fb.blit(FB_W as i32 - 5, FB_H as i32 - 5, &red);
    fb.blit(-100, 0, &red); // entirely off screen
    let px = |x: usize, y: usize| &fb.rgba()[(y * FB_W + x) * 4..][..4];
    assert_eq!(px(0, 0), [255, 0, 0, 255]);
    assert_eq!(px(9, 9), [255, 0, 0, 255]);
    assert_eq!(px(10, 10), [255, 255, 255, 255]);
    assert_eq!(px(FB_W - 1, FB_H - 1), [255, 0, 0, 255]);
}

#[test]
fn invert_twice_restores() {
    let mut g = game();
    g.paint();
    let before = g.pixels().to_vec();
    g.flash_exit();
    assert_ne!(g.pixels(), &before[..]);
    g.flash_exit();
    assert_eq!(g.pixels(), &before[..]);
}

#[test]
fn score_bar_fills() {
    let mut g = game();
    let filled = |g: &Game| {
        let row = &g.pixels()[..FB_W * 4];
        (0..FB_W).filter(|x| row[x * 4..][..3] == [0, 1, 0]).count()
    };
    g.display_score(0);
    assert_eq!(filled(&g), 0);
    g.diamonds = g.diamonds_needed / 2;
    g.display_score(0);
    assert!((95..=100).contains(&filled(&g)));
    g.display_score(g.diamonds_needed);
    assert_eq!(filled(&g), 196);
}

/// `ROCK_LEVEL_PNGS=dir cargo test level_pngs -- --ignored` writes the
/// opening screen of every level, drawn with the real bitmaps.
#[test]
#[ignore]
fn level_pngs() {
    let dir = std::env::var("ROCK_LEVEL_PNGS").unwrap_or_else(|_| ".".into());
    let mut g = Game::new(crate::assets::sprites());
    for lvl in 1..=9 {
        go_to_level(&mut g, lvl);
        // export_png expects bottom-up rows (it is meant for GL screen
        // grabs), so hand it the picture upside down.
        let bytes = g
            .pixels()
            .chunks(FB_W * 4)
            .rev()
            .flatten()
            .copied()
            .collect();
        let img = macroquad::prelude::Image {
            bytes,
            width: FB_W as u16,
            height: FB_H as u16,
        };
        img.export_png(&format!("{dir}/level{lvl}.png"));
    }
}

/// Rough timing: `cargo test --release bench -- --ignored --nocapture`.
#[test]
#[ignore]
fn bench() {
    let mut g = Game::new(crate::assets::sprites());
    let t = std::time::Instant::now();
    for rep in 0..20 {
        for lvl in 1..=9 {
            go_to_level(&mut g, lvl);
            let mut r = TpRandom::new(rep + lvl as u32);
            for _ in 0..3000 {
                if g.modal().is_some() {
                    g.close_modal(None);
                }
                if r.random(4) == 0 {
                    g.timer();
                } else {
                    g.key_down(Dir::ALL[r.random(4) as usize], false);
                }
            }
        }
    }
    println!("BENCH {:?}", t.elapsed());
}
