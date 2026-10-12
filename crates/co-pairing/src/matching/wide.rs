//! Fixed-width unsigned integers with wrapping arithmetic, used as edge
//! weights. All values in one matching problem share the same width (number
//! of 64-bit limbs), and arithmetic wraps modulo 2^(64·limbs). The Dutch
//! engine relies on that wrap-around to encode "slightly less than".
//!
//! Semantics follow bbpPairings' `DynamicUint` (Apache-2.0) exactly,
//! including `shift_grow`.

use smallvec::{SmallVec, smallvec};
use std::cmp::Ordering;
use std::fmt;

/// Weights up to 512 bits stay inline (no heap allocation).
type Limbs = SmallVec<[u64; 8]>;

/// Little-endian limbs.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Wide {
    limbs: Limbs,
}

fn bits_to_represent(v: u64) -> u32 {
    64 - v.leading_zeros()
}

impl Wide {
    /// A one-limb value.
    pub fn from_u64(v: u64) -> Self {
        Self {
            limbs: smallvec![v],
        }
    }

    /// Zero with the same width as `self` (`x & 0u` in the original).
    pub fn zero_like(&self) -> Self {
        Self {
            limbs: smallvec![0; self.limbs.len()],
        }
    }

    /// `small` with the same width as `self`.
    pub fn small_like(&self, small: u64) -> Self {
        let mut w = self.zero_like();
        w.limbs[0] = small;
        w
    }

    pub fn limbs(&self) -> &[u64] {
        &self.limbs
    }

    pub fn limbs_mut(&mut self) -> &mut [u64] {
        &mut self.limbs
    }

    pub fn width(&self) -> usize {
        self.limbs.len()
    }

    pub fn is_zero(&self) -> bool {
        is_zero(&self.limbs)
    }

    /// Copies `other`'s value into `self` (same width).
    pub fn assign(&mut self, other: &[u64]) {
        self.limbs.copy_from_slice(other);
    }

    pub fn or_small(&mut self, v: u64) {
        self.limbs[0] |= v;
    }

    /// `&= v`: keeps `low & v` and clears the higher limbs.
    pub fn and_small(&mut self, v: u64) {
        self.limbs[0] &= v;
        for l in &mut self.limbs[1..] {
            *l = 0;
        }
    }

    pub fn shl(&mut self, shift: usize) {
        shl(&mut self.limbs, shift);
    }

    pub fn shr(&mut self, shift: usize) {
        shr(&mut self.limbs, shift);
    }

    pub fn add_small(&mut self, v: u64) {
        add_small(&mut self.limbs, v);
    }

    pub fn sub_small(&mut self, v: u64) {
        sub_small(&mut self.limbs, v);
    }

    pub fn add(&mut self, other: &[u64]) {
        add(&mut self.limbs, other);
    }

    pub fn sub(&mut self, other: &[u64]) {
        sub(&mut self.limbs, other);
    }

    pub fn inc(&mut self) {
        add_small(&mut self.limbs, 1);
    }

    /// Sets the bit at `pos` (`|= 1 << pos`).
    pub fn or_bit(&mut self, pos: usize) {
        if pos / 64 < self.limbs.len() {
            self.limbs[pos / 64] |= 1 << (pos % 64);
        }
    }

    /// Adds `1 << pos` (wrapping).
    pub fn add_bit(&mut self, pos: usize) {
        let mut b = self.zero_like();
        b.or_bit(pos);
        add(&mut self.limbs, &b.limbs);
    }

    /// Left shift that first widens the value so nothing is lost.
    pub fn shift_grow(&mut self, shift: usize) {
        let mut extra = shift;
        for &limb in self.limbs.iter().rev() {
            if limb != 0 {
                let buffer = (64 - bits_to_represent(limb)) as usize;
                extra = extra.saturating_sub(buffer);
                break;
            } else if extra <= 64 {
                extra = 0;
                break;
            } else {
                extra -= 64;
            }
        }
        while extra >= 64 {
            self.limbs.push(0);
            extra -= 64;
        }
        if extra > 0 {
            self.limbs.push(0);
        }
        shl(&mut self.limbs, shift);
    }
}

impl PartialOrd for Wide {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Wide {
    fn cmp(&self, other: &Self) -> Ordering {
        cmp(&self.limbs, &other.limbs)
    }
}

impl fmt::Debug for Wide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for l in self.limbs.iter().rev() {
            write!(f, "{l:016x}")?;
        }
        Ok(())
    }
}

pub fn is_zero(a: &[u64]) -> bool {
    a.iter().all(|&l| l == 0)
}

pub fn cmp(a: &[u64], b: &[u64]) -> Ordering {
    debug_assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().rev().zip(b.iter().rev()) {
        match x.cmp(y) {
            Ordering::Equal => continue,
            o => return o,
        }
    }
    Ordering::Equal
}

pub fn lt(a: &[u64], b: &[u64]) -> bool {
    cmp(a, b) == Ordering::Less
}

