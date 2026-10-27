//! ROCK -- Dale O'Brien, Beta Version of Rock, January 1993.
//! Originally written in Borland Turbo Pascal for Windows 1.5 for Windows 3.x;
//! ported to Rust with macroquad.
//!
//! This file is the window: the menu bar, the 200 ms timer, keyboard
//! auto-repeat, the message boxes and scaling.  The game itself is in
//! `game/`.

mod assets;
mod game;
mod rng;
mod save;
mod script;
mod ui;

use game::{Dir, Game, FB_H, FB_W, LAST_LEVEL};
use macroquad::prelude::*;
use script::Action;
use ui::{Closed, Dialog, MenuBar, MenuCmd, MENU_H};

/// The virtual screen: menu bar above the 620 x 400 board.
const VIEW: Vec2 = Vec2::new(FB_W as f32, FB_H as f32 + MENU_H);

/// The original's `SetTimer(HWindow, 0, 200, nil)`.
const TICK: f32 = 0.2;
/// Windows' default keyboard repeat: about 500 ms delay, then 30 a second.
const REPEAT_DELAY: f32 = 0.5;
const REPEAT_RATE: f32 = 1.0 / 30.0;
/// Windows 3.1 desktop teal, around the scaled view.
const DESKTOP: Color = Color::new(0.0, 0.5, 0.5, 1.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Rock".to_owned(),
        window_width: (VIEW.x * 2.0) as i32,
        window_height: (VIEW.y * 2.0) as i32,
        window_resizable: true,
        icon: Some(assets::icon()),
        ..Default::default()
    }
}

fn arrow_dir(k: KeyCode) -> Option<Dir> {
    match k {
        KeyCode::Left => Some(Dir::Left),
        KeyCode::Up => Some(Dir::Up),
        KeyCode::Right => Some(Dir::Right),
        KeyCode::Down => Some(Dir::Down),
        _ => None,
    }
}

fn shift_down() -> bool {
    is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)
}

fn alt_down() -> bool {
    is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt)
}

/// Where the view sits in the window: whole-number scaling when it fits,
/// centred.
struct Layout {
    scale: f32,
    origin: Vec2,
}

impl Layout {
    fn current() -> Self {
        let window = vec2(screen_width(), screen_height());
        let fit = (window / VIEW).min_element();
        let scale = if fit >= 1.0 { fit.floor() } else { fit };
        Layout {
            scale,
            origin: ((window - VIEW * scale) / 2.0).floor(),
        }
    }

    fn to_view(&self, window_pos: Vec2) -> Vec2 {
        (window_pos - self.origin) / self.scale
    }
}

/// The arrow key being held, and the time until it next repeats.
struct Held {
    key: KeyCode,
    dir: Dir,
    wait: f32,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new(assets::sprites());
    // Carry on where the last run left off.  Scripted runs always start
    // at level 1 and don't touch the save file.
    let mut script = script::Script::from_env();
    let saving = !script.active();
    if saving {
        if let Some(n) = save::load() {
            game.go_to_level(n);
        }
    }
    let mut saved_level = game.level();
    game.paint();

    let board_tex = Texture2D::from_rgba8(FB_W as u16, FB_H as u16, game.pixels());
    board_tex.set_filter(FilterMode::Nearest);

