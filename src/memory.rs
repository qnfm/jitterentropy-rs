#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};
#[cfg(feature = "alloc")]
use core::hint::black_box;
#[cfg(feature = "alloc")]
use zeroize::Zeroize;

#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct MemoryNoise {
    buf: Vec<u8>,
    idx: usize,
}

#[cfg(feature = "alloc")]
impl MemoryNoise {
    pub fn new(size: usize) -> Option<Self> {
        if size == 0 || !size.is_power_of_two() {
            return None;
        }
        Some(Self {
            buf: vec![0u8; size],
            idx: 0,
        })
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline(never)]
    pub fn disturb<F: FnMut(&[u8])>(&mut self, seed: u64, rounds: usize, mut sink: F) {
        let mask = self.buf.len() - 1;
        let mut idx = (self.idx ^ seed as usize) & mask;
        let mut tmp = [0u8; 8];
        for _ in 0..rounds {
            let old = self.buf[idx];
            let new = old.wrapping_add((idx as u8).rotate_left(1)).wrapping_add(1);
            self.buf[idx] = new;
            tmp[0] = old;
            tmp[1] = new;
            tmp[2..].copy_from_slice(&(idx as u64).to_le_bytes()[..6]);
            sink(&tmp);
            idx = idx.wrapping_mul(1_103_515_245).wrapping_add(12_345) & mask;
            black_box(self.buf[idx]);
        }
        self.idx = idx;
        tmp.zeroize();
    }
}

#[cfg(feature = "alloc")]
impl Drop for MemoryNoise {
    fn drop(&mut self) {
        self.buf.zeroize();
    }
}
