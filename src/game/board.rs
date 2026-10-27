//! The 31 x 20 board and positions on it.

use super::tile::Tile;
use std::ops::{Add, Index};

pub const COLS: i32 = 31;
pub const ROWS: i32 = 20;

/// A cell position.  Columns run 1..=31 and rows 1..=20, as in the Pascal,
/// with the outer ring of cells being the `Bounds` wall.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Pos {
    pub col: i32,
    pub row: i32,
}

impl Pos {
    pub const fn new(col: i32, row: i32) -> Self {
        Pos { col, row }
    }

    pub fn up(self) -> Pos {
        self + (0, -1)
    }

    pub fn down(self) -> Pos {
        self + (0, 1)
    }
}

impl Add<(i32, i32)> for Pos {
    type Output = Pos;
    fn add(self, (dc, dr): (i32, i32)) -> Pos {
        Pos::new(self.col + dc, self.row + dr)
    }
}

/// The four directions, in the original's `Delta_State` order (which the
/// creatures' wall-following depends on).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Left,
    Up,
    Right,
    Down,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Left, Dir::Up, Dir::Right, Dir::Down];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Left => (-1, 0),
            Dir::Up => (0, -1),
            Dir::Right => (1, 0),
            Dir::Down => (0, 1),
        }
    }
}

/// Padding around the board on each side, so that a neighbour of an edge
/// cell is still a valid index.
const PAD: i32 = 1;
const STRIDE: usize = (COLS + 2 * PAD) as usize;
const LEN: usize = STRIDE * (ROWS + 2 * PAD) as usize;

/// The map: one tile per cell, plus a ring of `Outside` cells around it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board {
    cells: [Tile; LEN],
}

impl Board {
    pub fn new() -> Self {
        Board {
            cells: [Tile::Outside; LEN],
        }
    }

    fn index_of(p: Pos) -> Option<usize> {
        let (c, r) = (p.col + PAD, p.row + PAD);
        let in_range = (0..STRIDE as i32).contains(&c) && (0..(ROWS + 2 * PAD)).contains(&r);
        in_range.then(|| r as usize * STRIDE + c as usize)
    }

    /// Sets a cell.  Positions off the board (beyond the padding) are ignored.
    pub fn set(&mut self, p: Pos, t: Tile) {
        if let Some(i) = Self::index_of(p) {
            self.cells[i] = t;
        }
    }

    /// Every on-board position, column by column.
    pub fn positions() -> impl Iterator<Item = Pos> {
        (1..=COLS).flat_map(|c| (1..=ROWS).map(move |r| Pos::new(c, r)))
    }
}

impl Index<Pos> for Board {
    type Output = Tile;

    /// Positions off the board read as `Outside`.
    fn index(&self, p: Pos) -> &Tile {
        Self::index_of(p).map_or(&Tile::Outside, |i| &self.cells[i])
    }
}