    let target = render_target(VIEW.x as u32, VIEW.y as u32);
    target.texture.set_filter(FilterMode::Nearest);
    let mut view_cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VIEW.x, VIEW.y));
    view_cam.render_target = Some(target.clone());

    let mut menu = MenuBar::new();
    let mut dialog: Option<Dialog> = None;
    let mut since_tick = 0.0;
    let mut held: Option<Held> = None;

    loop {
        let dt = get_frame_time().min(0.25);
        let layout = Layout::current();
        let mouse = layout.to_view(mouse_position().into());

        // Scripted input, for automated testing; empty unless ROCK_SCRIPT
        // is set.
        let scripted = script.next_frame();

        // Show the game's open box, if it has one.
        dialog = match (game.modal(), dialog.take()) {
            (Some(m), Some(d)) if d.modal == m => Some(d),
            (Some(m), _) => Some(Dialog::new(m, &game.level().to_string(), VIEW)),
            (None, _) => None,
        };

        // ---- input --------------------------------------------------------
        let mut pressed: Vec<KeyCode> = get_keys_pressed().into_iter().collect();
        let mut chars: Vec<char> = std::iter::from_fn(get_char_pressed).collect();
        let mut command = None;
        for action in &scripted {
            match action {
                Action::Key(k) => pressed.push(*k),
                Action::Char(c) => chars.push(*c),
                Action::About => command = Some(MenuCmd::About),
                Action::Level => command = Some(MenuCmd::Level),
                Action::Shot(_) | Action::Quit => {}
            }
        }

        if let Some(d) = &mut dialog {
            held = None;
            if let Some(closed) = d.handle_input(&pressed, &chars, mouse) {
                let text = (closed == Closed::Ok).then(|| d.text().to_owned());
                game.close_modal(text.as_deref());
                dialog = None;
            }
        } else {
            if alt_down() {
                if pressed.contains(&KeyCode::A) {
                    command = Some(MenuCmd::About);
                } else if pressed.contains(&KeyCode::L) {
                    command = Some(MenuCmd::Level);
                }
            }
            command = command.or_else(|| menu.handle_mouse(mouse));
            match command {
                Some(MenuCmd::About) => game.menu_about(),
                Some(MenuCmd::Level) => game.menu_level(),
                None if !alt_down() => {
                    // Only the arrow keys do anything, so other keys don't
                    // interrupt a held arrow's repeat.
                    for key in pressed {
                        if let Some(dir) = arrow_dir(key) {
                            game.key_down(dir, shift_down());
                            held = Some(Held {
                                key,
                                dir,
                                wait: REPEAT_DELAY,
                            });
                        }
                    }
                    held = held.filter(|h| is_key_down(h.key));
                    if let Some(h) = &mut held {
                        h.wait -= dt;
                        while h.wait <= 0.0 {
                            h.wait += REPEAT_RATE;
                            game.key_down(h.dir, shift_down());
                        }
                    }
                }
                None => {}
            }
        }

        // ---- the timer (the game ignores it while a box is open) ---------
        since_tick += dt;
        if since_tick >= TICK {
            // Like WM_TIMER, ticks missed while the program was busy are
            // not made up.
            since_tick = if since_tick >= 2.0 * TICK {
                0.0
            } else {
                since_tick - TICK
            };
            game.timer();
        }

        // Save the level whenever a new one starts.  (While the "All done"
        // box is open it is one past the last, which isn't saved.)
        let level = game.level();
        if saving && level != saved_level && (1..=LAST_LEVEL).contains(&level) {
            save::store(level);
            saved_level = level;
        }

        // ---- draw ----------------------------------------------------------
        if game.take_redraw() {
            board_tex.update_from_bytes(FB_W as u32, FB_H as u32, game.pixels());
        }

        set_camera(&view_cam);
        clear_background(WHITE);
        draw_texture(&board_tex, 0.0, MENU_H, WHITE);
        menu.draw(dialog.is_none(), mouse);
        if let Some(d) = &dialog {
            d.draw(mouse);
        }

        set_default_camera();
        clear_background(DESKTOP);
        draw_texture_ex(
            &target.texture,
            layout.origin.x,
            layout.origin.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(VIEW * layout.scale),
                // Render targets come out upside down in macroquad.
                flip_y: true,
                ..Default::default()
            },
        );

        for action in scripted {
            match action {
                Action::Shot(path) => get_screen_data().export_png(&path),
                Action::Quit => return,
                _ => {}
            }
        }

        next_frame().await;
    }
}
