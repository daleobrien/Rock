//! The nine levels (`Init_Screen`), read from the level files in `levels/`.
//!
//! The files are compiled into the binary.  `levels/README.md` describes
//! the format; in short, a level is a few `keyword value` lines and then a
//! picture of the board:
//!
//! ```text
//! diamonds 35            // how many to collect before the door opens
//! random-draws 160       // optional
//! creature 10,9 hidden   // one line per creature, in the order they move
//! map
//! ###############################
//! #M.....=.*....=.......=.......#
//! ...
//! ```

use super::board::{Board, Pos, COLS, ROWS};
use super::creatures;
use super::tile::Tile;
use super::Game;

/// Every level re-seeds Turbo Pascal's generator with this, so the
/// "random" diamonds and boulders always land in the same places.
const LEVEL_SEED: u32 = 0xFFFF;

const LEVEL_FILES: [&str; 9] = [
    include_str!("../../levels/level1.txt"),
    include_str!("../../levels/level2.txt"),
    include_str!("../../levels/level3.txt"),
    include_str!("../../levels/level4.txt"),
    include_str!("../../levels/level5.txt"),
    include_str!("../../levels/level6.txt"),
    include_str!("../../levels/level7.txt"),
    include_str!("../../levels/level8.txt"),
    include_str!("../../levels/level9.txt"),
];

/// A level as read from its file.
#[derive(Debug)]
pub(super) struct Level {
    pub diamonds_needed: i32,
    /// How many numbers the original drew from `Random` while building the
    /// level.  The game's later random numbers carry on from there.
    pub random_draws: u32,
    /// The board, with empty space under the visible creatures and the
    /// morph.
    pub board: Board,
    pub man: Pos,
    pub exit: Pos,
    pub morph: Option<Pos>,
    pub creatures: Vec<Placed>,
}

/// A creature's starting cell.  A hidden one isn't drawn until it first
/// moves: the original built some levels over their creatures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Placed {
    pub pos: Pos,
    pub hidden: bool,
}

impl Level {
    /// Level `n` from `levels/`.  Anything outside 1..=9 gets level 1.
    pub fn get(n: i32) -> Level {
        let n = if (1..=9).contains(&n) { n } else { 1 };
        Level::parse(LEVEL_FILES[n as usize - 1])
            .unwrap_or_else(|e| panic!("levels/level{n}.txt: {e}"))
    }

