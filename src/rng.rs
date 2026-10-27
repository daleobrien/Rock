//! Turbo Pascal's `Random`, bit-for-bit.
//!
//! Verified against the System unit code linked into the original ROCK.EXE:
//! `RandSeed := RandSeed * $08088405 + 1`, and `Random(n)` returns the high
//! word of the new seed times `n`, shifted right by 16.  Because ROCK sets
//! `RandSeed := $FFFF` at the start of every level, the "random" layouts are
//! in fact fixed, and reproducing this generator reproduces them exactly.

#[derive(Clone, Debug)]
pub struct TpRandom {
    pub seed: u32,
}

impl TpRandom {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// `Random(range)` for a Word range; returns 0..range-1 (0 when range is 0).
    pub fn random(&mut self, range: u16) -> i32 {
        self.seed = self.seed.wrapping_mul(134_775_813).wrapping_add(1);
        (((self.seed >> 16) * range as u32) >> 16) as i32
    }

    /// Move on `n` numbers, as if `Random` had been called `n` times.
    pub fn skip(&mut self, n: u32) {
        for _ in 0..n {
            self.random(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_sequence() {
        // RandSeed = $FFFF -> $FFFF * 134775813 + 1 (mod 2^32)
        let mut r = TpRandom::new(0xFFFF);
        let s1 = 0xFFFFu32.wrapping_mul(134_775_813).wrapping_add(1);
        let v = r.random(19);
        assert_eq!(r.seed, s1);
        assert_eq!(v, (((s1 >> 16) * 19) >> 16) as i32);
        assert!(v < 19);
        assert_eq!(TpRandom::new(5).random(1), 0);
    }
}
