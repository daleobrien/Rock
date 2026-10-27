//! The game: rules, levels and drawing, independent of the window.
//!
//! This started as a line-by-line port of ROCK_FUC.PAS, ROCK_VAR.PAS and the
//! message handlers in ROCK.PAS.  The behaviour (including Turbo Pascal's
//! random numbers, and so the level layouts) is unchanged, apart from the
//! original's bugs that are fixed and listed in the README.
//!
//! The front end drives it like Windows drove the original: [`Game::timer`]
//! every 200 ms, [`Game::key_down`] for the arrow keys, the two menu commands,
//! and [`Game::close_modal`] when the player dismisses a box.

mod board;
mod creatures;
mod levels;
mod physics;
mod player;
mod render;
#[cfg(test)]
mod tests;
mod tile;

pub use board::Dir;
pub use render::{Bitmap, Sprites, CELL, FB_H, FB_W};

use crate::rng::TpRandom;
use board::{Board, Pos};
use creatures::Creature;
use render::Framebuffer;
use tile::Tile;

pub const LAST_LEVEL: i32 = 9;

/// Where the score bar sits in the top row, and how wide it is.
const SCORE_X: i32 = 103;
const SCORE_W: i32 = 197;

/// A message box or dialog the game wants shown.  While one is open the game
/// is paused, and whatever the game was doing when it opened carries on once
/// it is closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modal {
    /// You were killed.
    Death,
    /// You finished level 9.
    AllDone,
    /// The About box.
    About,
    /// The Level dialog.
    LevelInput,
}

impl Modal {
    pub fn title(self) -> &'static str {
        match self {
            Modal::Death => "Death says ..",
            Modal::AllDone => "You have escaped Death",
            Modal::About => "Beta Verison 1.0 - Rock",
            Modal::LevelInput => "You Choose",
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Modal::Death => "You are mine !",
            Modal::AllDone => "All done",
            Modal::About => "1993, Dale ",
            Modal::LevelInput => "Enter 1..9",
        }
    }
}

/// One step of a handler.  Handlers run as a list of steps so that, when a
/// step opens a box, the rest can carry on after the box is closed (the
/// original's `MessageBox` simply blocked in the middle of the handler).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    // The 200 ms timer
    DropAll,
    CheckMan,
    FlashExit,
    MoveCreatures,
    AnimateMorph,
    // An arrow key
    Move { dir: Dir, shift: bool },
    EnterExit,
    // After the death box: restart the same level
    AfterDeath,
    // Go to `level + 1`, via the "All done" box after the last level
    NextLevel,
    WrapToFirstLevel,
    StartLevel,
}

const TIMER_STEPS: [Step; 5] = [
    Step::DropAll,
    Step::CheckMan,
    Step::FlashExit,
    Step::MoveCreatures,
    Step::AnimateMorph,
];

/// What a step returns when it opens a box: the box, and the steps that must
/// run when it closes (before the rest of the interrupted handler).
type Interrupt = Option<(Modal, Vec<Step>)>;

pub struct Game {
    board: Board,
    fb: Framebuffer,
    sprites: Sprites,
    rng: TpRandom,

    /// The level being played (`Screen_no`).
    level: i32,
    man: Pos,
    alive: bool,
    /// Set when something kills the man; the death box opens at the next
    /// check.
    just_died: bool,
    diamonds: i32,
    diamonds_needed: i32,
    exit: Pos,
    /// The door has appeared at the exit.
    exit_open: bool,

    /// The original kept 100 creature records and reused them from level to
    /// level; `creature_count` of them are live on this level.
    creatures: [Creature; creatures::MAX],
    creature_count: usize,

    /// Where the morph is, on the level that has one.
    morph: Option<Pos>,
    /// The morph's current animation frame.  Cells are compared against it
    /// to find the morph, as the original compared bitmap handles.
    morph_tile: Tile,

    /// The open box, and the steps to run when it closes.
    pending: Option<(Modal, Vec<Step>)>,
}

impl Game {
    pub fn new(sprites: Sprites) -> Self {
        let mut g = Game {
            board: Board::new(),
            fb: Framebuffer::new(),
            sprites,
            rng: TpRandom::new(0),
            level: 1,
            man: Pos::default(),
            alive: false,
            just_died: false,
            diamonds: 0,
            diamonds_needed: 1,
            exit: Pos::default(),
            exit_open: false,
            creatures: [Creature::default(); creatures::MAX],
            creature_count: 0,
            morph: None,
            morph_tile: Tile::Morph(0),
            pending: None,
        };
        g.init_level(1);
        g
    }

    // ----- the front end's interface -------------------------------------

    /// The level being played.
    pub fn level(&self) -> i32 {
        self.level
    }

    /// The client-area picture, RGBA, `FB_W` x `FB_H`.
    pub fn pixels(&self) -> &[u8] {
        self.fb.rgba()
    }

    /// True if the picture changed since the last call.
    pub fn take_redraw(&mut self) -> bool {
        self.fb.take_dirty()
    }

    /// Redraw the whole client area (`WM_PAINT`).
    pub fn paint(&mut self) {
        for p in Board::positions() {
            self.draw(p);
        }
        self.display_score(0);
    }

    /// The 200 ms timer.  Does nothing while a box is open.
    pub fn timer(&mut self) {
        if self.pending.is_none() {
            self.run(TIMER_STEPS);
        }
    }