pub fn shl(a: &mut [u64], shift: usize) {
    let n = a.len();
    let limb_shift = shift / 64;
    let bit = shift % 64;
    for i in (0..n).rev() {
        let src = i as isize - limb_shift as isize;
        let mut v = 0u64;
        if src >= 0 {
            let s = src as usize;
            v = a[s] << bit;
            if bit > 0 && s > 0 {
                v |= a[s - 1] >> (64 - bit);
            }
        }
        a[i] = v;
    }
}

pub fn shr(a: &mut [u64], shift: usize) {
    let n = a.len();
    let limb_shift = shift / 64;
    let bit = shift % 64;
    for i in 0..n {
        let s = i + limb_shift;
        let mut v = 0u64;
        if s < n {
            v = a[s] >> bit;
            if bit > 0 && s + 1 < n {
                v |= a[s + 1] << (64 - bit);
            }
        }
        a[i] = v;
    }
}

pub fn add_small(a: &mut [u64], mut v: u64) {
    for l in a.iter_mut() {
        let (r, carry) = l.overflowing_add(v);
        *l = r;
        if !carry {
            return;
        }
        v = 1;
    }
}

pub fn sub_small(a: &mut [u64], mut v: u64) {
    for l in a.iter_mut() {
        let (r, borrow) = l.overflowing_sub(v);
        *l = r;
        if !borrow {
            return;
        }
        v = 1;
    }
}

pub fn add(a: &mut [u64], b: &[u64]) {
    debug_assert_eq!(a.len(), b.len());
    let mut carry = false;
    for (x, &y) in a.iter_mut().zip(b) {
        let (r1, c1) = x.overflowing_add(y);
        let (r2, c2) = r1.overflowing_add(carry as u64);
        *x = r2;
        carry = c1 || c2;
    }
}

pub fn sub(a: &mut [u64], b: &[u64]) {
    debug_assert_eq!(a.len(), b.len());
    let mut borrow = false;
    for (x, &y) in a.iter_mut().zip(b) {
        let (r1, b1) = x.overflowing_sub(y);
        let (r2, b2) = r1.overflowing_sub(borrow as u64);
        *x = r2;
        borrow = b1 || b2;
    }
}

/// `out = a + b - c` (wrapping), all the same width, in one pass.
pub fn add_sub_into(out: &mut [u64], a: &[u64], b: &[u64], c: &[u64]) {
    let mut carry: i128 = 0;
    for i in 0..out.len() {
        let acc = i128::from(a[i]) + i128::from(b[i]) - i128::from(c[i]) + carry;
        out[i] = acc as u64;
        carry = acc >> 64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(limbs: &[u64]) -> Wide {
        Wide {
            limbs: limbs.iter().copied().collect(),
        }
    }

    #[test]
    fn shifts_across_limbs() {
        let mut a = w(&[0x8000_0000_0000_0001, 0]);
        a.shl(1);
        assert_eq!(a.limbs.as_slice(), [2, 1]);
        a.shl(64);
        assert_eq!(a.limbs.as_slice(), [0, 2]);
        a.shr(65);
        assert_eq!(a.limbs.as_slice(), [1, 0]);
    }

    #[test]
    fn wrapping_arithmetic() {
        let mut a = w(&[0, 0]);
        a.sub_small(1);
        assert_eq!(a.limbs.as_slice(), [u64::MAX, u64::MAX]);
        a.add_small(2);
        assert_eq!(a.limbs.as_slice(), [1, 0]);
        let mut b = w(&[u64::MAX, 0]);
        b.add(&[1, 0]);
        assert_eq!(b.limbs.as_slice(), [0, 1]);
        b.sub(&[1, 0]);
        assert_eq!(b.limbs.as_slice(), [u64::MAX, 0]);
    }

    #[test]
    fn shift_grow_widens_only_when_needed() {
        let mut a = Wide::from_u64(2);
        a.shift_grow(10);
        assert_eq!(a.limbs.as_slice(), [2048]);
        a.shift_grow(60);
        assert_eq!(a.width(), 2);
        assert_eq!(a.limbs.as_slice(), [0, 2048 >> 4]);
        let mut z = Wide::from_u64(0);
        z.shift_grow(64);
        assert_eq!(z.width(), 1);
    }

    #[test]
    fn add_sub_matches_two_steps() {
        let cases: [([u64; 2], [u64; 2], [u64; 2]); 4] = [
            ([u64::MAX, 0], [1, 0], [0, 0]),
            ([0, 0], [0, 0], [1, 0]),
            ([5, 7], [u64::MAX, 3], [u64::MAX, 9]),
            ([0, 1], [0, 0], [u64::MAX, 0]),
        ];
        for (a, b, c) in cases {
            let mut out = [0u64; 2];
            add_sub_into(&mut out, &a, &b, &c);
            let mut expect = a;
            add(&mut expect, &b);
            sub(&mut expect, &c);
            assert_eq!(out, expect);
        }
    }

    #[test]
    fn ordering_uses_high_limbs_first() {
        assert!(w(&[5, 1]) > w(&[u64::MAX, 0]));
        assert!(w(&[1, 1]) < w(&[2, 1]));
    }
}
