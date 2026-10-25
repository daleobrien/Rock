//! The bitmaps from ROCK20.RES, compiled into the binary straight from
//! resources/ (written by tools/extract_res.py).

use crate::game::Sprites;
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::{Image, ImageFormat};

macro_rules! png {
    ($name:literal) => {
        include_bytes!(concat!("../resources/bitmaps/", $name, ".png"))
    };
}

fn decode(bytes: &[u8]) -> Image {
    Image::from_file_with_format(bytes, Some(ImageFormat::Png)).expect("embedded PNG")
}

fn pixels(bytes: &[u8]) -> Vec<u32> {
    decode(bytes)
        .bytes
        .chunks_exact(4)
        .map(|p| (p[0] as u32) << 16 | (p[1] as u32) << 8 | p[2] as u32)
        .collect()
}

pub fn sprites() -> Sprites {
    Sprites {
        boulder: pixels(png!("boulder")),
        fboulder: pixels(png!("fboulder")),
        fnboulder: pixels(png!("fnboulder")),
        diamond: pixels(png!("diamond")),
        fdiamond: pixels(png!("fdiamond")),
        fndiamond: pixels(png!("fndiamond")),
        man1: pixels(png!("man1")),
        man2: pixels(png!("man2")),
        empty: pixels(png!("empty")),
        earth: pixels(png!("earth")),
        wall: pixels(png!("wall")),
        bounds: pixels(png!("bounds")),
        door: pixels(png!("door")),
        morph: [
            pixels(png!("morph1")),
            pixels(png!("morph2")),
            pixels(png!("morph3")),
            pixels(png!("morph4")),
        ],
        life: [
            pixels(png!("life0")),
            pixels(png!("life1")),
            pixels(png!("life2")),
            pixels(png!("life3")),
            pixels(png!("life4")),
            pixels(png!("life5")),
            pixels(png!("life6")),
            pixels(png!("life7")),
            pixels(png!("life8")),
            pixels(png!("life9")),
        ],
        score1: pixels(png!("score1")),
        score2: pixels(png!("score2")),
    }
}

/// The 32x32 program icon, scaled (nearest) to the sizes miniquad wants.
pub fn icon() -> Icon {
    let src = decode(include_bytes!("../resources/icons/icon.png"));
    fn scaled<const N: usize>(src: &Image, size: usize) -> [u8; N] {
        let mut out = [0u8; N];
        for y in 0..size {
            for x in 0..size {
                let sx = x * src.width as usize / size;
                let sy = y * src.height as usize / size;
                let s = (sy * src.width as usize + sx) * 4;
                out[(y * size + x) * 4..][..4].copy_from_slice(&src.bytes[s..s + 4]);
            }
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
    #[test]
    fn sprites_decode() {
        let s = super::sprites();
        assert_eq!(s.boulder.len(), 400);
        assert_eq!(s.score1.len(), 40);
        assert!(s.life.iter().all(|l| l.len() == 400));
    }
}
