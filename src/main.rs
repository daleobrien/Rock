//! ROCK -- Dale O'Brien, Beta Version of Rock, January 1993.
//! Originally written in Borland Turbo Pascal for Windows 1.5 for Windows 3.x;
//! ported to Rust with macroquad.
//!
//! main.rs is the "TRock window": it owns the window, the menu bar, the
//! 200ms timer, keyboard auto-repeat and the Windows-3.1-style message boxes.
//! All game rules live in game.rs.

mod assets;
mod game;
mod rng;
mod script;
mod ui;

use game::{Game, FB_H, FB_W};
use macroquad::prelude::*;
use ui::{Dialog, MenuBar, MENU_H};

/// The virtual screen: menu bar + 620x400 client area.
const VIEW_W: f32 = FB_W as f32;
const VIEW_H: f32 = FB_H as f32 + MENU_H;

/// SetTimer(HWindow, 0, 200, nil); Game::timer skips ticks while a box is open.
const TICK: f32 = 0.2;
/// Windows' default keyboard repeat: ~500ms delay, ~30 repeats per second.
const REPEAT_DELAY: f32 = 0.5;
const REPEAT_RATE: f32 = 1.0 / 30.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Rock".to_owned(),
        window_width: (VIEW_W * 2.0) as i32,
        window_height: (VIEW_H * 2.0) as i32,
        window_resizable: true,
        icon: Some(assets::icon()),
        ..Default::default()
    }
}

/// What a key press means to TRock.Keyboard: Some(1..=4) for the arrow keys
/// (VK_LEFT..VK_DOWN minus 36), None for anything else.
fn arrow_of(k: KeyCode) -> Option<usize> {
    match k {
        KeyCode::Left => Some(1),
        KeyCode::Up => Some(2),
        KeyCode::Right => Some(3),
        KeyCode::Down => Some(4),
        _ => None,
    }
}

fn shift_down() -> bool {
    is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)
}

fn alt_down() -> bool {
    is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt)
}

/// Keys that Windows sends as WM_SYSKEYDOWN, which TRock.Keyboard never saw.
fn is_sys_key(k: KeyCode) -> bool {
    matches!(k, KeyCode::LeftAlt | KeyCode::RightAlt | KeyCode::F10) || alt_down()
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new(assets::sprites());
    game.paint();

    let mut fb_image = Image::gen_image_color(FB_W as u16, FB_H as u16, WHITE);
    let fb_tex = Texture2D::from_image(&fb_image);
    fb_tex.set_filter(FilterMode::Nearest);

    let target = render_target(VIEW_W as u32, VIEW_H as u32);
    target.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VIEW_W, VIEW_H));
    cam.render_target = Some(target.clone());

    let mut menu = MenuBar::new();
    let mut dialog: Option<Dialog> = None;
    let mut tick_acc = 0.0f32;
    let mut repeat: Option<(KeyCode, f32)> = None;
    let mut script = script::Script::from_env();

    loop {
        let dt = get_frame_time().min(0.25);

        // Where the virtual screen sits in the real window.
        let sw = screen_width();
        let sh = screen_height();
        let mut scale = (sw / VIEW_W).min(sh / VIEW_H);
        if scale >= 1.0 {
            scale = scale.floor();
        }
        let ox = ((sw - VIEW_W * scale) / 2.0).floor();
        let oy = ((sh - VIEW_H * scale) / 2.0).floor();
        let (mx, my) = mouse_position();
        let mouse = vec2((mx - ox) / scale, (my - oy) / scale);
        ui::set_mouse(mouse);

        // Scripted input (test automation only; does nothing unless
        // ROCK_SCRIPT is set).
        let scripted = script.next_frame();

        // Keep the on-screen dialog in step with the game's modal stack.
        match (game.modal().cloned(), &dialog) {
            (Some(m), Some(d)) if d.modal == m => {}
            (Some(m), _) => dialog = Some(Dialog::new(m, &game.level_text)),
            (None, _) => dialog = None,
        }

        // ---- input ----------------------------------------------------
        let mut pressed: Vec<KeyCode> = get_keys_pressed().into_iter().collect();
        let mut chars: Vec<char> = std::iter::from_fn(get_char_pressed).collect();
        let mut menu_cmd = None;
        for s in &scripted {
            match s {
                script::Action::Key(k) => pressed.push(*k),
                script::Action::Char(c) => chars.push(*c),
                script::Action::About => menu_cmd = Some(ui::MenuCmd::About),
                script::Action::Level => menu_cmd = Some(ui::MenuCmd::Level),
                _ => {}
            }
        }

        if let Some(d) = dialog.as_mut() {
            repeat = None;
            if let Some(result) = d.handle_input(&pressed, &chars, mouse) {
                game.close_modal(result);
                dialog = None;
            }
        } else {
            if alt_down() {
                if pressed.contains(&KeyCode::A) {
                    menu_cmd = Some(ui::MenuCmd::About);
                } else if pressed.contains(&KeyCode::L) {
                    menu_cmd = Some(ui::MenuCmd::Level);
                }
            }
            if menu_cmd.is_none() {
                menu_cmd = menu.handle_mouse(mouse);
            }
            match menu_cmd {
                Some(ui::MenuCmd::About) => game.menu_about(),
                Some(ui::MenuCmd::Level) => game.menu_level(),
                None => {
                    // Only the arrow keys do anything, so Shift (or any other
                    // key) doesn't interrupt a held arrow's auto-repeat.
                    for k in pressed {
                        if is_sys_key(k) || arrow_of(k).is_none() {
                            continue;
                        }
                        game.key_down(arrow_of(k), shift_down());
                        repeat = Some((k, REPEAT_DELAY));
                    }
                    // Auto-repeat of the last key pressed, like WM_KEYDOWN.
                    if let Some((k, t)) = repeat.as_mut() {
                        if !is_key_down(*k) {
                            repeat = None;
                        } else {
                            *t -= dt;
                            while *t <= 0.0 {
                                *t += REPEAT_RATE;
                                game.key_down(arrow_of(*k), shift_down());
                            }
                        }
                    }
                }
            }
        }

        // ---- WM_TIMER (runs even while a box is open) ------------------
        tick_acc += dt;
        if tick_acc >= TICK {
            tick_acc -= TICK;
            if tick_acc >= TICK {
                tick_acc = 0.0; // Windows coalesces missed WM_TIMERs
            }
            game.timer();
        }

        // ---- draw -------------------------------------------------------
        for (i, px) in game.fb.iter().enumerate() {
            let p = &mut fb_image.bytes[i * 4..i * 4 + 4];
            p[0] = (px >> 16) as u8;
            p[1] = (px >> 8) as u8;
            p[2] = *px as u8;
            p[3] = 255;
        }
        fb_tex.update(&fb_image);

        set_camera(&cam);
        clear_background(WHITE);
        draw_texture(&fb_tex, 0.0, MENU_H, WHITE);
        menu.draw(dialog.is_none());
        if let Some(d) = &dialog {
            d.draw(VIEW_W, VIEW_H);
        }

        set_default_camera();
        clear_background(Color::from_rgba(0, 128, 128, 255)); // Win 3.1 desktop teal
        draw_texture_ex(
            &target.texture,
            ox,
            oy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VIEW_W * scale, VIEW_H * scale)),
                // Render targets come out upside down in macroquad.
                flip_y: true,
                ..Default::default()
            },
        );

        for s in &scripted {
            match s {
                script::Action::Shot(path) => {
                    get_screen_data().export_png(path);
                }
                script::Action::Quit => return,
                _ => {}
            }
        }

        next_frame().await;
    }
}
