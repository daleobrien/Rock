//! The man: moving, digging, pushing, and dying.

use super::board::{Dir, Pos};
use super::tile::Tile;
use super::{Game, Interrupt, Modal, Step};

impl Game {
    /// An arrow key.  The man digs through earth, collects diamonds and pushes
    /// boulders sideways into empty space; with `shift` he does it without
    /// moving.
    pub(super) fn move_man(&mut self, dir: Dir, shift: bool) {
        if !self.alive {
            return;
        }
        let d @ (dc, _) = dir.delta();
        let start = self.man;
        let mut from = start;
        let mut to = start + d;

        // Push a boulder sideways.
        if dc != 0 && self.board[to].is_pushable() && self.board[to + (dc, 0)] == Tile::Empty {
            self.board.set(to, self.board[from]);
            self.board.set(from, Tile::Empty);
            self.board.set(to + (dc, 0), Tile::FBoulder);
            if shift {
                self.board.set(from, self.board[to]);
                self.board.set(to, Tile::Empty);
                to = from;
            }
            self.draw(start);
            self.draw(start + d);
            self.draw(start + (2 * dc, 0));
            from = to;
        }

        // Dig, or step into the cell.
        if self.board[to].is_passable() {
            if self.board[to].is_diamond() {
                self.display_score(1);
            }
            self.board.set(to, self.board[from]);
            self.board.set(from, Tile::Empty);
            if shift {
                self.board.set(from, self.board[to]);
                self.board.set(to, Tile::Empty);
                to = from;
            }
            self.draw(from);
            self.draw(from + d);
        } else {
            to = from;
        }
        self.man = to;
    }

    /// Animate the man, and open the death box if something has fallen on
    /// him or a creature or explosion got him (`Check_if_man_still_happening`).
    pub(super) fn check_man(&mut self) -> Interrupt {
        let p = self.man;
        if self.alive {
            let t = if self.board[p] == Tile::Man1 {
                Tile::Man2
            } else {
                Tile::Man1
            };
            self.put(p, t);
            if self.board[p.up()].is_falling() {
                self.just_died = true;
                self.alive = false;
            }
        }
        if !std::mem::take(&mut self.just_died) {
            return None;
        }
        // Whatever killed him takes his place.
        self.board.set(p, self.board[p.up()]);
        self.put(p.up(), Tile::Empty);
        self.draw(p);
        Some((Modal::Death, vec![Step::AfterDeath, Step::NextLevel]))
    }

    pub(super) fn place_man(&mut self, col: i32, row: i32) {
        self.man = Pos::new(col, row);
        self.board.set(self.man, Tile::Man1);
    }
}
