//! The picture of the window's client area, and the bitmaps drawn into it.
//!
//! The original never kept a picture of the screen.  It `BitBlt`ed each cell
//! straight to the window when it changed, stretched two 2-pixel strips into
//! the score bar, and flashed the exit with `InvertRect`.  This framebuffer
//! does the same operations so the result looks the same, and records
//! whether anything changed so the front end only uploads it when needed.

use super::tile::Tile;

/// Size of a cell, in pixels.
pub const CELL: usize = 20;
/// Client area: 31 x 20 cells.
pub const FB_W: usize = 31 * CELL;
pub const FB_H: usize = 20 * CELL;

const BYTES_PER_PIXEL: usize = 4;

/// An RGBA picture, `w` x `h`, rows top to bottom.
#[derive(Clone)]
pub struct Bitmap {
    pub w: usize,
    pub h: usize,
    pub rgba: Box<[u8]>,
}

impl Bitmap {
    pub fn new(w: usize, h: usize, rgba: impl Into<Box<[u8]>>) -> Self {
        let rgba = rgba.into();
        assert_eq!(rgba.len(), w * h * BYTES_PER_PIXEL, "bitmap size");
        Bitmap { w, h, rgba }
    }

    /// A single-colour bitmap, for tests.
    #[cfg(test)]
    pub fn solid(w: usize, h: usize, rgb: [u8; 3]) -> Self {
        let px = [rgb[0], rgb[1], rgb[2], 255];
        Bitmap::new(w, h, px.repeat(w * h))
    }

    fn row(&self, y: usize) -> &[u8] {
        let n = self.w * BYTES_PER_PIXEL;
        &self.rgba[y * n..][..n]
    }
}

/// Every bitmap the game draws, decoded once at startup.
pub struct Sprites {
    /// 20 x 20 cell pictures, indexed by [`Tile::sprite`].
    pub tiles: [Bitmap; Tile::SPRITES],
    /// The 2 x 20 strips stretched into the score bar: filled, then unfilled.
    pub score_filled: Bitmap,
    pub score_empty: Bitmap,
}

impl Sprites {
    /// Distinct solid colours, for tests.
    #[cfg(test)]
    pub fn blank() -> Self {
        Sprites {
            tiles: std::array::from_fn(|i| Bitmap::solid(CELL, CELL, [i as u8, 0, 0])),
            score_filled: Bitmap::solid(2, CELL, [0, 1, 0]),
            score_empty: Bitmap::solid(2, CELL, [0, 2, 0]),
        }
    }
}

pub struct Framebuffer {
    rgba: Vec<u8>,
    dirty: bool,
}

impl Framebuffer {
    pub fn new() -> Self {
        Framebuffer {
            rgba: vec![255; FB_W * FB_H * BYTES_PER_PIXEL],
            dirty: true,
        }
    }

    /// The pixels, RGBA, rows top to bottom.
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    /// True if anything was drawn since the last call.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    fn row_mut(&mut self, y: usize) -> &mut [u8] {
        let n = FB_W * BYTES_PER_PIXEL;
        &mut self.rgba[y * n..][..n]
    }

    /// `BitBlt`: copy `src` to (x, y), clipped to the client area.
    pub fn blit(&mut self, x: i32, y: i32, src: &Bitmap) {
        let x0 = x.max(0);
        let x1 = (x + src.w as i32).min(FB_W as i32);
        let y0 = y.max(0);
        let y1 = (y + src.h as i32).min(FB_H as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let (sx, len) = ((x0 - x) as usize, (x1 - x0) as usize);
        for fy in y0..y1 {
            let s = &src.row((fy - y) as usize)[sx * BYTES_PER_PIXEL..][..len * BYTES_PER_PIXEL];
            self.row_mut(fy as usize)[x0 as usize * BYTES_PER_PIXEL..][..s.len()]
                .copy_from_slice(s);
        }
        self.dirty = true;
    }

    /// `StretchBlt` of `src` to `w` x `src.h` at (x, 0), picking the nearest
    /// source column as GDI does when enlarging.
    pub fn stretch_columns(&mut self, x: i32, w: i32, src: &Bitmap) {
        if w <= 0 {
            return;
        }
        let visible = (0..w).filter(|dx| (0..FB_W as i32).contains(&(x + dx)));
        for y in 0..src.h.min(FB_H) {
            let s = src.row(y);
            let d = self.row_mut(y);
            for dx in visible.clone() {
                let fx = (x + dx) as usize;
                let sx = (dx * src.w as i32 / w) as usize;
                d[fx * BYTES_PER_PIXEL..][..BYTES_PER_PIXEL]
                    .copy_from_slice(&s[sx * BYTES_PER_PIXEL..][..BYTES_PER_PIXEL]);
            }
        }
        self.dirty = true;
    }

    /// `InvertRect`: invert the colour of a w x h rectangle at (x, y).
    pub fn invert(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let x0 = x.max(0) as usize;
        let x1 = (x + w).clamp(0, FB_W as i32) as usize;
        for fy in y.max(0)..(y + h).min(FB_H as i32) {
            for px in self.row_mut(fy as usize)[x0 * BYTES_PER_PIXEL..x1 * BYTES_PER_PIXEL]
                .chunks_exact_mut(BYTES_PER_PIXEL)
            {
                px[0] ^= 0xFF;
                px[1] ^= 0xFF;
                px[2] ^= 0xFF;
            }
        }
        self.dirty = true;
    }
}
