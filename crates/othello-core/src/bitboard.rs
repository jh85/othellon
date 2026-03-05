use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BitBoard<const N: usize, const W: usize> {
    pub words: [u64; W],
}

impl<const N: usize, const W: usize> BitBoard<N, W> {
    pub const fn zero() -> Self {
        Self { words: [0; W] }
    }

    pub fn valid_mask() -> Self {
        let total_bits = N * N;
        let mut words = [0u64; W];
        let mut w = 0;
        while w < W {
            let start = w * 64;
            let end = start + 64;
            if end <= total_bits {
                words[w] = u64::MAX;
            } else if start < total_bits {
                let bits = total_bits - start;
                words[w] = (1u64 << bits) - 1;
            }
            w += 1;
        }
        Self { words }
    }

    pub fn single(idx: usize) -> Self {
        debug_assert!(idx < N * N);
        let mut bb = Self::zero();
        bb.words[idx / 64] = 1u64 << (idx % 64);
        bb
    }

    pub fn not_col_mask(col: usize) -> Self {
        let mut bb = Self::valid_mask();
        for row in 0..N {
            bb.clear_mut(row * N + col);
        }
        bb
    }

    #[inline]
    pub fn get(&self, idx: usize) -> bool {
        (self.words[idx / 64] >> (idx % 64)) & 1 == 1
    }

    #[inline]
    pub fn set_mut(&mut self, idx: usize) {
        self.words[idx / 64] |= 1u64 << (idx % 64);
    }

    #[inline]
    pub fn clear_mut(&mut self, idx: usize) {
        self.words[idx / 64] &= !(1u64 << (idx % 64));
    }

    pub fn is_zero(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    pub fn count_ones(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    /// Shift left by k bits (towards higher bit indices = south/east)
    pub fn shl(&self, k: usize) -> Self {
        if k >= N * N {
            return Self::zero();
        }
        let word_shift = k / 64;
        let bit_shift = k % 64;
        let mut words = [0u64; W];
        for i in word_shift..W {
            words[i] = self.words[i - word_shift] << bit_shift;
            if bit_shift > 0 && i > word_shift {
                words[i] |= self.words[i - word_shift - 1] >> (64 - bit_shift);
            }
        }
        let result = Self { words };
        result & Self::valid_mask()
    }

    /// Shift right by k bits (towards lower bit indices = north/west)
    pub fn shr(&self, k: usize) -> Self {
        if k >= N * N {
            return Self::zero();
        }
        let word_shift = k / 64;
        let bit_shift = k % 64;
        let mut words = [0u64; W];
        for i in 0..W - word_shift {
            words[i] = self.words[i + word_shift] >> bit_shift;
            if bit_shift > 0 && i + word_shift + 1 < W {
                words[i] |= self.words[i + word_shift + 1] << (64 - bit_shift);
            }
        }
        Self { words }
    }

    pub fn iter_ones(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for (word_idx, &word) in self.words.iter().enumerate() {
            let base = word_idx * 64;
            let mut w = word;
            while w != 0 {
                let idx = w.trailing_zeros() as usize;
                result.push(base + idx);
                w &= w - 1;
            }
        }
        result
    }
}

impl<const N: usize, const W: usize> BitAnd for BitBoard<N, W> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        let mut words = [0u64; W];
        for i in 0..W {
            words[i] = self.words[i] & rhs.words[i];
        }
        Self { words }
    }
}

impl<const N: usize, const W: usize> BitOr for BitBoard<N, W> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        let mut words = [0u64; W];
        for i in 0..W {
            words[i] = self.words[i] | rhs.words[i];
        }
        Self { words }
    }
}

impl<const N: usize, const W: usize> BitXor for BitBoard<N, W> {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        let mut words = [0u64; W];
        for i in 0..W {
            words[i] = self.words[i] ^ rhs.words[i];
        }
        Self { words }
    }
}

impl<const N: usize, const W: usize> Not for BitBoard<N, W> {
    type Output = Self;
    fn not(self) -> Self {
        let mut words = [0u64; W];
        for i in 0..W {
            words[i] = !self.words[i];
        }
        Self { words }
    }
}

impl<const N: usize, const W: usize> BitAndAssign for BitBoard<N, W> {
    fn bitand_assign(&mut self, rhs: Self) {
        for i in 0..W {
            self.words[i] &= rhs.words[i];
        }
    }
}

impl<const N: usize, const W: usize> BitOrAssign for BitBoard<N, W> {
    fn bitor_assign(&mut self, rhs: Self) {
        for i in 0..W {
            self.words[i] |= rhs.words[i];
        }
    }
}

