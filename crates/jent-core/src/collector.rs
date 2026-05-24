use crate::{Conditioner, HealthState, HighResTimer, JentError};

pub struct JentCollector<T: HighResTimer> {
    timer: T,
    health: HealthState,
    last_delta: u64,
    last_delta2: u64,
    last_delta3: u64,
    work_rounds: usize,
}

impl<T: HighResTimer> JentCollector<T> {
    pub const fn new(timer: T) -> Self {
        Self {
            timer,
            health: HealthState::new(),
            last_delta: 0,
            last_delta2: 0,
            last_delta3: 0,
            work_rounds: 64,
        }
    }

    pub fn set_work_rounds(&mut self, rounds: usize) {
        self.work_rounds = rounds.max(1);
    }

    #[inline(never)]
    pub fn sample_delta(&mut self) -> Result<u64, JentError> {
        let t1 = self.timer.now();
        let work = self.cpu_jitter_work();
        core::hint::black_box(work);
        let t2 = self.timer.now();

        let delta = t2.wrapping_sub(t1);
        self.health.observe_delta(delta)?;

        self.last_delta3 = self.last_delta2;
        self.last_delta2 = self.last_delta;
        self.last_delta = delta;

        Ok(delta)
    }

    #[inline(never)]
    fn cpu_jitter_work(&mut self) -> u64 {
        let mut x = self.last_delta
            ^ self.last_delta2.rotate_left(17)
            ^ self.last_delta3.rotate_left(31)
            ^ 0x9E37_79B9_7F4A_7C15;

        for i in 0..self.work_rounds {
            x ^= (i as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
            x = x.rotate_left(13);
            x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x ^= x >> 29;
        }

        x
    }

    pub fn raw_block<const N: usize>(&mut self) -> Result<[u8; N], JentError> {
        let mut out = [0u8; N];
        let mut written = 0;

        while written < N {
            let delta = self.sample_delta()?;
            let bytes = delta.to_le_bytes();
            let n = core::cmp::min(8, N - written);
            out[written..written + n].copy_from_slice(&bytes[..n]);
            written += n;
        }

        Ok(out)
    }

    pub fn fill_conditioned<C: Conditioner>(
        &mut self,
        conditioner: &mut C,
        out: &mut [u8],
    ) -> Result<(), JentError> {
        let mut pos = 0;
        let mut conditioned = [0u8; 64];

        while pos < out.len() {
            let raw = self.raw_block::<128>()?;
            let produced = conditioner.condition(&raw, &mut conditioned)?;
            let n = core::cmp::min(produced, out.len() - pos);
            out[pos..pos + n].copy_from_slice(&conditioned[..n]);
            pos += n;
        }

        Ok(())
    }
}
