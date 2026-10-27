//! What a cell of the board holds.
//!
//! The original stored bitmap handles in its map and compared handles, so
//! every bitmap is its own kind of cell: a boulder that has never moved
//! (`Boulder`), one that has landed (`FBoulder`) and one that is falling
//! (`FnBoulder`) are different tiles with different pictures.

/// One cell of the board.  The variant names follow the bitmap names in
/// ROCK20.RES.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Boulder,
    FBoulder,
    FnBoulder,
    Diamond,
    FDiamond,
    FnDiamond,
    Man1,
    Man2,
    Empty,
    Earth,
    Wall,
    Bounds,
    Door,
    /// Morph1..Morph4 (0..=3), the magic wall.
    Morph(u8),
    /// Life0..Life9 (0..=9), the creatures' animation frames.
    Life(u8),
    /// Off the board (the padding around it).  Never drawn.
    Outside,
}

impl Tile {
    /// How many distinct sprites the tiles use.
    pub const SPRITES: usize = 27;

    /// This tile's index into [`crate::game::Sprites`], if it has a picture.
    pub fn sprite(self) -> Option<usize> {
        use Tile::*;
        Some(match self {
            Boulder => 0,
            FBoulder => 1,
            FnBoulder => 2,
            Diamond => 3,
            FDiamond => 4,
            FnDiamond => 5,
            Man1 => 6,
            Man2 => 7,
            Empty => 8,
            Earth => 9,
            Wall => 10,
            Bounds => 11,
            Door => 12,
            Morph(n) => 13 + n as usize,
            Life(n) => 17 + n as usize,
            Outside => return None,
        })
    }

    pub fn is_man(self) -> bool {
        matches!(self, Tile::Man1 | Tile::Man2)
    }

    /// A boulder or diamond sitting still (`Still`).
    pub fn is_resting(self) -> bool {
        use Tile::*;
        matches!(self, Boulder | FBoulder | Diamond | FDiamond)
    }

    /// A boulder or diamond on its way down (`its_moving`).
    pub fn is_falling(self) -> bool {
        matches!(self, Tile::FnBoulder | Tile::FnDiamond)
    }

    /// Something a falling or resting object can sit on and roll off
    /// (`on_a_Diamond_or_Rock`).
    pub fn is_round_or_hard(self) -> bool {
        use Tile::*;
        matches!(
            self,
            Boulder | Diamond | FBoulder | FDiamond | Bounds | Wall
        )
    }

    /// A diamond in any state (`Eatible_Diamond`).
    pub fn is_diamond(self) -> bool {
        matches!(self, Tile::Diamond | Tile::FDiamond | Tile::FnDiamond)
    }

    /// A boulder in any state (`Is_Rock`).
    pub fn is_boulder(self) -> bool {
        matches!(self, Tile::Boulder | Tile::FBoulder | Tile::FnBoulder)
    }

    /// A boulder the man can push (a falling one can't be pushed).
    pub fn is_pushable(self) -> bool {
        matches!(self, Tile::Boulder | Tile::FBoulder)
    }

    /// Somewhere the man can walk or dig into (`Space_clear`).  Falling
    /// diamonds count; falling boulders don't.
    pub fn is_passable(self) -> bool {
        use Tile::*;
        matches!(self, Earth | Empty | FDiamond | Diamond | FnDiamond | Door)
    }

    /// Where a creature can go: empty space, or the man (`Empty_or_Man_or_life`).
    pub fn is_open_to_creature(self) -> bool {
        self == Tile::Empty || self.is_man()
    }

    /// The falling version of a boulder or diamond (`Change_to_Falling`).
    pub fn falling(self) -> Tile {
        use Tile::*;
        match self {
            Boulder | FBoulder => FnBoulder,
            Diamond | FDiamond => FnDiamond,
            other => other,
        }
    }

    /// The landed version of a boulder or diamond (`Change_to_Rest`).
    pub fn landed(self) -> Tile {
        use Tile::*;
        match self {
            Boulder | FnBoulder => FBoulder,
            Diamond | FnDiamond => FDiamond,
            other => other,
        }
    }
}
