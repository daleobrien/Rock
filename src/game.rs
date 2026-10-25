//! The game itself: a line-by-line port of ROCK_FUC.PAS / ROCK_VAR.PAS and the
//! message handlers in ROCK.PAS.
//!
//! The original drew straight onto the window's device context with `BitBlt`
//! every time a cell changed (`PlaceObject`) and never kept a picture of the
//! screen.  To keep its look exactly (including the flashing exit made with
//! `InvertRect`, and the score bar that a redrawn top-row cell can erase) this
//! port keeps a CPU framebuffer of the 620x400 client area and performs the
//! same blits into it.  The front end just shows that framebuffer.
//!
//! Coordinates are 1-based like the Pascal: `map[col][row]`, col 1..=31,
//! row 1..=20.  The array is padded by one cell on every side, so the
//! occasional out-of-range read or write in the original (for example
//! `morphchange` writing `Map[0,0]` on levels without a morph) lands somewhere
//! harmless instead of in neighbouring variables.

use crate::rng::TpRandom;
use std::collections::VecDeque;

pub const MAX_X: usize = 600;
pub const MAX_Y: usize = 400;
pub const MAX_ROW: i32 = 20;
pub const MAX_COL: i32 = 31;
pub const LAST_LEVEL: i32 = 9;
/// Client area the original drew into (31 columns x 20 rows of 20px cells).
pub const FB_W: usize = MAX_X + 20;
pub const FB_H: usize = MAX_Y;
pub const CELL: i32 = 20;

/// Delta_State[1..4]: left, up, right, down.  Index 0 is unused.
const DELTA_STATE: [(i32, i32); 5] = [(0, 0), (-1, 0), (0, -1), (1, 0), (0, 1)];

/// What a map cell holds.  The original stored bitmap handles in the map and
/// compared handles, so every bitmap is its own "kind" of cell.
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
    /// Morph1..Morph4 (0..=3)
    Morph(u8),
    /// Life0..Life9 (0..=9), the "nasty things"
    Life(u8),
    /// Outside the 31x20 playfield (the padding ring).
    Outside,
}

/// Decoded bitmaps as 0x00RRGGBB pixels.
pub struct Sprites {
    pub boulder: Vec<u32>,
    pub fboulder: Vec<u32>,
    pub fnboulder: Vec<u32>,
    pub diamond: Vec<u32>,
    pub fdiamond: Vec<u32>,
    pub fndiamond: Vec<u32>,
    pub man1: Vec<u32>,
    pub man2: Vec<u32>,
    pub empty: Vec<u32>,
    pub earth: Vec<u32>,
    pub wall: Vec<u32>,
    pub bounds: Vec<u32>,
    pub door: Vec<u32>,
    pub morph: [Vec<u32>; 4],
    pub life: [Vec<u32>; 10],
    /// 2x20 strips stretched into the score bar
    pub score1: Vec<u32>,
    pub score2: Vec<u32>,
}

impl Sprites {
    /// Solid-colour sprites, for tests.
    #[cfg(test)]
    pub fn blank() -> Self {
        let t = |c: u32| vec![c; 400];
        Sprites {
            boulder: t(1),
            fboulder: t(2),
            fnboulder: t(3),
            diamond: t(4),
            fdiamond: t(5),
            fndiamond: t(6),
            man1: t(7),
            man2: t(8),
            empty: t(9),
            earth: t(10),
            wall: t(11),
            bounds: t(12),
            door: t(13),
            morph: [t(14), t(15), t(16), t(17)],
            life: std::array::from_fn(|i| t(20 + i as u32)),
            score1: vec![30; 40],
            score2: vec![31; 40],
        }
    }

    fn tile(&self, t: Tile) -> Option<&[u32]> {
        Some(match t {
            Tile::Boulder => &self.boulder,
            Tile::FBoulder => &self.fboulder,
            Tile::FnBoulder => &self.fnboulder,
            Tile::Diamond => &self.diamond,
            Tile::FDiamond => &self.fdiamond,
            Tile::FnDiamond => &self.fndiamond,
            Tile::Man1 => &self.man1,
            Tile::Man2 => &self.man2,
            Tile::Empty => &self.empty,
            Tile::Earth => &self.earth,
            Tile::Wall => &self.wall,
            Tile::Bounds => &self.bounds,
            Tile::Door => &self.door,
            Tile::Morph(i) => &self.morph[i as usize],
            Tile::Life(i) => &self.life[i as usize],
            Tile::Outside => return None,
        })
    }
}

/// A modal message box or dialog the game wants shown.  The original called
/// `MessageBox`/`ExecDialog`, which block the handler that called them while
/// Windows keeps dispatching the 200ms timer to the game underneath.
#[derive(Clone, Debug, PartialEq)]
pub enum Modal {
    /// MessageBox(.., 'You are mine !', 'Death says ..')
    Death,
    /// MessageBox(.., 'All done', 'You have escaped Death')
    AllDone,
    /// MessageBox(.., '1993, Dale ', 'Beta Verison 1.0 - Rock')
    About,
    /// TInputDialog 'You Choose' / 'Enter 1..9'
    LevelInput,
}

impl Modal {
    pub fn title(&self) -> &'static str {
        match self {
            Modal::Death => "Death says ..",
            Modal::AllDone => "You have escaped Death",
            Modal::About => "Beta Verison 1.0 - Rock",
            Modal::LevelInput => "You Choose",
        }
    }
    pub fn text(&self) -> &'static str {
        match self {
            Modal::Death => "You are mine !",
            Modal::AllDone => "All done",
            Modal::About => "1993, Dale ",
            Modal::LevelInput => "Enter 1..9",
        }
    }
}