impl<const N: usize, const W: usize> BitXorAssign for BitBoard<N, W> {
    fn bitxor_assign(&mut self, rhs: Self) {
        for i in 0..W {
            self.words[i] ^= rhs.words[i];
        }
    }
}

impl<const N: usize, const W: usize> fmt::Debug for BitBoard<N, W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for row in 0..N {
            for col in 0..N {
                write!(f, "{}", if self.get(row * N + col) { '1' } else { '0' })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        let bb = BitBoard::<8, 1>::zero();
        assert!(bb.is_zero());
        assert_eq!(bb.count_ones(), 0);
    }

    #[test]
    fn test_single_and_get() {
        let bb = BitBoard::<8, 1>::single(27);
        assert!(bb.get(27));
        assert!(!bb.get(26));
        assert_eq!(bb.count_ones(), 1);
    }

    #[test]
    fn test_set_clear() {
        let mut bb = BitBoard::<8, 1>::zero();
        bb.set_mut(10);
        bb.set_mut(20);
        assert!(bb.get(10));
        assert!(bb.get(20));
        assert_eq!(bb.count_ones(), 2);
        bb.clear_mut(10);
        assert!(!bb.get(10));
        assert_eq!(bb.count_ones(), 1);
    }

    #[test]
    fn test_valid_mask_8x8() {
        let mask = BitBoard::<8, 1>::valid_mask();
        assert_eq!(mask.words[0], u64::MAX);
    }

    #[test]
    fn test_valid_mask_4x4() {
        let mask = BitBoard::<4, 1>::valid_mask();
        assert_eq!(mask.words[0], 0xFFFF);
    }

    #[test]
    fn test_valid_mask_10x10() {
        let mask = BitBoard::<10, 2>::valid_mask();
        assert_eq!(mask.words[0], u64::MAX);
        assert_eq!(mask.words[1], (1u64 << 36) - 1);
    }

    #[test]
    fn test_shl_single_word() {
        let bb = BitBoard::<8, 1>::single(0);
        let shifted = bb.shl(8);
        assert!(shifted.get(8));
        assert_eq!(shifted.count_ones(), 1);
    }

    #[test]
    fn test_shr_single_word() {
        let bb = BitBoard::<8, 1>::single(63);
        let shifted = bb.shr(8);
        assert!(shifted.get(55));
        assert_eq!(shifted.count_ones(), 1);
    }

    #[test]
    fn test_shl_multi_word() {
        let bb = BitBoard::<10, 2>::single(60);
        let shifted = bb.shl(10);
        assert!(shifted.get(70));
        assert_eq!(shifted.count_ones(), 1);
    }

    #[test]
    fn test_shr_multi_word() {
        let bb = BitBoard::<10, 2>::single(70);
        let shifted = bb.shr(10);
        assert!(shifted.get(60));
        assert_eq!(shifted.count_ones(), 1);
    }

    #[test]
    fn test_shl_cross_word_boundary() {
        let bb = BitBoard::<10, 2>::single(63);
        let shifted = bb.shl(1);
        assert!(shifted.get(64));
        assert_eq!(shifted.count_ones(), 1);
    }

    #[test]
    fn test_shr_cross_word_boundary() {
        let bb = BitBoard::<10, 2>::single(64);
        let shifted = bb.shr(1);
        assert!(shifted.get(63));
        assert_eq!(shifted.count_ones(), 1);
    }

    #[test]
    fn test_bitwise_ops() {
        let a = BitBoard::<8, 1>::single(5);
        let b = BitBoard::<8, 1>::single(10);
        let c = a | b;
        assert!(c.get(5));
        assert!(c.get(10));
        assert_eq!(c.count_ones(), 2);

        let d = c & a;
        assert!(d.get(5));
        assert!(!d.get(10));

        let e = !BitBoard::<4, 1>::zero() & BitBoard::<4, 1>::valid_mask();
        assert_eq!(e.count_ones(), 16);
    }

    #[test]
    fn test_iter_ones() {
        let mut bb = BitBoard::<8, 1>::zero();
        bb.set_mut(3);
        bb.set_mut(7);
        bb.set_mut(63);
        let ones = bb.iter_ones();
        assert_eq!(ones, vec![3, 7, 63]);
    }

    #[test]
    fn test_not_col_mask() {
        let mask = BitBoard::<4, 1>::not_col_mask(3);
        for row in 0..4 {
            for col in 0..4 {
                if col == 3 {
                    assert!(!mask.get(row * 4 + col));
                } else {
                    assert!(mask.get(row * 4 + col));
                }
            }
        }
    }
}
