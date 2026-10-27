//! Scripted input for automated testing and screenshots.  Inactive unless the
//! ROCK_SCRIPT environment variable is set, e.g.
//!
//! ROCK_SCRIPT="wait:10 shot:a.png key:Right wait:5 level char:3 key:Enter wait:2 shot:b.png quit"

use macroquad::prelude::KeyCode;
use std::collections::VecDeque;

pub enum Action {
    Key(KeyCode),
    Char(char),
    About,
    Level,
    Shot(String),
    Quit,
}

pub struct Script {
    steps: VecDeque<String>,
    wait: u32,
}

impl Script {
    pub fn from_env() -> Self {
        let steps = std::env::var("ROCK_SCRIPT")
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        Script { steps, wait: 0 }
    }

    /// True if ROCK_SCRIPT gave any steps.
    pub fn active(&self) -> bool {
        !self.steps.is_empty()
    }

    /// Actions for this frame: everything up to the next `wait:N`.
    pub fn next_frame(&mut self) -> Vec<Action> {
        let mut out = Vec::new();
        if self.wait > 0 {
            self.wait -= 1;
            return out;
        }
        while let Some(s) = self.steps.pop_front() {
            let (cmd, arg) = s.split_once(':').unwrap_or((s.as_str(), ""));
            match cmd {
                "wait" => {
                    self.wait = arg.parse().unwrap_or(1);
                    break;
                }
                "key" => out.push(Action::Key(match arg {
                    "Left" => KeyCode::Left,
                    "Right" => KeyCode::Right,
                    "Up" => KeyCode::Up,
                    "Down" => KeyCode::Down,
                    "Enter" => KeyCode::Enter,
                    "Escape" => KeyCode::Escape,
                    "Backspace" => KeyCode::Backspace,
                    _ => KeyCode::Space,
                })),
                "char" => out.extend(arg.chars().map(Action::Char)),
                "about" => out.push(Action::About),
                "level" => out.push(Action::Level),
                "shot" => out.push(Action::Shot(arg.to_owned())),
                "quit" => out.push(Action::Quit),
                _ => {}
            }
        }
        out
    }
}
