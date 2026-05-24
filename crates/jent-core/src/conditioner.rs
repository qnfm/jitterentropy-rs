use crate::JentError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionerKind {
    Sha3_512,
    Shake256,
}

pub trait Conditioner {
    fn kind(&self) -> ConditionerKind;
    fn condition(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, JentError>;
}

#[cfg(feature = "sha3-512")]
pub struct Sha3_512Conditioner;

#[cfg(feature = "sha3-512")]
impl Sha3_512Conditioner {
    pub const OUTPUT_LEN: usize = 64;

    pub const fn new() -> Self {
        Self
    }
}

#[cfg(feature = "sha3-512")]
impl Conditioner for Sha3_512Conditioner {
    fn kind(&self) -> ConditionerKind {
        ConditionerKind::Sha3_512
    }

    fn condition(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, JentError> {
        use sha3::{Digest, Sha3_512};

        if output.len() < Self::OUTPUT_LEN {
            return Err(JentError::OutputBufferTooSmall);
        }

        let digest = Sha3_512::digest(input);
        output[..Self::OUTPUT_LEN].copy_from_slice(&digest);
        Ok(Self::OUTPUT_LEN)
    }
}

#[cfg(not(feature = "sha3-512"))]
pub struct Sha3_512Conditioner;

#[cfg(not(feature = "sha3-512"))]
impl Conditioner for Sha3_512Conditioner {
    fn kind(&self) -> ConditionerKind {
        ConditionerKind::Sha3_512
    }

    fn condition(&mut self, _input: &[u8], _output: &mut [u8]) -> Result<usize, JentError> {
        Err(JentError::UnsupportedConditioner)
    }
}

#[cfg(feature = "shake256")]
pub struct Shake256Conditioner {
    output_len: usize,
}

#[cfg(feature = "shake256")]
impl Shake256Conditioner {
    pub const fn new(output_len: usize) -> Self {
        Self { output_len }
    }
}

#[cfg(feature = "shake256")]
impl Conditioner for Shake256Conditioner {
    fn kind(&self) -> ConditionerKind {
        ConditionerKind::Shake256
    }

    fn condition(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, JentError> {
        use sha3::{Shake256, digest::{ExtendableOutput, Update, XofReader}};

        if output.len() < self.output_len {
            return Err(JentError::OutputBufferTooSmall);
        }

        let mut hasher = Shake256::default();
        hasher.update(input);
        let mut reader = hasher.finalize_xof();
        reader.read(&mut output[..self.output_len]);
        Ok(self.output_len)
    }
}

#[cfg(not(feature = "shake256"))]
pub struct Shake256Conditioner;

#[cfg(not(feature = "shake256"))]
impl Conditioner for Shake256Conditioner {
    fn kind(&self) -> ConditionerKind {
        ConditionerKind::Shake256
    }

    fn condition(&mut self, _input: &[u8], _output: &mut [u8]) -> Result<usize, JentError> {
        Err(JentError::UnsupportedConditioner)
    }
}