    pub fn parse(text: &str) -> Result<Level, String> {
        let mut lines = text.lines().enumerate().map(|(i, l)| (i + 1, l));
        let mut diamonds_needed = None;
        let mut random_draws = 0;
        let mut creatures = Vec::new();

        // The settings, up to the `map` line.
        loop {
            let (n, line) = lines.next().ok_or("no `map` line")?;
            let words: Vec<&str> = strip_comment(line).split_whitespace().collect();
            let number = |s: &str| s.parse().map_err(|_| format!("line {n}: bad number `{s}`"));
            match words[..] {
                [] => {}
                ["map"] => break,
                ["diamonds", d] => diamonds_needed = Some(number(d)?),
                ["random-draws", d] => {
                    random_draws = u32::try_from(number(d)?)
                        .map_err(|_| format!("line {n}: `random-draws` can't be negative"))?
                }
                ["creature", at] | ["creature", at, "hidden"] => {
                    let pos = parse_pos(at).ok_or(format!("line {n}: bad cell `{at}`"))?;
                    let hidden = words.len() == 3;
                    creatures.push(Placed { pos, hidden });
                }
                _ => return Err(format!("line {n}: don't understand `{}`", line.trim())),
            }
        }
        let diamonds_needed = diamonds_needed.ok_or("no `diamonds` line")?;
        if diamonds_needed < 1 {
            return Err("`diamonds` must be at least 1".into());
        }
        if creatures.len() > creatures::MAX {
            return Err(format!("more than {} creatures", creatures::MAX));
        }

        // The picture: one character per cell.
        let mut board = Board::new();
        let (mut man, mut exit, mut morph) = (None, None, None);
        let mut creature_cells = Vec::new();
        for row in 1..=ROWS {
            let (n, line) = lines
                .next()
                .ok_or(format!("the map has only {} rows", row - 1))?;
            let cells: Vec<char> = line.chars().collect();
            if cells.len() != COLS as usize {
                return Err(format!("line {n}: map rows must be {COLS} wide"));
            }
            for (col, ch) in (1..).zip(cells) {
                let p = Pos::new(col, row);
                let edge = col == 1 || col == COLS || row == 1 || row == ROWS;
                if edge && !matches!(ch, '#' | 'E') {
                    return Err(format!("line {n}: the edge must be `#` or `E`"));
                }
                let once = |slot: &mut Option<Pos>, what| match slot {
                    Some(_) => Err(format!("line {n}: more than one {what}")),
                    None => {
                        *slot = Some(p);
                        Ok(())
                    }
                };
                let tile = match ch {
                    '#' => Tile::Bounds,
                    '=' => Tile::Wall,
                    '.' => Tile::Earth,
                    ' ' => Tile::Empty,
                    'O' => Tile::Boulder,
                    '*' => Tile::Diamond,
                    'M' => {
                        once(&mut man, "man")?;
                        Tile::Man1
                    }
                    'E' => {
                        once(&mut exit, "exit")?;
                        Tile::Bounds
                    }
                    '%' => {
                        once(&mut morph, "morph")?;
                        Tile::Empty
                    }
                    '@' => {
                        creature_cells.push(p);
                        Tile::Empty
                    }
                    _ => return Err(format!("line {n}: unknown map character `{ch}`")),
                };
                board.set(p, tile);
            }
        }
        if let Some((n, line)) = lines.find(|(_, l)| !strip_comment(l).trim().is_empty()) {
            return Err(format!(
                "line {n}: unexpected `{}` after the map",
                line.trim()
            ));
        }

        // Each `@` needs exactly one visible creature, and hidden ones can't
        // be on an `@`.
        for c in &creatures {
            let on_at = creature_cells.contains(&c.pos);
            if c.hidden == on_at {
                let (col, row) = (c.pos.col, c.pos.row);
                return Err(match c.hidden {
                    true => format!("hidden creature at {col},{row} is on an `@`"),
                    false => format!("creature at {col},{row} isn't on an `@` (add `hidden`?)"),
                });
            }
        }
        for p in creature_cells {
            let visible = creatures.iter().filter(|c| !c.hidden && c.pos == p).count();
            if visible != 1 {
                return Err(format!(
                    "the `@` at {},{} has {visible} creature lines, not 1",
                    p.col, p.row
                ));
            }
        }

        Ok(Level {
            diamonds_needed,
            random_draws,
            board,
            man: man.ok_or("no man (`M`) on the map")?,
            exit: exit.ok_or("no exit (`E`) on the map")?,
            morph,
            creatures,
        })
    }
}

fn strip_comment(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

/// `col,row`, both counted from 1 at the top left.
fn parse_pos(s: &str) -> Option<Pos> {
    let (col, row) = s.split_once(',')?;
    let p = Pos::new(col.trim().parse().ok()?, row.trim().parse().ok()?);
    let inside = (2..COLS).contains(&p.col) && (2..ROWS).contains(&p.row);
    inside.then_some(p)
}

impl Game {
    /// Set up level `n`.  Anything outside 1..=9 gets level 1's layout.
    pub(super) fn init_level(&mut self, n: i32) {
        let level = Level::get(n);
        self.morph = None;
        self.just_died = false;
        self.exit_open = false;
        self.alive = true;
        self.rng.seed = LEVEL_SEED;
        self.rng.skip(level.random_draws);
        self.diamonds = 0;
        self.diamonds_needed = level.diamonds_needed;
        self.exit = level.exit;

        for p in Board::positions() {
            self.board.set(p, level.board[p]);
        }
        self.creature_count = level.creatures.len();
        for (id, c) in (1..).zip(&level.creatures) {
            self.place_creature(id, c.pos, c.hidden);
        }
        if let Some(p) = level.morph {
            self.morph = Some(p);
            self.board.set(p, self.morph_tile);
        }
        self.place_man(level.man.col, level.man.row);
    }
}