    /// An arrow key (including auto-repeats).  Shift digs or pushes without
    /// moving.
    pub fn key_down(&mut self, dir: Dir, shift: bool) {
        if self.pending.is_none() {
            self.run([Step::Move { dir, shift }, Step::CheckMan, Step::EnterExit]);
        }
    }

    /// Menu: About.
    pub fn menu_about(&mut self) {
        self.open(Modal::About);
    }

    /// Menu: Level.
    pub fn menu_level(&mut self) {
        self.open(Modal::LevelInput);
    }

    fn open(&mut self, modal: Modal) {
        if self.pending.is_none() {
            self.pending = Some((modal, Vec::new()));
        }
    }

    /// The open box, if any.
    pub fn modal(&self) -> Option<Modal> {
        self.pending.as_ref().map(|(m, _)| *m)
    }

    /// Close the open box.  For the Level dialog, `level_text` is what was
    /// entered when OK was pressed, or `None` for Cancel; anything but a
    /// level from 1 to 9 leaves the game as it was.
    pub fn close_modal(&mut self, level_text: Option<&str>) {
        let Some((modal, resume)) = self.pending.take() else {
            return;
        };
        self.run(resume);
        if modal == Modal::LevelInput {
            let chosen = level_text.and_then(|t| t.trim().parse::<i32>().ok());
            if let Some(n) = chosen {
                self.go_to_level(n);
            }
        }
        // Windows then repainted whatever the box had covered.
        self.paint();
    }

    /// Start level `n`, as the Level dialog does.  Anything but 1 to 9 is
    /// ignored.
    pub fn go_to_level(&mut self, n: i32) {
        if (1..=LAST_LEVEL).contains(&n) {
            self.level = n - 1;
            self.run([Step::NextLevel]);
        }
    }

    // ----- running handlers ----------------------------------------------

    fn run(&mut self, steps: impl IntoIterator<Item = Step>) {
        let mut steps = steps.into_iter();
        while let Some(step) = steps.next() {
            if let Some((modal, mut resume)) = self.step(step) {
                resume.extend(steps);
                self.pending = Some((modal, resume));
                return;
            }
        }
    }

    fn step(&mut self, step: Step) -> Interrupt {
        match step {
            Step::DropAll => self.drop_all(),
            Step::CheckMan => return self.check_man(),
            Step::FlashExit => self.flash_exit(),
            Step::MoveCreatures => self.move_creatures(),
            Step::AnimateMorph => self.animate_morph(),
            Step::Move { dir, shift } => self.move_man(dir, shift),
            Step::EnterExit => {
                if self.exit_open && self.man == self.exit {
                    return self.next_level();
                }
            }
            Step::AfterDeath => {
                // Restart the same level: NextLevel adds the 1 back.
                self.level = (self.level - 1).max(0);
                self.alive = true;
            }
            Step::NextLevel => return self.next_level(),
            Step::WrapToFirstLevel => self.level = 1,
            Step::StartLevel => self.start_level(),
        }
        None
    }

    fn next_level(&mut self) -> Interrupt {
        self.level += 1;
        if self.level == LAST_LEVEL + 1 {
            return Some((
                Modal::AllDone,
                vec![Step::WrapToFirstLevel, Step::StartLevel],
            ));
        }
        self.start_level();
        None
    }

    fn start_level(&mut self) {
        self.init_level(self.level);
        self.paint();
    }

    // ----- drawing --------------------------------------------------------

    /// Draw one cell from the board (`PlaceObject`).
    fn draw(&mut self, p: Pos) {
        if let Some(i) = self.board[p].sprite() {
            let (x, y) = ((p.col - 1) * CELL as i32, (p.row - 1) * CELL as i32);
            self.fb.blit(x, y, &self.sprites.tiles[i]);
        }
    }

    /// Change a cell and draw it.
    fn put(&mut self, p: Pos, t: Tile) {
        self.board.set(p, t);
        self.draw(p);
    }

    /// Add to the diamond count and redraw the score bar.  Collecting the
    /// last diamond needed opens the door.
    fn display_score(&mut self, add: i32) {
        self.diamonds = (self.diamonds + add).min(self.diamonds_needed);
        if self.diamonds == self.diamonds_needed {
            self.put(self.exit, Tile::Door);
            self.exit_open = true;
        }
        let filled = (f64::from(self.diamonds) * f64::from(SCORE_W - 1)
            / f64::from(self.diamonds_needed))
        .round() as i32;
        let s = &self.sprites;
        self.fb
            .stretch_columns(SCORE_X + filled, SCORE_W - filled, &s.score_empty);
        self.fb.stretch_columns(SCORE_X, filled, &s.score_filled);
    }

    /// Invert the exit cell, so it flashes on every timer tick.
    fn flash_exit(&mut self) {
        let c = CELL as i32;
        self.fb
            .invert((self.exit.col - 1) * c, (self.exit.row - 1) * c, c, c);
    }

    /// Show a random one of the first three morph frames (the fourth bitmap
    /// is never used).
    fn animate_morph(&mut self) {
        if let Some(p) = self.morph {
            self.morph_tile = Tile::Morph(self.rng.random(3) as u8);
            self.put(p, self.morph_tile);
        }
    }
}
