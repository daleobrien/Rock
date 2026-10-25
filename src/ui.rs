//! Stand-ins for the Windows 3.1 chrome the original got for free: the menu
//! bar from the MENU_1 resource, MessageBox, and OWL's TInputDialog.

use crate::game::Modal;
use macroquad::prelude::*;

pub const MENU_H: f32 = 20.0;

const NAVY: Color = Color::new(0.0, 0.0, 0.5, 1.0);
const GRAY: Color = Color::new(0.75, 0.75, 0.75, 1.0);
const DARK: Color = Color::new(0.5, 0.5, 0.5, 1.0);
const FONT: u16 = 16;

fn text(s: &str, x: f32, y: f32, color: Color) {
    draw_text(s, x.floor(), y.floor(), FONT as f32, color);
}

/// A 1px rectangle outline drawn with filled rects, so it stays crisp when
/// the virtual screen is scaled up.
fn outline(r: Rect, color: Color) {
    draw_rectangle(r.x, r.y, r.w, 1.0, color);
    draw_rectangle(r.x, r.y + r.h - 1.0, r.w, 1.0, color);
    draw_rectangle(r.x, r.y, 1.0, r.h, color);
    draw_rectangle(r.x + r.w - 1.0, r.y, 1.0, r.h, color);
}

fn text_w(s: &str) -> f32 {
    measure_text(s, None, FONT, 1.0).width
}

