use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake256,
};

pub const OUTPUT_BLOCK_BYTES: usize = 64;

pub trait Conditioner {
    fn absorb(&mut self, bytes: &[u8]);
    fn finalize_block(self) -> [u8; OUTPUT_BLOCK_BYTES];
}

#[derive(Default)]
pub struct Shake256Conditioner {
    state: Shake256,
}

impl Conditioner for Shake256Conditioner {
    #[inline(always)]
    fn absorb(&mut self, bytes: &[u8]) {
        self.state.update(bytes);
    }

    fn finalize_block(self) -> [u8; OUTPUT_BLOCK_BYTES] {
        let mut reader = self.state.finalize_xof();
        let mut out = [0u8; OUTPUT_BLOCK_BYTES];
        reader.read(&mut out);
        out
    }
}
