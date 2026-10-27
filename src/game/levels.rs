//! The nine levels (`Init_Screen`) and the tools for building them.

use super::board::{Board, Pos, COLS, ROWS};
use super::tile::Tile;
use super::Game;

/// Every level re-seeds Turbo Pascal's generator with this, so the
/// "random" diamonds and boulders always land in the same places.
const LEVEL_SEED: u32 = 0xFFFF;

impl Game {
    /// Set up level `n`.  Anything outside 1..=9 gets level 1's layout.
    pub(super) fn init_level(&mut self, n: i32) {
        self.creature_count = 0;
        self.morph = None;
        self.just_died = false;
        self.exit_open = false;
        self.alive = true;
        self.rng.seed = LEVEL_SEED;
        self.diamonds = 0;
        self.fill_all(Tile::Earth);

        match n {
            2 => self.level_2(),
            3 => self.level_3(),
            4 => self.level_4(),
            5 => self.level_5(),
            6 => self.level_6(),
            7 => self.level_7(),
            8 => self.level_8(),
            9 => self.level_9(),
            _ => self.level_1(),
        }

        // The Bounds wall around the edge.
        for p in Board::positions() {
            if p.col == 1 || p.col == COLS || p.row == 1 || p.row == ROWS {
                self.board.set(p, Tile::Bounds);
            }
        }
    }

    /// Diamonds and boulders among three walls; no creatures.
    fn level_1(&mut self) {
        self.diamonds_needed = 67;
        self.exit = Pos::new(COLS, 19);
        self.set(COLS, 18, Tile::Boulder);
        self.scatter(100, Tile::Diamond);
        self.scatter(70, Tile::Boulder);
        for col in [8, 15, 23] {
            self.wall_down(col, 2, 18);
        }
        for col in [8, 15, 23] {
            self.fill(col, 9, 1, 4, Tile::Earth);
        }
        self.place_man(2, 2);
    }

    /// Level 1's walls with a corridor across the middle and 3 creatures.
    fn level_2(&mut self) {
        self.diamonds_needed = 35;
        self.exit = Pos::new(COLS, 19);
        self.set(COLS, 18, Tile::Boulder);
        self.scatter(50, Tile::Diamond);
        self.scatter(30, Tile::Boulder);
        self.creature_count = 3;
        for col in [8, 15, 23] {
            self.wall_down(col, 2, 18);
        }
        self.place_creature(1, 10, 9);
        self.place_creature(2, 14, 9);
        self.place_creature(3, 20, 9);
        self.fill(8, 9, 16, 4, Tile::Empty);
        self.place_man(2, 2);
    }

    /// A diamond vault ringed by 8 creatures, beside a block of boulders.
    fn level_3(&mut self) {
        self.diamonds_needed = 50;
        self.exit = Pos::new(COLS, 19);
        self.set(COLS, 18, Tile::Boulder);
        self.fill_all(Tile::Earth);
        self.place_man(2, 9);
        self.creature_count = 8;
        for (id, col, row) in [
            (1, 5, 3),
            (3, 10, 3),
            (5, 5, 8),
            (7, 10, 8),
            (2, 5, 13),
            (4, 10, 13),
            (6, 5, 18),
            (8, 10, 18),
        ] {
            self.place_creature(id, col, row);
        }
        self.fill(4, 2, 8, 19, Tile::Empty);
        self.fill(5, 3, 6, 16, Tile::Earth);
        self.fill(6, 4, 4, 14, Tile::Diamond);
        self.fill(13, 3, 17, 16, Tile::Boulder);
    }

    /// 14 creatures in pockets above and below a line of boulders.
    fn level_4(&mut self) {
        self.diamonds_needed = 40;
        self.exit = Pos::new(COLS, 19);
        self.set(COLS, 18, Tile::Boulder);
        self.creature_count = 14;
        self.fill(9, 10, 15, 1, Tile::Boulder);
        for i in 1..=7 {
            let col = 2 * i + 8;
            self.place_creature(2 * i as usize - 1, col, 6);
            self.set(col, 7, Tile::Empty);
            self.place_creature(2 * i as usize, col, 13);
            self.set(col, 14, Tile::Empty);
        }
        self.place_man(2, 10);
    }

    /// 4 creatures in a room and no diamonds: they have to be blown up.  The
    /// exit is in the middle of the room.
    fn level_5(&mut self) {
        self.diamonds_needed = 20;
        self.exit = Pos::new(15, 10);
        self.set(COLS, 18, Tile::Boulder);
        for col in [12, 14, 16, 18] {
            self.fill(col, 2, 1, 1, Tile::Boulder);
        }
        self.fill(12, 7, 7, 7, Tile::Empty);
        self.creature_count = 4;
        self.place_creature(1, 12, 7);
        self.place_creature(2, 18, 7);
        self.place_creature(3, 12, 13);
        self.place_creature(4, 18, 13);
        self.board.set(self.exit, Tile::Bounds);
        self.place_man(2, 10);
    }