/// Text with the first letter underlined, for "&About" style labels.
fn mnemonic(s: &str, x: f32, y: f32, color: Color) {
    text(s, x, y, color);
    let w = text_w(&s[..1]);
    draw_rectangle(x, y + 2.0, w, 1.0, color);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MenuCmd {
    /// cm_first + 100
    About,
    /// cm_first + 200
    Level,
}

/// MENU_1: two top-level commands, "&About" (100) and "&Level" (200).
pub struct MenuBar {
    pressed: Option<usize>,
}

const MENU_ITEMS: [(&str, MenuCmd); 2] = [("About", MenuCmd::About), ("Level", MenuCmd::Level)];

impl MenuBar {
    pub fn new() -> Self {
        MenuBar { pressed: None }
    }

    fn item_rects() -> [Rect; 2] {
        let mut x = 0.0;
        let mut out = [Rect::default(); 2];
        for (i, (label, _)) in MENU_ITEMS.iter().enumerate() {
            let w = text_w(label) + 16.0;
            out[i] = Rect::new(x, 0.0, w, MENU_H - 1.0);
            x += w;
        }
        out
    }

    /// Windows menus act on button release over the item pressed.
    pub fn handle_mouse(&mut self, mouse: Vec2) -> Option<MenuCmd> {
        let hit = Self::item_rects().iter().position(|r| r.contains(mouse));
        if is_mouse_button_pressed(MouseButton::Left) {
            self.pressed = hit;
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(p) = self.pressed.take().filter(|&p| Some(p) == hit) {
                return Some(MENU_ITEMS[p].1);
            }
        }
        None
    }

    pub fn draw(&self, enabled: bool) {
        draw_rectangle(0.0, 0.0, screen_w(), MENU_H, WHITE);
        draw_rectangle(0.0, MENU_H - 1.0, screen_w(), 1.0, BLACK);
        for (i, r) in Self::item_rects().iter().enumerate() {
            let hot = enabled && self.pressed == Some(i) && r.contains(cur_mouse());
            let fg = if hot { WHITE } else { BLACK };
            if hot {
                draw_rectangle(r.x, r.y, r.w, r.h, NAVY);
            }
            mnemonic(MENU_ITEMS[i].0, r.x + 8.0, 14.0, fg);
        }
    }
}

fn screen_w() -> f32 {
    crate::game::FB_W as f32
}

thread_local!(static MOUSE: std::cell::Cell<Vec2> = const { std::cell::Cell::new(Vec2::ZERO) });

/// The mouse position in virtual-screen coordinates, set by main.rs each frame.
pub fn set_mouse(v: Vec2) {
    MOUSE.with(|m| m.set(v));
}

fn cur_mouse() -> Vec2 {
    MOUSE.with(|m| m.get())
}

/// A MessageBox(MB_OK) or the Level input dialog.
pub struct Dialog {
    pub modal: Modal,
    edit: String,
    /// Edit text is selected when the dialog opens, so typing replaces it.
    selected: bool,
    pressed: Option<usize>,
    rect: Rect,
}

impl Dialog {
    pub fn new(modal: Modal, level_text: &str) -> Self {
        let is_input = modal == Modal::LevelInput;
        let w = if is_input {
            220.0
        } else {
            (text_w(modal.text()).max(text_w(modal.title())) + 60.0).max(160.0)
        };
        let h = if is_input { 120.0 } else { 100.0 };
        Dialog {
            modal,
            edit: level_text.chars().take(1).collect(),
            selected: true,
            pressed: None,
            rect: Rect::new(0.0, 0.0, w, h),
        }
    }

    fn placed(&self, vw: f32, vh: f32) -> Rect {
        let r = self.rect;
        Rect::new(
            ((vw - r.w) / 2.0).floor(),
            ((vh - r.h) / 2.0).floor(),
            r.w,
            r.h,
        )
    }

    /// Button rectangles relative to the dialog: OK (and Cancel).
    fn buttons(&self, at: Rect) -> Vec<Rect> {
        let bw = 64.0;
        let bh = 24.0;
        let y = at.y + at.h - bh - 10.0;
        if self.modal == Modal::LevelInput {
            let gap = 16.0;
            let x0 = at.x + (at.w - 2.0 * bw - gap) / 2.0;
            vec![
                Rect::new(x0, y, bw, bh),
                Rect::new(x0 + bw + gap, y, bw, bh),
            ]
        } else {
            vec![Rect::new(at.x + (at.w - bw) / 2.0, y, bw, bh)]
        }
    }

    fn ok(&self) -> Option<Option<String>> {
        if self.modal == Modal::LevelInput {
            Some(Some(self.edit.clone()))
        } else {
            Some(None)
        }
    }

    /// Returns Some(result) when the dialog closes: Some(Some(text)) for OK
    /// on the input dialog, Some(None) for Cancel or a message box.
    pub fn handle_input(
        &mut self,
        pressed: &[KeyCode],
        chars: &[char],
        mouse: Vec2,
    ) -> Option<Option<String>> {
        let is_input = self.modal == Modal::LevelInput;
        if pressed.contains(&KeyCode::Enter) || pressed.contains(&KeyCode::KpEnter) {
            return self.ok();
        }
        if pressed.contains(&KeyCode::Escape) {
            return Some(None);
        }
        if !is_input && pressed.contains(&KeyCode::Space) {
            return Some(None);
        }
        if is_input {
            if pressed.contains(&KeyCode::Backspace) {
                if self.selected {
                    self.edit.clear();
                } else {
                    self.edit.pop();
                }
                self.selected = false;
            }
            // Only a single digit 1..9 can be typed.
            for &c in chars {
                if ('1'..='9').contains(&c) {
                    self.edit.clear();
                    self.edit.push(c);
                    self.selected = false;
                }
            }
        }
        let at = self.placed(crate::VIEW_W, crate::VIEW_H);
        let buttons = self.buttons(at);
        let hit = buttons.iter().position(|r| r.contains(mouse));
        if is_mouse_button_pressed(MouseButton::Left) {
            self.pressed = hit;
        }
        if is_mouse_button_released(MouseButton::Left) {
            let p = self.pressed.take();
            if p.is_some() && p == hit {
                return if p == Some(0) { self.ok() } else { Some(None) };
            }
        }
        None
    }

    pub fn draw(&self, vw: f32, vh: f32) {
        let r = self.placed(vw, vh);
        // Frame + title bar
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
                if (get_time() * 2.0) as i64 % 2 == 0 {
                    draw_rectangle(e.x + 5.0 + tw, e.y + 4.0, 1.0, e.h - 8.0, BLACK);
                }
            }
        } else {
            text(body, r.x + (r.w - text_w(body)) / 2.0, r.y + 48.0, BLACK);
        }

        let labels: &[&str] = if self.modal == Modal::LevelInput {
            &["OK", "Cancel"]
        } else {
            &["OK"]
        };
        let mouse = cur_mouse();
        for (i, b) in self.buttons(r).iter().enumerate() {
            let down = self.pressed == Some(i) && b.contains(mouse);
            button(*b, labels[i], i == 0, down);
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
    draw_rectangle(i.x, i.y + i.h - 2.0, i.w, 2.0, lo);
    draw_rectangle(i.x + i.w - 2.0, i.y, 2.0, i.h, lo);
    let off = if down { 1.0 } else { 0.0 };
    text(
        label,
        b.x + (b.w - text_w(label)) / 2.0 + off,
        b.y + 16.0 + off,
        BLACK,
    );
}