/// One step of a message handler.  Handlers are split into steps so that the
/// rest of a handler can be resumed after a modal box is dismissed.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    // TRock.Timer
    DropAll,
    CheckMan,
    FlashExit,
    LifeMove,
    MorphChange,
    // TRock.Keyboard
    Arrow { dir: usize, shift: bool },
    ExitCheck,
    // Check_if_man_still_happening, after the death box
    DeathRest,
    // Next_Screen and its tail after the 'All done' box
    NextScreen,
    WrapToLevel1,
    InitAndDraw,
    // TRock.Level, after the input dialog
    LevelRest,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ManPos {
    pub x: i32,
    pub y: i32,
    pub ox: i32,
    pub oy: i32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Morphic {
    pub x: i32,
    pub y: i32,
    /// Set to 0 by Set_morph and never used, in the original too.
    #[allow(dead_code)]
    pub state: i32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Life {
    pub x: i32,
    pub y: i32,
    pub side: i32,
    pub ox: i32,
    pub oy: i32,
    pub dead: bool,
    pub state: i32,
    pub length: i32,
}

pub struct Game {
    pub map: [[Tile; (MAX_ROW + 2) as usize]; (MAX_COL + 2) as usize],
    pub fb: Vec<u32>,
    sprites: Sprites,
    rng: TpRandom,

    pub alive: bool,
    pub sound: bool,
    pub just_died: bool,
    pub exit_available: bool,
    pub no_of_life: i32,
    pub no_of_required_diamonds: i32,
    pub exit: (i32, i32),
    pub screen_no: i32,
    pub man: ManPos,
    pub morph_no: i32,
    pub morphic: [Morphic; 101],
    pub life: [Life; 101],
    pub diamonds: i32,
    /// The current morph bitmap (`Morph` in the Pascal), compared by identity.
    pub morph: Tile,

    /// Text buffer behind the Level dialog.  The original passed a pointer to
    /// the string constant '1', so whatever was typed last time is what the
    /// dialog shows next time.
    pub level_text: String,

    /// Modal boxes currently open, innermost last, each with the handler steps
    /// that resume when it closes.
    modals: Vec<(Modal, VecDeque<Op>)>,
    /// Set whenever a step asks for MessageBeep (only if `sound`, which the
    /// original never turns on).
    pub beeps: u32,
}

impl Game {
    /// TRock.Init + the program's main block.
    pub fn new(sprites: Sprites) -> Self {
        let mut g = Game {
            map: [[Tile::Outside; (MAX_ROW + 2) as usize]; (MAX_COL + 2) as usize],
            fb: vec![0xFFFFFF; FB_W * FB_H],
            sprites,
            rng: TpRandom::new(0),
            alive: false,
            sound: false,
            just_died: false,
            exit_available: false,
            no_of_life: 0,
            no_of_required_diamonds: 1,
            exit: (0, 0),
            screen_no: 1,
            man: ManPos::default(),
            morph_no: 0,
            morphic: [Morphic::default(); 101],
            life: [Life::default(); 101],
            diamonds: 0,
            morph: Tile::Morph(0),
            level_text: "1".to_string(),
            modals: Vec::new(),
            beeps: 0,
        };
        g.init_screen(1);
        g.screen_no = 1;
        g
    }

    // ----- input from the front end -------------------------------------

    /// WM_PAINT for the whole client area.
    pub fn paint(&mut self) {
        for row in 1..=MAX_ROW {
            for col in 1..=MAX_COL {
                self.place_object(col, row);
            }
        }
        self.display_score(0);
    }

    /// WM_TIMER (every 200ms).  The game is paused while a box is open.  (In
    /// the original the timer kept running behind message boxes and dialogs.)
    pub fn timer(&mut self) {
        if !self.modals.is_empty() {
            return;
        }
        self.run(VecDeque::from([
            Op::DropAll,
            Op::CheckMan,
            Op::FlashExit,
            Op::LifeMove,
            Op::MorphChange,
        ]));
    }

    /// WM_KEYDOWN.  `arrow` is Some(1..=4) for Left, Up, Right, Down (the
    /// Pascal's `Msg.WParam - 36`).  Other keys are ignored.  (In the original
    /// any key ran the rest of the handler, making the man blink.)
    pub fn key_down(&mut self, arrow: Option<usize>, shift: bool) {
        let Some(dir) = arrow else {
            return;
        };
        if !self.modals.is_empty() {
            return;
        }
        self.run(VecDeque::from([
            Op::Arrow { dir, shift },
            Op::CheckMan,
            Op::ExitCheck,
        ]));
    }

    /// Menu: &About (command 100).
    pub fn menu_about(&mut self) {
        if self.modals.is_empty() {
            self.modals.push((Modal::About, VecDeque::new()));
        }
    }

    /// Menu: &Level (command 200).
    /// The dialog opens showing the current level.
    pub fn menu_level(&mut self) {
        if self.modals.is_empty() {
            self.level_text = self.screen_no.clamp(1, LAST_LEVEL).to_string();
            self.modals.push((
                Modal::LevelInput,
                VecDeque::from([Op::LevelRest, Op::NextScreen]),
            ));
        }
    }

    /// The modal box on top, if any.
    pub fn modal(&self) -> Option<&Modal> {
        self.modals.last().map(|(m, _)| m)
    }

    /// Dismiss the top modal box.  For the Level dialog, `ok_text` is the
    /// edit box contents on OK, or None on Cancel.  Cancel, or anything that
    /// isn't a level from 1 to 9, leaves the game alone.  (The original still
    /// changed level on Cancel and took any character typed.)
    pub fn close_modal(&mut self, ok_text: Option<String>) {
        let Some((modal, mut rest)) = self.modals.pop() else {
            return;
        };
        if modal == Modal::LevelInput {
            match ok_text
                .as_deref()
                .map(str::trim)
                .and_then(|t| t.parse::<i32>().ok())
            {
                Some(n) if (1..=LAST_LEVEL).contains(&n) => self.level_text = n.to_string(),
                _ => rest.clear(),
            }
        }
        self.run(rest);
        // Windows then repaints whatever the box was covering.
        self.paint();
    }

    // ----- handler step runner ------------------------------------------

    fn run(&mut self, mut ops: VecDeque<Op>) {
        while let Some(op) = ops.pop_front() {
            if let Some((modal, mut cont)) = self.step(op) {
                cont.extend(ops);
                self.modals.push((modal, cont));
                return;
            }
        }
    }

    /// Runs one step; returns a modal box to open plus the steps to run
    /// after it closes (ahead of whatever was left of the handler).
    fn step(&mut self, op: Op) -> Option<(Modal, VecDeque<Op>)> {
        match op {
            Op::DropAll => self.drop_all(),
            Op::CheckMan => return self.check_if_man_still_happening(),
            Op::FlashExit => self.flash_exit(),
            Op::LifeMove => self.lifemove(),
            Op::MorphChange => self.morphchange(),
            Op::Arrow { dir, shift } => self.arrow(dir, shift),
            Op::ExitCheck => {
                if self.in_exit() {
                    return self.next_screen();
                }
            }
            Op::DeathRest => {
                if self.sound {
                    self.beeps += 1;
                }
                self.screen_no -= 1;
                if self.screen_no < 0 {
                    self.screen_no = 0;
                }
                self.alive = true;
            }
            Op::NextScreen => return self.next_screen(),
            Op::WrapToLevel1 => self.screen_no = 1,
            Op::InitAndDraw => self.init_and_draw(),
            Op::LevelRest => {
                // Screen_no := Ord(Text[0]) - 49;  then Next_Screen adds 1.
                // close_modal has already checked the text is 1..9.
                self.screen_no = self.level_text.parse::<i32>().unwrap_or(1) - 1;
            }
        }
        None
    }

    // ----- drawing -----------------------------------------------------

    fn blit(&mut self, dx: i32, dy: i32, w: i32, h: i32, src: &[u32], sw: i32) {
        for y in 0..h {
            let fy = dy + y;
            if fy < 0 || fy >= FB_H as i32 {
                continue;
            }
            for x in 0..w {
                let fx = dx + x;
                if fx < 0 || fx >= FB_W as i32 {
                    continue;
                }
                self.fb[fy as usize * FB_W + fx as usize] = src[(y * sw + x) as usize];
            }
        }
    }

    /// PlaceObject: BitBlt the cell's bitmap to ((X-1)*20, (Y-1)*20).
    pub fn place_object(&mut self, x: i32, y: i32) {
        if !(0..=MAX_COL + 1).contains(&x) || !(0..=MAX_ROW + 1).contains(&y) {
            return;
        }
        let t = self.map[x as usize][y as usize];
        if let Some(src) = self.sprites.tile(t) {
            let src = src.to_vec();
            self.blit((x - 1) * CELL, (y - 1) * CELL, CELL, CELL, &src, CELL);
        }
    }

    /// StretchBlt of a 2x20 strip to w x 20 (nearest-pixel, as GDI does when
    /// enlarging).
    fn stretch_strip(&mut self, dx: i32, w: i32, src_is_score1: bool) {
        if w <= 0 {
            return;
        }
        let src = if src_is_score1 {
            self.sprites.score1.clone()
        } else {
            self.sprites.score2.clone()
        };
        for y in 0..20 {
            for x in 0..w {
                let sx = x * 2 / w;
                let fx = dx + x;
                if (0..FB_W as i32).contains(&fx) {
                    self.fb[y as usize * FB_W + fx as usize] = src[(y * 2 + sx) as usize];
                }
            }
        }
    }

    // ----- ROCK_FUC.PAS -------------------------------------------------

    fn m(&self, col: i32, row: i32) -> Tile {
        if (0..=MAX_COL + 1).contains(&col) && (0..=MAX_ROW + 1).contains(&row) {
            self.map[col as usize][row as usize]
        } else {
            Tile::Outside
        }
    }

    fn set(&mut self, col: i32, row: i32, t: Tile) {
        if (0..=MAX_COL + 1).contains(&col) && (0..=MAX_ROW + 1).contains(&row) {
            self.map[col as usize][row as usize] = t;
        }
    }

    fn still(&self, col: i32, row: i32) -> bool {
        matches!(
            self.m(col, row),
            Tile::Boulder | Tile::FBoulder | Tile::Diamond | Tile::FDiamond
        )
    }

    fn it_is_empty_below(&self, col: i32, row: i32) -> bool {
        self.m(col, row + 1) == Tile::Empty
    }

    fn a_clear_side(&self, col: i32, row: i32) -> bool {
        (self.m(col - 1, row + 1) == Tile::Empty && self.m(col - 1, row) == Tile::Empty)
            || (self.m(col + 1, row + 1) == Tile::Empty && self.m(col + 1, row) == Tile::Empty)
    }

    fn on_a_diamond_or_rock(&self, col: i32, row: i32) -> bool {
        matches!(
            self.m(col, row + 1),
            Tile::Boulder
                | Tile::Diamond
                | Tile::FBoulder
                | Tile::FDiamond
                | Tile::Bounds
                | Tile::Wall
        )
    }

    fn on_a_diamond_or_rock_or_earth(&self, col: i32, row: i32) -> bool {
        self.on_a_diamond_or_rock(col, row) || self.m(col, row + 1) == Tile::Earth
    }

    fn can_move_sideways(&self, col: i32, row: i32) -> bool {
        self.on_a_diamond_or_rock(col, row) && self.a_clear_side(col, row)
    }

    fn its_moving(&self, col: i32, row: i32) -> bool {
        matches!(self.m(col, row), Tile::FnBoulder | Tile::FnDiamond)
    }

    fn swap_vertical(&mut self, col: i32, row: i32) {
        let t = self.m(col, row);
        self.set(col, row + 1, t);
        self.set(col, row, Tile::Empty);
        self.place_object(col, row);
        self.change_to_falling(col, row + 1);
        self.place_object(col, row + 1);
    }

    fn swap_diag(&mut self, col: i32, row: i32) {
        // The original's `if Random(1) < 0.5` was always true, so things
        // always tried to roll right first.  Random(2) gives the 50/50 that
        // was intended.
        let side = if self.rng.random(2) == 0 { 1 } else { -1 };
        let t = self.m(col, row);
        if self.m(col + side, row + 1) == Tile::Empty && self.m(col + side, row) == Tile::Empty {
            self.set(col + side, row, t);
            self.change_to_falling(col + side, row);
            self.place_object(col + side, row);
        } else {
            self.set(col - side, row, t);
            self.change_to_falling(col - side, row);
            self.place_object(col - side, row);
        }
        self.set(col, row, Tile::Empty);
        self.place_object(col, row);
    }

    fn space_clear(&self, col: i32, row: i32) -> bool {
        matches!(
            self.m(col, row),
            Tile::Earth
                | Tile::Empty
                | Tile::FDiamond
                | Tile::Diamond
                | Tile::FnDiamond
                | Tile::Door
        )
    }

    pub fn display_score(&mut self, inc: i32) {
        if self.sound {
            self.beeps += 1;
        }
        self.diamonds += inc;
        if self.diamonds > self.no_of_required_diamonds {
            self.diamonds = self.no_of_required_diamonds;
        }
        if self.diamonds == self.no_of_required_diamonds {
            self.set(self.exit.0, self.exit.1, Tile::Door);
            self.place_object(self.exit.0, self.exit.1);
            self.exit_available = true;
        }
        // Pascal Round: half away from zero, same as f64::round.
        let temp =
            (self.diamonds as f64 * 196.0 / self.no_of_required_diamonds as f64).round() as i32;
        self.stretch_strip(103 + temp, 197 - temp, false);
        self.stretch_strip(103, temp, true);
    }

    fn eatible_diamond(&self, col: i32, row: i32) -> bool {
        matches!(
            self.m(col, row),
            Tile::Diamond | Tile::FDiamond | Tile::FnDiamond
        )
    }

    fn set_square(&mut self, a: i32, b: i32, width: i32, depth: i32, name: Tile) {
        let w = if a == 1 { 1 } else { 0 };
        let x = if a + width > MAX_COL {
            a + width - MAX_COL + 1
        } else {
            1
        };
        let y = if b == 1 { 1 } else { 0 };
        let z = if b + depth > MAX_ROW {
            b + depth - MAX_ROW + 1
        } else {
            1
        };
        for i in w..=(width - x) {
            for j in y..=(depth - z) {
                if matches!(self.m(a + i, b + j), Tile::Man1 | Tile::Man2) {
                    self.just_died = true;
                    self.alive = false;
                }
                self.set(a + i, b + j, name);
                self.place_object(a + i, b + j);
            }
        }
    }

    fn boxed_in(&self, col: i32, row: i32) -> bool {
        self.m(col - 1, row) != Tile::Empty
            && self.m(col + 1, row) != Tile::Empty
            && self.m(col, row - 1) != Tile::Empty
            && self.m(col, row + 1) != Tile::Empty
    }

    fn check_if_fng_object_above(&self, i: usize) -> bool {
        matches!(
            self.m(self.life[i].x, self.life[i].y - 1),
            Tile::FnDiamond | Tile::FnBoulder
        )
    }

    fn explode(&mut self, index: usize) {
        let col = self.life[index].x;
        let row = self.life[index].y;
        if self.check_if_fng_object_above(index) && !self.life[index].dead {
            self.set_square(col - 1, row - 1, 3, 3, Tile::FDiamond);
            self.life[index].dead = true;
        }
        if self.boxed_in(col, row) && !self.life[index].dead {
            self.life[index].length -= 1;
            if self.life[index].length == 0 {
                self.set_square(col - 1, row - 1, 3, 3, Tile::FDiamond);
                self.life[index].dead = true;
            }
        }
    }

    fn empty_or_man_or_life(&self, col: i32, row: i32) -> bool {
        matches!(self.m(col, row), Tile::Man1 | Tile::Man2 | Tile::Empty)
    }

    fn lifemove(&mut self) {
        for index in 1..=self.no_of_life.clamp(0, 100) as usize {
            self.explode(index);
            if self.life[index].dead {
                continue;
            }
            let mut j = 0;
            let k = index as i32 % 2;
            let m = if ((1 + index as i32) % 4) > 1 { 0 } else { 1 };
            loop {
                let l = &mut self.life[index];
                l.side = l.side + 1 - 2 * k;
                if l.side < 1 {
                    l.side += 4;
                }
                if l.side > 4 {
                    l.side -= 4;
                }
                let (dx, dy) = DELTA_STATE[l.side as usize];
                let (x, y) = (l.x, l.y);
                if self.empty_or_man_or_life(x + dx, y + dy) {
                    if matches!(self.m(x + dx, y + dy), Tile::Man1 | Tile::Man2) {
                        self.alive = false;
                        self.just_died = true;
                        self.set(x + dx, y + dy, Tile::Empty);
                    }
                    let t = self.m(x, y);
                    self.set(x + dx, y + dy, t);
                    self.set(x, y, Tile::Empty);
                    let l = &mut self.life[index];
                    l.ox = x;
                    l.oy = y;
                    l.x = x + dx;
                    l.y = y + dy;
                    // -1 = straight ahead, -2 RH, or +1 straight, -2 LH
                    l.side = l.side - 2 + m * (1 + k * 2);
                    if l.side < 1 {
                        l.side += 4;
                    }
                    j = 3;
                }
                j += 1;
                if j > 3 {
                    break;
                }
            }
            let l = &mut self.life[index];
            if l.state > 8 {
                l.state = 0;
            } else {
                l.state += 1;
            }
            let (x, y, ox, oy, st) = (l.x, l.y, l.ox, l.oy, l.state);
            self.set(x, y, Tile::Life(st as u8));
            self.place_object(ox, oy);
            self.place_object(x, y);
        }
    }

    fn change_to_rest(&mut self, col: i32, row: i32) {
        let t = match self.m(col, row) {
            Tile::FnBoulder | Tile::Boulder => Tile::FBoulder,
            Tile::FnDiamond | Tile::Diamond => Tile::FDiamond,
            other => other,
        };
        self.set(col, row, t);
    }

    fn change_to_falling(&mut self, col: i32, row: i32) {
        let t = match self.m(col, row) {
            Tile::Boulder | Tile::FBoulder => Tile::FnBoulder,
            Tile::Diamond | Tile::FDiamond => Tile::FnDiamond,
            other => other,
        };
        self.set(col, row, t);
    }

    fn set_bounds(&mut self) {
        for i in 1..=MAX_ROW {
            self.set(1, i, Tile::Bounds);
            self.set(MAX_COL, i, Tile::Bounds);
        }
        for i in 1..=MAX_COL {
            self.set(i, 1, Tile::Bounds);
            self.set(i, MAX_ROW, Tile::Bounds);
        }
    }

    fn set_wall(&mut self, a: i32, b: i32, length: i32, horizontal: bool) {
        for i in 0..length {
            if horizontal {
                self.set(a + i, b, Tile::Wall);
            } else {
                self.set(a, b + i, Tile::Wall);
            }
        }
    }

    fn set_all_earth(&mut self) {
        for row in 1..=MAX_ROW {
            for col in 1..=MAX_COL {
                self.set(col, row, Tile::Earth);
            }
        }
    }

    fn set_morph(&mut self, col: i32, row: i32, index: usize) {
        self.set(col, row, self.morph);
        self.morphic[index] = Morphic {
            x: col,
            y: row,
            state: 0,
        };
    }

    fn set_life(&mut self, i: usize, a: i32, b: i32) {
        let l = &mut self.life[i];
        l.x = a;
        l.y = b;
        l.side = 1;
        l.state = i as i32 % 9;
        l.dead = false;
        l.length = 30;
        self.set(a, b, Tile::Life((i % 9) as u8));
    }

    fn set_man(&mut self, a: i32, b: i32) {
        self.set(a, b, Tile::Man1);
        self.man.x = a;
        self.man.y = b;
    }

    /// `Map[Random(Max_Col-1)+1, Random(Max_row-1)+1] := t`.  ROCK.EXE
    /// evaluates the row index first, then the column.
    fn scatter(&mut self, t: Tile) {
        let row = self.rng.random((MAX_ROW - 1) as u16) + 1;
        let col = self.rng.random((MAX_COL - 1) as u16) + 1;
        self.set(col, row, t);
    }

    #[allow(clippy::manual_range_contains)] // conditions kept as in the Pascal
    pub fn init_screen(&mut self, index: i32) {
        self.no_of_life = 0;
        self.morph_no = 0;
        self.morphic[0] = Morphic::default();
        self.just_died = false;
        self.exit_available = false;
        self.alive = true;
        self.sound = false;
        self.rng.seed = 0xFFFF;
        self.diamonds = 0;
        self.set_all_earth();
        let max_col = MAX_COL;
        let max_row = MAX_ROW;

        if index == 1 || index > LAST_LEVEL || index < 1 {
            // No life just get all rocks
            self.no_of_life = 0;
            self.set_all_earth();
            self.no_of_required_diamonds = 67;
            self.exit = (max_col, 19);
            self.set(max_col, 18, Tile::Boulder);
            for _ in 1..=100 {
                self.scatter(Tile::Diamond);
            }
            for _ in 1..=70 {
                self.scatter(Tile::Boulder);
            }
            self.set_wall(8, 2, 18, false);
            self.set_wall(15, 2, 18, false);
            self.set_wall(23, 2, 18, false);
            self.set_square(8, 9, 1, 4, Tile::Earth);
            self.set_square(15, 9, 1, 4, Tile::Earth);
            self.set_square(23, 9, 1, 4, Tile::Earth);
            self.set_man(2, 2);
        }

        if index == 2 {
            self.exit_available = false;
            self.set_all_earth();
            self.no_of_required_diamonds = 35;
            self.exit = (max_col, 19);
            self.set(max_col, 18, Tile::Boulder);
            for _ in 1..=50 {
                self.scatter(Tile::Diamond);
            }
            for _ in 1..=30 {
                self.scatter(Tile::Boulder);
            }
            self.no_of_life = 3;
            self.set_wall(8, 2, 18, false);
            self.set_wall(15, 2, 18, false);
            self.set_wall(23, 2, 18, false);
            self.set_life(1, 10, 9);
            self.set_life(2, 14, 9);
            self.set_life(3, 20, 9);
            self.set_square(8, 9, 16, 4, Tile::Empty);
            self.set_man(2, 2);
        }

        if index == 3 {
            self.no_of_required_diamonds = 50;
            self.exit = (max_col, 19);
            self.set(max_col, 18, Tile::Boulder);
            self.set_all_earth();
            self.set_man(2, 9);
            self.no_of_life = 8;
            self.set_life(1, 5, 3);
            self.set_life(3, 10, 3);
            self.set_life(5, 5, 8);
            self.set_life(7, 10, 8);
            self.set_life(2, 5, 13);
            self.set_life(4, 10, 13);
            self.set_life(6, 5, 18);
            self.set_life(8, 10, 18);
            self.set_square(4, 2, 8, 19, Tile::Empty);
            self.set_square(5, 3, 6, 16, Tile::Earth);
            self.set_square(6, 4, 4, 14, Tile::Diamond);
            self.set_square(13, 3, 17, 16, Tile::Boulder);
        }

        if index == 4 {
            self.no_of_required_diamonds = 40;
            self.exit = (max_col, 19);
            self.set(max_col, 18, Tile::Boulder);
            self.no_of_life = 14;
            self.set_square(9, 10, 15, 1, Tile::Boulder);
            for i in 1..=7 {
                self.set_life((i * 2 - 1) as usize, 2 * i + 8, 6);
                self.set(2 * i + 8, 7, Tile::Empty);
                self.set_life((i * 2) as usize, 2 * i + 8, 13);
                self.set(2 * i + 8, 14, Tile::Empty);
            }
            self.set_man(2, 10);
        }

        if index == 5 {
            self.no_of_required_diamonds = 20;
            self.exit = (15, 10);
            self.set(max_col, 18, Tile::Boulder);
            self.set_square(12, 2, 1, 1, Tile::Boulder);
            self.set_square(14, 2, 1, 1, Tile::Boulder);
            self.set_square(16, 2, 1, 1, Tile::Boulder);
            self.set_square(18, 2, 1, 1, Tile::Boulder);
            self.set_square(12, 7, 7, 7, Tile::Empty);
            self.no_of_life = 4;
            self.set_life(1, 12, 7);
            self.set_life(2, 18, 7);
            self.set_life(3, 12, 13);
            self.set_life(4, 18, 13);
            self.set(self.exit.0, self.exit.1, Tile::Bounds);
            self.set_man(2, 10);
        }

        if index == 6 {
            self.no_of_life = 0;
            self.no_of_required_diamonds = 50;
            self.exit = (max_col, max_row - 3);
            self.set(max_col, max_row - 4, Tile::Boulder);
            self.set_square(5, 4, 10, 5, Tile::Boulder);
            self.morph_no = 1;
            self.set_morph(15, 14, 0);
            self.set_man(self.exit.0 - 1, self.exit.1);
        }

        if index == 7 {
            self.no_of_required_diamonds = 50;
            self.exit = (max_col, max_row - 3);
            self.set(max_col, max_row - 4, Tile::Boulder);
            self.set_square(2, 2, 30, 20, Tile::Empty);
            self.no_of_life = 30;
            for i in 1..=10 {
                self.set_square(2 + i * 2, 4, 1, 1, Tile::Diamond);
                self.set_square(2 + i * 2, 5, 1, 1, Tile::Earth);
                self.set_life((i * 3 - 1) as usize, 2 + i * 2, 6);

                self.set_square(2 + i * 2, 10, 1, 1, Tile::Diamond);
                self.set_square(2 + i * 2, 11, 1, 1, Tile::Earth);
                self.set_life((i * 3 - 2) as usize, 2 + i * 2, 12);

                self.set_square(2 + i * 2, 16, 1, 1, Tile::Diamond);
                self.set_square(2 + i * 2, 17, 1, 1, Tile::Earth);
                self.set_life((i * 3) as usize, 2 + i * 2, 18);
            }
            self.set_man(self.exit.0 - 1, self.exit.1);
        }

        if index == 8 {
            self.no_of_required_diamonds = 50;
            self.exit = (max_col, 10);
            self.set(max_col, 9, Tile::Boulder);
            self.set_square(2, 2, 30, 20, Tile::Empty);
            self.no_of_life = 50;
            for i in 1..=10 {
                for (n, row) in [(4, 6), (3, 9), (2, 12), (1, 15), (0, 18)] {
                    self.set_square(2 + i * 2, row - 2, 1, 1, Tile::Boulder);
                    self.set_square(2 + i * 2, row - 1, 1, 1, Tile::Earth);
                    self.set_life((i * 5 - n) as usize, 2 + i * 2, row);
                }
            }
            self.set_man(self.exit.0 - 1, 10);
        }

        if index == 9 {
            self.no_of_required_diamonds = 2;
            self.exit = (max_col, 10);
            self.no_of_life = 12;
            self.set_square(11, 5, 5, 1, Tile::Empty);
            self.set_life(11, 15, 5);
            self.set_life(12, 21, 5);
            self.set_square(11, 7, 5, 1, Tile::Empty);
            self.set_life(1, 11, 7);
            self.set_life(6, 21, 7);
            self.set_square(11, 9, 5, 1, Tile::Empty);
            self.set_life(2, 12, 9);
            self.set_life(7, 20, 9);
            self.set_square(11, 11, 5, 1, Tile::Empty);
            self.set_life(3, 13, 11);
            self.set_life(8, 19, 11);
            self.set_square(11, 13, 5, 1, Tile::Empty);
            self.set_life(4, 14, 13);
            self.set_life(9, 18, 13);
            self.set_square(11, 15, 5, 1, Tile::Empty);
            self.set_life(5, 15, 15);
            self.set_life(10, 17, 15);
            self.set_square(17, 2, 5, 20, Tile::Empty);
            self.set_wall(16, 2, 18, false);
            self.set_wall(17, 10, 1, false);
            self.set_square(16, 10, 1, 1, Tile::Diamond);
            self.set_square(max_col - 1, 9, 1, 1, Tile::Boulder);
            self.set_man(2, 10);
        }

        self.set_bounds();
    }

    fn next_screen(&mut self) -> Option<(Modal, VecDeque<Op>)> {
        self.screen_no += 1;
        if self.screen_no == LAST_LEVEL + 1 {
            return Some((
                Modal::AllDone,
                VecDeque::from([Op::WrapToLevel1, Op::InitAndDraw]),
            ));
        }
        self.init_and_draw();
        None
    }

    fn init_and_draw(&mut self) {
        self.init_screen(self.screen_no);
        for s in 1..=MAX_COL {
            for t in 1..=MAX_ROW {
                self.place_object(s, t);
            }
        }
        self.display_score(0);
    }

    fn check_if_fng_object_above_man(&self) -> bool {
        matches!(
            self.m(self.man.x, self.man.y - 1),
            Tile::FnDiamond | Tile::FnBoulder
        ) && self.alive
    }

    fn check_if_man_still_happening(&mut self) -> Option<(Modal, VecDeque<Op>)> {
        let (x, y) = (self.man.x, self.man.y);
        if self.alive {
            let t = if self.m(x, y) == Tile::Man1 {
                Tile::Man2
            } else {
                Tile::Man1
            };
            self.set(x, y, t);
            self.place_object(x, y);
        }
        if self.check_if_fng_object_above_man() {
            self.just_died = true;
            self.alive = false;
        }
        if self.just_died {
            self.just_died = false;
            let above = self.m(x, y - 1);
            self.set(x, y, above);
            self.set(x, y - 1, Tile::Empty);
            self.place_object(x, y - 1);
            self.place_object(x, y);
            return Some((
                Modal::Death,
                VecDeque::from([Op::DeathRest, Op::NextScreen]),
            ));
        }
        None
    }

    fn is_rock(&self, col: i32, row: i32) -> bool {
        matches!(
            self.m(col, row),
            Tile::Boulder | Tile::FBoulder | Tile::FnBoulder
        )
    }

    fn rock_on_morph(&mut self, col: i32, row: i32) {
        if row < MAX_ROW - 1 && row > 2 {
            if self.m(col, row) == self.morph && self.is_rock(col, row - 1) {
                self.set(col, row - 1, Tile::Empty);
                self.place_object(col, row - 1);
                self.set(col, row + 1, Tile::FDiamond);
                self.place_object(col, row + 1);
            }
            if self.m(col, row) == self.morph && self.eatible_diamond(col, row - 1) {
                self.set(col, row - 1, Tile::Empty);
                self.place_object(col, row - 1);
                self.set(col, row + 1, Tile::FBoulder);
                self.place_object(col, row + 1);
            }
        }
    }

    fn drop_all(&mut self) {
        for col in 1..=MAX_COL {
            for row in (1..=MAX_ROW - 1).rev() {
                if self.still(col, row) {
                    if self.it_is_empty_below(col, row) {
                        self.swap_vertical(col, row);
                    } else if self.can_move_sideways(col, row) {
                        self.swap_diag(col, row);
                    }
                }
                if self.its_moving(col, row) && self.on_a_diamond_or_rock_or_earth(col, row) {
                    self.change_to_rest(col, row);
                    self.place_object(col, row);
                } else if self.its_moving(col, row) {
                    if self.it_is_empty_below(col, row) {
                        self.swap_vertical(col, row);
                    } else if self.can_move_sideways(col, row) {
                        self.swap_diag(col, row);
                    }
                }
                self.rock_on_morph(col, row);
            }
        }
    }

    /// InvertRect over the exit cell, toggling it every tick.
    fn flash_exit(&mut self) {
        let (ex, ey) = self.exit;
        for y in (ey * 20 - 20)..(ey * 20) {
            for x in (ex * 20 - 20)..(ex * 20) {
                if (0..FB_W as i32).contains(&x) && (0..FB_H as i32).contains(&y) {
                    self.fb[y as usize * FB_W + x as usize] ^= 0xFFFFFF;
                }
            }
        }
    }

    /// Only Morph1..Morph3 are ever chosen (`Random(3)`), and only morph 0 is
    /// animated.  Does nothing on levels without a morph.  (The original never
    /// cleared `Morphic[0]`, so after level 6 a morph kept appearing at
    /// (15,14) on every later level, and before it `Map[0,0]` was written.)
    fn morphchange(&mut self) {
        if self.morph_no == 0 {
            return;
        }
        self.morph = Tile::Morph(self.rng.random(3) as u8);
        let (x, y) = (self.morphic[0].x, self.morphic[0].y);
        self.set(x, y, self.morph);
        self.place_object(x, y);
    }

    fn in_exit(&self) -> bool {
        self.exit_available && self.man.x == self.exit.0 && self.man.y == self.exit.1
    }

    /// The arrow-key part of TRock.Keyboard.
    fn arrow(&mut self, dir: usize, shift: bool) {
        if !self.alive {
            return;
        }
        let mut mp = self.man;
        mp.ox = mp.x;
        mp.oy = mp.y;
        let (dx, dy) = DELTA_STATE[dir];
        mp.x += dx;
        mp.y += dy;
        let (ox, oy) = (mp.ox, mp.oy);

        // Push boulder
        if dx != 0
            && matches!(self.m(mp.x, mp.y), Tile::FBoulder | Tile::Boulder)
            && self.m(dx + mp.x, mp.y) == Tile::Empty
        {
            let t = self.m(ox, mp.y);
            self.set(mp.x, mp.y, t);
            self.set(ox, mp.y, Tile::Empty);
            self.set(dx + mp.x, mp.y, Tile::FBoulder);
            if shift {
                let t = self.m(mp.x, mp.y);
                self.set(ox, mp.y, t);
                self.set(mp.x, mp.y, Tile::Empty);
                mp.x = ox;
            }
            self.place_object(ox, mp.y);
            self.place_object(ox + dx, oy + dy);
            self.place_object(ox + 2 * dx, oy + dy);
            mp.ox = mp.x;
        }
        let (ox, oy) = (mp.ox, mp.oy);

        // Dig or dig and move
        if self.space_clear(mp.x, mp.y) {
            if self.eatible_diamond(mp.x, mp.y) {
                self.display_score(1);
            }
            let t = self.m(ox, oy);
            self.set(mp.x, mp.y, t);
            self.set(ox, oy, Tile::Empty);
            if shift {
                let t = self.m(mp.x, mp.y);
                self.set(ox, oy, t);
                self.set(mp.x, mp.y, Tile::Empty);
                mp.x = ox;
                mp.y = oy;
            }
            self.place_object(ox, oy);
            self.place_object(ox + dx, oy + dy);
        } else {
            mp.x = ox;
            mp.y = oy;
        }
        self.man = mp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game::new(Sprites::blank())
    }

    fn count(g: &Game, t: Tile) -> usize {
        (1..=MAX_COL)
            .flat_map(|c| (1..=MAX_ROW).map(move |r| (c, r)))
            .filter(|&(c, r)| g.m(c, r) == t)
            .count()
    }

    #[test]
    fn levels_are_deterministic() {
        let mut a = game();
        let mut b = game();
        for lvl in 1..=9 {
            a.init_screen(lvl);
            b.init_screen(lvl);
            assert_eq!(a.map, b.map, "level {lvl}");
        }
    }

    #[test]
    fn level1_layout() {
        let g = game();
        assert_eq!(g.m(2, 2), Tile::Man1);
        assert_eq!(g.exit, (31, 19));
        assert_eq!(g.no_of_required_diamonds, 67);
        // Walls at cols 8/15/23 with an earth gap at rows 9..12
        assert_eq!(g.m(8, 5), Tile::Wall);
        assert_eq!(g.m(8, 10), Tile::Earth);
        assert!(count(&g, Tile::Diamond) > 67);
        // Border is all Bounds
        for c in 1..=MAX_COL {
            assert_eq!(g.m(c, 1), Tile::Bounds);
            assert_eq!(g.m(c, MAX_ROW), Tile::Bounds);
        }
    }

    #[test]
    fn boulder_falls_and_kills() {
        let mut g = game();
        g.init_screen(1);
        // Clear a column above the man and drop a boulder on him.
        let (x, y) = (g.man.x, g.man.y);
        for col in 1..=MAX_COL {
            for row in 2..MAX_ROW {
                g.set(col, row, Tile::Earth);
            }
        }
        g.set_man(x, y + 4);
        g.set(x, y + 1, Tile::Boulder);
        g.set(x, y + 2, Tile::Empty);
        g.set(x, y + 3, Tile::Empty);
        g.timer(); // boulder starts falling into the gap
        assert_eq!(g.m(x, y + 2), Tile::FnBoulder);
        assert!(g.modal().is_none());
        g.timer(); // falling boulder directly above the man -> death box
        assert_eq!(g.modal(), Some(&Modal::Death));
        assert!(!g.alive);
        g.close_modal(None);
        assert!(g.alive);
        assert_eq!(g.screen_no, 1);
        assert_eq!(g.m(2, 2), Tile::Man1);
    }

    #[test]
    fn eat_diamond_scores() {
        let mut g = game();
        let (x, y) = (g.man.x, g.man.y);
        g.set(x + 1, y, Tile::Diamond);
        g.key_down(Some(3), false);
        assert_eq!(g.diamonds, 1);
        assert_eq!(g.man.x, x + 1);
        assert!(matches!(g.m(x + 1, y), Tile::Man1 | Tile::Man2));
        assert_eq!(g.m(x, y), Tile::Empty);
    }

    #[test]
    fn shift_digs_without_moving() {
        let mut g = game();
        let (x, y) = (g.man.x, g.man.y);
        g.set(x + 1, y, Tile::Earth);
        g.key_down(Some(3), true);
        assert_eq!((g.man.x, g.man.y), (x, y));
        assert_eq!(g.m(x + 1, y), Tile::Empty);
    }

    #[test]
    fn push_boulder() {
        let mut g = game();
        let (x, y) = (g.man.x, g.man.y);
        g.set(x + 1, y, Tile::Boulder);
        g.set(x + 2, y, Tile::Empty);
        g.key_down(Some(3), false);
        assert_eq!(g.man.x, x + 1);
        assert_eq!(g.m(x + 2, y), Tile::FBoulder);
    }

    #[test]
    fn level_dialog() {
        let mut g = game();
        g.menu_level();
        assert_eq!(g.modal(), Some(&Modal::LevelInput));
        assert_eq!(g.level_text, "1");
        g.close_modal(Some("4".into()));
        assert_eq!(g.screen_no, 4);
        assert_eq!(g.no_of_life, 14);
        // Opens on the current level; Cancel and bad input change nothing.
        g.key_down(Some(3), false);
        assert_eq!(g.man.x, 3);
        g.menu_level();
        assert_eq!(g.level_text, "4");
        g.close_modal(None);
        assert!(g.modal().is_none());
        assert_eq!(g.screen_no, 4);
        assert_eq!(g.man.x, 3, "level was not restarted");
        for bad in ["", "0", ":", "10", "x"] {
            g.menu_level();
            g.close_modal(Some(bad.into()));
            assert!(g.modal().is_none());
            assert_eq!(g.screen_no, 4, "input {bad:?}");
        }
        g.menu_level();
        g.close_modal(Some("9".into()));
        assert_eq!(g.screen_no, 9);
    }

    #[test]
    fn morph_only_on_level_6() {
        let mut g = game();
        g.menu_level();
        g.close_modal(Some("6".into()));
        g.timer();
        assert!(matches!(g.m(15, 14), Tile::Morph(_)));
        g.menu_level();
        g.close_modal(Some("7".into()));
        for _ in 0..5 {
            g.timer();
        }
        assert!(!matches!(g.m(15, 14), Tile::Morph(_)));
        assert_eq!(g.m(0, 0), Tile::Outside);
    }

    #[test]
    fn rolls_both_ways() {
        // A boulder balanced on another with space on both sides should
        // sometimes roll left and sometimes right.
        let (mut left, mut right) = (0, 0);
        let mut g = game();
        for seed in 0..200 {
            for col in 2..MAX_COL {
                for row in 2..MAX_ROW {
                    g.set(col, row, Tile::Empty);
                }
            }
            g.set(10, 19, Tile::Wall);
            g.set(10, 18, Tile::Boulder);
            g.rng.seed = seed * 7919;
            g.drop_all();
            // (It may have fallen further within the same sweep.)
            if (17..=19).any(|r| g.m(9, r) == Tile::FnBoulder) {
                left += 1;
            }
            if (17..=19).any(|r| g.m(11, r) == Tile::FnBoulder) {
                right += 1;
            }
        }
        assert!(left > 20 && right > 20, "left {left} right {right}");
    }

    #[test]
    fn other_keys_do_nothing() {
        let mut g = game();
        let before = g.m(2, 2);
        g.key_down(None, false);
        assert_eq!(g.m(2, 2), before);
    }

    #[test]
    fn reaching_exit_advances() {
        let mut g = game();
        g.diamonds = g.no_of_required_diamonds - 1;
        g.display_score(1);
        assert!(g.exit_available);
        assert_eq!(g.m(31, 19), Tile::Door);
        g.set(g.man.x, g.man.y, Tile::Empty);
        g.set_man(30, 19);
        g.key_down(Some(3), false);
        assert_eq!(g.screen_no, 2);
        assert_eq!(g.no_of_life, 3);
    }

    #[test]
    fn paused_under_modal() {
        let mut g = game();
        g.menu_about();
        let before = (g.rng.seed, g.map);
        g.timer();
        assert_eq!((g.rng.seed, g.map), before);
        assert_eq!(g.modal(), Some(&Modal::About));
    }

    /// `ROCK_LEVEL_PNGS=dir cargo test level_pngs -- --ignored` writes the
    /// opening screen of every level, drawn with the real bitmaps.
    #[test]
    #[ignore]
    fn level_pngs() {
        let dir = std::env::var("ROCK_LEVEL_PNGS").unwrap_or_else(|_| ".".into());
        let mut g = Game::new(crate::assets::sprites());
        for lvl in 1..=9 {
            g.screen_no = lvl - 1;
            g.run(VecDeque::from([Op::NextScreen]));
            // export_png expects bottom-up rows (it is meant for GL screen
            // grabs), so hand it the framebuffer upside down.
            let bytes =
                g.fb.chunks(FB_W)
                    .rev()
                    .flatten()
                    .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8, 255])
                    .collect();
            let img = macroquad::prelude::Image {
                bytes,
                width: FB_W as u16,
                height: FB_H as u16,
            };
            img.export_png(&format!("{dir}/level{lvl}.png"));
        }
    }

    #[test]
    fn soak_all_levels() {
        // Run every level for a while with random-ish input; must not panic.
        let mut g = game();
        let mut r = TpRandom::new(1234);
        for lvl in 1..=9 {
            g.screen_no = lvl - 1;
            g.run(VecDeque::from([Op::NextScreen]));
            for _ in 0..2000 {
                if g.modal().is_some() {
                    g.close_modal(None);
                }
                if r.random(3) == 0 {
                    g.timer();
                } else {
                    let d = r.random(4) as usize + 1;
                    g.key_down(Some(d), r.random(5) == 0);
                }
            }
        }
    }
}
