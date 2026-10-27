//! Stand-ins for the Windows 3.1 chrome the original got for free: the menu
//! bar from the MENU_1 resource, MessageBox, and OWL's TInputDialog.
//!
//! Everything is drawn in virtual-screen coordinates (the 620-pixel-wide
//! window), and takes the mouse position in the same coordinates.

use crate::game::{Modal, FB_W};
use macroquad::prelude::*;

pub const MENU_H: f32 = 20.0;

const NAVY: Color = Color::new(0.0, 0.0, 0.5, 1.0);
const GRAY: Color = Color::new(0.75, 0.75, 0.75, 1.0);
const DARK: Color = Color::new(0.5, 0.5, 0.5, 1.0);
const FONT: u16 = 16;

fn text(s: &str, x: f32, y: f32, color: Color) {
    draw_text(s, x.floor(), y.floor(), f32::from(FONT), color);
}

fn text_w(s: &str) -> f32 {
    measure_text(s, None, FONT, 1.0).width
}

/// A 1px rectangle outline drawn with filled rects, so it stays crisp when
/// the virtual screen is scaled up.
fn outline(r: Rect, color: Color) {
    draw_rectangle(r.x, r.y, r.w, 1.0, color);
    draw_rectangle(r.x, r.y + r.h - 1.0, r.w, 1.0, color);
    draw_rectangle(r.x, r.y, 1.0, r.h, color);
    draw_rectangle(r.x + r.w - 1.0, r.y, 1.0, r.h, color);
}

/// Text with the first letter underlined, for "&About" style labels.
fn mnemonic(s: &str, x: f32, y: f32, color: Color) {
    text(s, x, y, color);
    draw_rectangle(x, y + 2.0, text_w(&s[..1]), 1.0, color);
}

/// Tracks a mouse press on one of several buttons: like Windows, a button
/// only fires if the mouse is released over the one it was pressed on.
#[derive(Default)]
struct Clicks {
    pressed: Option<usize>,
}

impl Clicks {
    fn update(&mut self, rects: &[Rect], mouse: Vec2) -> Option<usize> {
        let hit = rects.iter().position(|r| r.contains(mouse));
        if is_mouse_button_pressed(MouseButton::Left) {
            self.pressed = hit;
        }
        if is_mouse_button_released(MouseButton::Left) {
            return self.pressed.take().filter(|&p| Some(p) == hit);
        }
        None
    }

    /// Whether button `i` should be drawn pushed in.
    fn is_down(&self, i: usize, rect: Rect, mouse: Vec2) -> bool {
        self.pressed == Some(i) && rect.contains(mouse)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuCmd {
    About,
    Level,
}

/// MENU_1: two top-level commands, "&About" (100) and "&Level" (200).
pub struct MenuBar {
    rects: [Rect; 2],
    clicks: Clicks,
}

const MENU_ITEMS: [(&str, MenuCmd); 2] = [("About", MenuCmd::About), ("Level", MenuCmd::Level)];

impl MenuBar {
    pub fn new() -> Self {
        let mut x = 0.0;
        let rects = MENU_ITEMS.map(|(label, _)| {
            let w = text_w(label) + 16.0;
            let r = Rect::new(x, 0.0, w, MENU_H - 1.0);
            x += w;
            r
        });
        MenuBar {
            rects,
            clicks: Clicks::default(),
        }
    }

    pub fn handle_mouse(&mut self, mouse: Vec2) -> Option<MenuCmd> {
        self.clicks
            .update(&self.rects, mouse)
            .map(|i| MENU_ITEMS[i].1)
    }

    pub fn draw(&self, enabled: bool, mouse: Vec2) {
        let w = FB_W as f32;
        draw_rectangle(0.0, 0.0, w, MENU_H, WHITE);
        draw_rectangle(0.0, MENU_H - 1.0, w, 1.0, BLACK);
        for (i, (r, (label, _))) in self.rects.iter().zip(MENU_ITEMS).enumerate() {
            let hot = enabled && self.clicks.is_down(i, *r, mouse);
            if hot {
                draw_rectangle(r.x, r.y, r.w, r.h, NAVY);
            }
            mnemonic(label, r.x + 8.0, 14.0, if hot { WHITE } else { BLACK });
        }
    }
}

/// How a dialog was closed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Closed {
    Ok,
    Cancel,
}

/// A MessageBox(MB_OK) or the Level input dialog, centred in the view.
pub struct Dialog {
    pub modal: Modal,
    /// The Level dialog's edit box.
    edit: String,
    /// Edit text is selected when the dialog opens, so typing replaces it.
    selected: bool,
    rect: Rect,
    buttons: Vec<Rect>,
    labels: &'static [&'static str],
    clicks: Clicks,
}