    /// A block of boulders above the morph.
    fn level_6(&mut self) {
        self.diamonds_needed = 50;
        self.exit = Pos::new(COLS, ROWS - 3);
        self.set(COLS, ROWS - 4, Tile::Boulder);
        self.fill(5, 4, 10, 5, Tile::Boulder);
        let morph = Pos::new(15, 14);
        self.morph = Some(morph);
        self.board.set(morph, self.morph_tile);
        self.place_man(self.exit.col - 1, self.exit.row);
    }

    /// 30 creatures in open space, each under a diamond on a plug of earth.
    fn level_7(&mut self) {
        self.diamonds_needed = 50;
        self.exit = Pos::new(COLS, ROWS - 3);
        self.set(COLS, ROWS - 4, Tile::Boulder);
        self.fill(2, 2, 30, 20, Tile::Empty);
        self.creature_count = 30;
        for i in 1..=10 {
            let col = 2 + i * 2;
            for (row, id) in [(6, i * 3 - 1), (12, i * 3 - 2), (18, i * 3)] {
                self.fill(col, row - 2, 1, 1, Tile::Diamond);
                self.fill(col, row - 1, 1, 1, Tile::Earth);
                self.place_creature(id as usize, col, row);
            }
        }
        self.place_man(self.exit.col - 1, self.exit.row);
    }

    /// 50 creatures in open space, each under a boulder on a plug of earth.
    fn level_8(&mut self) {
        self.diamonds_needed = 50;
        self.exit = Pos::new(COLS, 10);
        self.set(COLS, 9, Tile::Boulder);
        self.fill(2, 2, 30, 20, Tile::Empty);
        self.creature_count = 50;
        for i in 1..=10 {
            let col = 2 + i * 2;
            for (n, row) in [(4, 6), (3, 9), (2, 12), (1, 15), (0, 18)] {
                self.fill(col, row - 2, 1, 1, Tile::Boulder);
                self.fill(col, row - 1, 1, 1, Tile::Earth);
                self.place_creature((i * 5 - n) as usize, col, row);
            }
        }
        self.place_man(self.exit.col - 1, 10);
    }

    /// Two diamonds behind a wall, guarded by 12 creatures in corridors.
    fn level_9(&mut self) {
        self.diamonds_needed = 2;
        self.exit = Pos::new(COLS, 10);
        self.creature_count = 12;
        for (row, (a, a_col), (b, b_col)) in [
            (5, (11, 15), (12, 21)),
            (7, (1, 11), (6, 21)),
            (9, (2, 12), (7, 20)),
            (11, (3, 13), (8, 19)),
            (13, (4, 14), (9, 18)),
            (15, (5, 15), (10, 17)),
        ] {
            self.fill(11, row, 5, 1, Tile::Empty);
            self.place_creature(a, a_col, row);
            self.place_creature(b, b_col, row);
        }
        self.fill(17, 2, 5, 20, Tile::Empty);
        self.wall_down(16, 2, 18);
        self.wall_down(17, 10, 1);
        self.fill(16, 10, 1, 1, Tile::Diamond);
        self.fill(COLS - 1, 9, 1, 1, Tile::Boulder);
        self.place_man(2, 10);
    }

    // ----- building blocks ---------------------------------------------

    fn set(&mut self, col: i32, row: i32, t: Tile) {
        self.board.set(Pos::new(col, row), t);
    }

    fn fill_all(&mut self, t: Tile) {
        for p in Board::positions() {
            self.board.set(p, t);
        }
    }

    /// A vertical wall `len` cells long, going down from (col, row).
    fn wall_down(&mut self, col: i32, row: i32, len: i32) {
        for r in row..row + len {
            self.set(col, r, Tile::Wall);
        }
    }

    /// Put `t` at `count` random cells, using the original's generator and
    /// order: the row is drawn before the column.
    fn scatter(&mut self, count: usize, t: Tile) {
        for _ in 0..count {
            let row = self.rng.random((ROWS - 1) as u16) + 1;
            let col = self.rng.random((COLS - 1) as u16) + 1;
            self.set(col, row, t);
        }
    }

    /// Fill a `width` x `depth` rectangle at (col, row) with `t`, kept off
    /// the Bounds wall (`Set_Square`).  Filling over the man kills him.
    pub(super) fn fill(&mut self, col: i32, row: i32, width: i32, depth: i32, t: Tile) {
        let first_col = if col == 1 { 2 } else { col };
        let first_row = if row == 1 { 2 } else { row };
        let last_col = (col + width - 1).min(COLS - 1);
        let last_row = (row + depth - 1).min(ROWS - 1);
        for c in first_col..=last_col {
            for r in first_row..=last_row {
                let p = Pos::new(c, r);
                if self.board[p].is_man() {
                    self.just_died = true;
                    self.alive = false;
                }
                self.put(p, t);
            }
        }
    }
}
