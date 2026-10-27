//! The bitmaps from ROCK20.RES, compiled into the binary straight from
//! resources/ (written by tools/extract_res.py).

use crate::game::{Bitmap, Sprites};
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::{Image, ImageFormat};

macro_rules! pngs {
    ($($name:literal),* $(,)?) => {
        [$(include_bytes!(concat!("../resources/bitmaps/", $name, ".png")).as_slice()),*]
    };
}

/// In [`crate::game::Tile::sprite`] order.
const TILE_PNGS: [&[u8]; 27] = pngs![
    "boulder",
    "fboulder",
    "fnboulder",
    "diamond",
    "fdiamond",
    "fndiamond",
    "man1",
    "man2",
    "empty",
    "earth",
    "wall",
    "bounds",
    "door",
    "morph1",
    "morph2",
    "morph3",
    "morph4",
    "life0",
    "life1",
    "life2",
    "life3",
    "life4",
    "life5",
    "life6",
    "life7",
    "life8",
    "life9",
];
const SCORE_PNGS: [&[u8]; 2] = pngs!["score1", "score2"];

fn decode(bytes: &[u8]) -> Image {
    Image::from_file_with_format(bytes, Some(ImageFormat::Png)).expect("embedded PNG")
}

fn bitmap(png: &[u8]) -> Bitmap {
    let img = decode(png);
    Bitmap::new(img.width as usize, img.height as usize, img.bytes)
}

pub fn sprites() -> Sprites {
    let [score_filled, score_empty] = SCORE_PNGS.map(bitmap);
    Sprites {
        tiles: TILE_PNGS.map(bitmap),
        score_filled,
        score_empty,
    }
}

/// The 32x32 program icon, scaled (nearest) to the sizes miniquad wants.
pub fn icon() -> Icon {
    let src = decode(include_bytes!("../resources/icons/icon.png"));
    fn scaled<const N: usize>(src: &Image, size: usize) -> [u8; N] {
        let mut out = [0u8; N];
        for (i, px) in out.chunks_exact_mut(4).enumerate() {
            let (x, y) = (i % size, i / size);
            let sx = x * src.width as usize / size;
            let sy = y * src.height as usize / size;
            let s = (sy * src.width as usize + sx) * 4;
            px.copy_from_slice(&src.bytes[s..s + 4]);
        }
        out
    }
    Icon {
        small: scaled::<{ 16 * 16 * 4 }>(&src, 16),
        medium: scaled::<{ 32 * 32 * 4 }>(&src, 32),
        big: scaled::<{ 64 * 64 * 4 }>(&src, 64),
    }
}

#[cfg(test)]
mod tests {
    use crate::game::CELL;

    #[test]
    fn sprites_decode() {
        let s = super::sprites();
        assert!(s.tiles.iter().all(|b| (b.w, b.h) == (CELL, CELL)));
        assert_eq!((s.score_filled.w, s.score_filled.h), (2, CELL));
        assert_eq!((s.score_empty.w, s.score_empty.h), (2, CELL));
    }
}
