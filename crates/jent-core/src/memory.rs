pub struct MemoryLoop<const N: usize> {
    buf: [u8; N],
    cursor: usize,
}

impl<const N: usize> MemoryLoop<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0u8; N],
            cursor: 0,
        }
    }

    #[inline(never)]
    pub fn stir(&mut self) {
        // Placeholder memory access pattern. The real port should mirror the upstream memory loop.
        for i in 0..N {
            let idx = (self.cursor + i.wrapping_mul(37)) % N;
            let v = self.buf[idx];
            self.buf[idx] = v.rotate_left(1).wrapping_add(i as u8).wrapping_add(0xA5);
        }
        self.cursor = self.cursor.wrapping_add(1) % N;
        core::hint::black_box(&self.buf);
    }
}

impl<const N: usize> Default for MemoryLoop<N> {
    fn default() -> Self {
        Self::new()
    }
}
