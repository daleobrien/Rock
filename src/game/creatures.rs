//! The "life" creatures: wall-followers that kill the man on contact and
//! turn into diamonds when crushed or trapped (`Lifemove`, `Explode`).

use super::board::{Dir, Pos};
use super::tile::Tile;
use super::Game;

/// How many creature records there are (the Pascal's `Life[1..100]`).
pub const MAX: usize = 100;

/// Timer ticks a creature survives while boxed in.
const PATIENCE: i32 = 30;

#[derive(Clone, Copy, Default, Debug)]
pub struct Creature {
    pub pos: Pos,
    /// Where it was before its last move; redrawn every tick.
    pub prev: Pos,
    /// Direction it last tried, 1..=4 as an index into [`Dir::ALL`] plus 1.
    /// Briefly 5 after some turns; it is wrapped before it is used.
    pub side: i32,
    pub dead: bool,
    /// Animation frame, 0..=9.
    pub frame: u8,
    /// Ticks left before it gives up when boxed in.
    pub patience: i32,
}

impl Game {
    /// Place creature number `id` (1-based; its parity decides which way it
    /// turns).  Its `prev` position is carried over from whatever the record
    /// last held, as in the original.
    pub(super) fn place_creature(&mut self, id: usize, col: i32, row: i32) {
        let pos = Pos::new(col, row);
        let frame = (id % 9) as u8;
        let c = &mut self.creatures[id - 1];
        c.pos = pos;
        c.side = 1;
        c.frame = frame;
        c.dead = false;
        c.patience = PATIENCE;
        self.board.set(pos, Tile::Life(frame));
    }

    pub(super) fn move_creatures(&mut self) {
        for i in 0..self.creature_count {
            self.maybe_explode(i);
            if self.creatures[i].dead {
                continue;
            }

            // Odd creatures turn one way, even ones the other; and every
            // other pair turns back towards the wall after moving.
            let id = i as i32 + 1;
            let k = id % 2;
            let m = if (1 + id) % 4 > 1 { 0 } else { 1 };

            for _ in 0..4 {
                let c = &mut self.creatures[i];
                c.side = wrap_side(c.side + 1 - 2 * k);
                let from = c.pos;
                let to = from + Dir::ALL[(c.side - 1) as usize].delta();
                if !self.board[to].is_open_to_creature() {
                    continue;
                }
                if self.board[to].is_man() {
                    self.alive = false;
                    self.just_died = true;
                }
                self.board.set(to, self.board[from]);
                self.board.set(from, Tile::Empty);
                let c = &mut self.creatures[i];
                c.prev = from;
                c.pos = to;
                c.side -= 2 - m * (1 + k * 2);
                if c.side < 1 {
                    c.side += 4;
                }
                break;
            }

            let c = &mut self.creatures[i];
            c.frame = if c.frame > 8 { 0 } else { c.frame + 1 };
            let (pos, prev, frame) = (c.pos, c.prev, c.frame);
            self.board.set(pos, Tile::Life(frame));
            self.draw(prev);
            self.draw(pos);
        }
    }

    /// A creature with something falling on it, or one that has been boxed
    /// in for too long, becomes a 3 x 3 block of diamonds.
    fn maybe_explode(&mut self, i: usize) {
        let Creature { pos, dead, .. } = self.creatures[i];
        if dead {
            return;
        }
        let crushed = self.board[pos.up()].is_falling();
        let starved = !crushed && self.boxed_in(pos) && {
            let c = &mut self.creatures[i];
            c.patience -= 1;
            c.patience == 0
        };
        if crushed || starved {
            self.fill(pos.col - 1, pos.row - 1, 3, 3, Tile::FDiamond);
            self.creatures[i].dead = true;
        }
    }

    fn boxed_in(&self, p: Pos) -> bool {
        Dir::ALL
            .iter()
            .all(|d| self.board[p + d.delta()] != Tile::Empty)
    }
}

/// Keep a side number in 1..=4.
fn wrap_side(side: i32) -> i32 {
    match side {
        s if s < 1 => s + 4,
        s if s > 4 => s - 4,
        s => s,
    }
}
