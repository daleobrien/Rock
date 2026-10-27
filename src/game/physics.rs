//! Falling and rolling boulders and diamonds, and the morph (`Drop_all`).

use super::board::{Pos, COLS, ROWS};
use super::tile::Tile;
use super::Game;

impl Game {
    /// One tick of gravity.  The board is swept column by column, bottom to
    /// top, so an object moved sideways into the next column can move again
    /// in the same sweep, just as in the original.
    pub(super) fn drop_all(&mut self) {
        for col in 1..=COLS {
            for row in (1..ROWS).rev() {
                let p = Pos::new(col, row);

                if self.board[p].is_resting() {
                    self.fall_or_roll(p);
                }

                let t = self.board[p];
                if t.is_falling() {
                    let below = self.board[p.down()];
                    if below.is_round_or_hard() || below == Tile::Earth {
                        self.put(p, t.landed());
                    } else {
                        self.fall_or_roll(p);
                    }
                }

                self.morph_at(p);
            }
        }
    }

    /// Drop into empty space below, or else roll off something round or hard.
    fn fall_or_roll(&mut self, p: Pos) {
        if self.board[p.down()] == Tile::Empty {
            self.fall(p);
        } else if self.can_roll(p) {
            self.roll(p);
        }
    }

    /// Sitting on something round or hard, with space to the left or right
    /// and below that.
    fn can_roll(&self, p: Pos) -> bool {
        let clear =
            |dc| self.board[p + (dc, 0)] == Tile::Empty && self.board[p + (dc, 1)] == Tile::Empty;
        self.board[p.down()].is_round_or_hard() && (clear(-1) || clear(1))
    }

    fn fall(&mut self, p: Pos) {
        let t = self.board[p];
        self.board.set(p.down(), t);
        self.put(p, Tile::Empty);
        self.put(p.down(), t.falling());
    }

    /// Roll sideways, choosing a side at random and taking the other one if
    /// that side is blocked.
    fn roll(&mut self, p: Pos) {
        let side = if self.rng.random(2) == 0 { 1 } else { -1 };
        let clear =
            self.board[p + (side, 1)] == Tile::Empty && self.board[p + (side, 0)] == Tile::Empty;
        let to = p + (if clear { side } else { -side }, 0);
        self.put(to, self.board[p].falling());
        self.put(p, Tile::Empty);
    }

    /// A boulder landing on the morph comes out below as a diamond, and a
    /// diamond as a boulder (`Rock_on_Morph`).  Whatever was below is
    /// replaced.
    fn morph_at(&mut self, p: Pos) {
        if !(3..ROWS - 1).contains(&p.row) || self.board[p] != self.morph_tile {
            return;
        }
        if self.board[p.up()].is_boulder() {
            self.put(p.up(), Tile::Empty);
            self.put(p.down(), Tile::FDiamond);
        }
        if self.board[p.up()].is_diamond() {
            self.put(p.up(), Tile::Empty);
            self.put(p.down(), Tile::FBoulder);
        }
    }
}