impl Dialog {
    pub fn new(modal: Modal, initial_text: &str, view: Vec2) -> Self {
        let is_input = modal == Modal::LevelInput;
        let (w, h) = if is_input {
            (220.0, 120.0)
        } else {
            let w = text_w(modal.text()).max(text_w(modal.title())) + 60.0;
            (w.max(160.0), 100.0)
        };
        let rect = Rect::new(
            ((view.x - w) / 2.0).floor(),
            ((view.y - h) / 2.0).floor(),
            w,
            h,
        );

        let (bw, bh, gap) = (64.0, 24.0, 16.0);
        let y = rect.bottom() - bh - 10.0;
        let labels: &'static [&'static str] = if is_input { &["OK", "Cancel"] } else { &["OK"] };
        let total = labels.len() as f32 * (bw + gap) - gap;
        let x0 = rect.x + (rect.w - total) / 2.0;
        let buttons = (0..labels.len())
            .map(|i| Rect::new(x0 + i as f32 * (bw + gap), y, bw, bh))
            .collect();

        Dialog {
            modal,
            edit: initial_text.chars().take(1).collect(),
            selected: true,
            rect,
            buttons,
            labels,
            clicks: Clicks::default(),
        }
    }

    /// What is in the edit box.
    pub fn text(&self) -> &str {
        &self.edit
    }

    /// Returns how the dialog was closed, if it was.
    pub fn handle_input(
        &mut self,
        pressed: &[KeyCode],
        chars: &[char],
        mouse: Vec2,
    ) -> Option<Closed> {
        let is_input = self.modal == Modal::LevelInput;
        let key = |k| pressed.contains(&k);
        if key(KeyCode::Enter) || key(KeyCode::KpEnter) || (!is_input && key(KeyCode::Space)) {
            return Some(Closed::Ok);
        }
        if key(KeyCode::Escape) {
            return Some(Closed::Cancel);
        }
        if is_input {
            if key(KeyCode::Backspace) {
                self.edit.clear();
                self.selected = false;
            }
            // Only a single digit 1..9 can be typed.
            if let Some(&c) = chars.iter().rev().find(|c| ('1'..='9').contains(*c)) {
                self.edit = c.to_string();
                self.selected = false;
            }
        }
        self.clicks.update(&self.buttons, mouse).map(|i| {
            if i == 0 {
                Closed::Ok
            } else {
                Closed::Cancel
            }
        })
    }

    pub fn draw(&self, mouse: Vec2) {
        let r = self.rect;
        // Frame and caption
        draw_rectangle(r.x - 1.0, r.y - 1.0, r.w + 2.0, r.h + 2.0, BLACK);
        draw_rectangle(r.x, r.y, r.w, r.h, WHITE);
        draw_rectangle(r.x, r.y, r.w, 20.0, NAVY);
        let title = self.modal.title();
        text(title, r.x + (r.w - text_w(title)) / 2.0, r.y + 14.0, WHITE);
        // System-menu box at the left of the caption
        draw_rectangle(r.x, r.y, 20.0, 20.0, GRAY);
        draw_rectangle(r.x + 5.0, r.y + 8.0, 10.0, 3.0, WHITE);
        outline(Rect::new(r.x + 5.0, r.y + 8.0, 10.0, 3.0), BLACK);

        let body = self.modal.text();
        if self.modal == Modal::LevelInput {
            text(body, r.x + 16.0, r.y + 44.0, BLACK);
            let e = Rect::new(r.x + 16.0, r.y + 52.0, r.w - 32.0, 22.0);
            draw_rectangle(e.x, e.y, e.w, e.h, WHITE);
            outline(e, BLACK);
            let tw = text_w(&self.edit);
            if self.selected && !self.edit.is_empty() {
                draw_rectangle(e.x + 3.0, e.y + 3.0, tw + 2.0, e.h - 6.0, NAVY);
                text(&self.edit, e.x + 4.0, e.y + 16.0, WHITE);
            } else {
                text(&self.edit, e.x + 4.0, e.y + 16.0, BLACK);
                let caret_on = (get_time() * 2.0) as i64 % 2 == 0;
                if caret_on {
                    draw_rectangle(e.x + 5.0 + tw, e.y + 4.0, 1.0, e.h - 8.0, BLACK);
                }
            }
        } else {
            text(body, r.x + (r.w - text_w(body)) / 2.0, r.y + 48.0, BLACK);
        }

        for (i, (&b, label)) in self.buttons.iter().zip(self.labels).enumerate() {
            button(b, label, i == 0, self.clicks.is_down(i, b, mouse));
        }
    }
}

/// A Windows 3.1 push button; the default button gets a heavier border.
fn button(b: Rect, label: &str, default: bool, down: bool) {
    draw_rectangle(b.x, b.y, b.w, b.h, BLACK);
    let inset = if default { 2.0 } else { 1.0 };
    let i = Rect::new(
        b.x + inset,
        b.y + inset,
        b.w - 2.0 * inset,
        b.h - 2.0 * inset,
    );
    draw_rectangle(i.x, i.y, i.w, i.h, GRAY);
    let (hi, lo) = if down { (DARK, GRAY) } else { (WHITE, DARK) };
    draw_rectangle(i.x, i.y, i.w, 2.0, hi);
    draw_rectangle(i.x, i.y, 2.0, i.h, hi);
    draw_rectangle(i.x, i.bottom() - 2.0, i.w, 2.0, lo);
    draw_rectangle(i.right() - 2.0, i.y, 2.0, i.h, lo);
    let off = if down { 1.0 } else { 0.0 };
    text(
        label,
        b.x + (b.w - text_w(label)) / 2.0 + off,
        b.y + 16.0 + off,
        BLACK,
    );
}
