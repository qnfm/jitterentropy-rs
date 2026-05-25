#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};
use core::hint::black_box;
use zeroize::Zeroize;

use crate::conditioner::{Conditioner, Shake256Conditioner, OUTPUT_BLOCK_BYTES};
use crate::error::{InitError, ReadError};
use crate::flags::Flags;
#[cfg(feature = "alloc")]
use crate::flags::MemoryLimit;
use crate::health::{HealthFailure, HealthState};
#[cfg(feature = "alloc")]
use crate::memory::MemoryNoise;
use crate::status::Status;
use crate::timer::{CallbackTimer, PlatformTimer, Timer};
use crate::JENT_VERSION;

const MIN_OSR: u32 = 1;
const MAX_OSR: u32 = 128;
const POWERUP_TESTS: usize = 1024;
#[cfg(feature = "alloc")]
const DEFAULT_MEMORY_SIZE: usize = 512 * 1024;
#[cfg(feature = "alloc")]
const DEFAULT_MEMORY_ROUNDS: usize = 128;
const NTG1_PRESEED_SAMPLES: usize = 240;

#[derive(Clone, Debug)]
pub struct EntropyCollectorBuilder {
    osr: u32,
    flags: Flags,
    memory_size: Option<usize>,
}

impl Default for EntropyCollectorBuilder {
    fn default() -> Self {
        Self {
            osr: MIN_OSR,
            flags: Flags::empty(),
            memory_size: None,
        }
    }
}

impl EntropyCollectorBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn osr(mut self, osr: u32) -> Self {
        self.osr = osr.max(MIN_OSR);
        self
    }
    pub fn flags(mut self, flags: Flags) -> Self {
        self.flags = flags;
        self
    }
    pub fn memory_size(mut self, memory_size: usize) -> Self {
        self.memory_size = Some(memory_size);
        self
    }

    pub fn build(self) -> Result<EntropyCollector<PlatformTimer>, InitError> {
        EntropyCollector::with_timer(self.osr, self.flags, self.memory_size, PlatformTimer)
    }

    pub fn build_with_timer<T: Timer>(self, timer: T) -> Result<EntropyCollector<T>, InitError> {
        EntropyCollector::with_timer(self.osr, self.flags, self.memory_size, timer)
    }

    pub fn build_with_callback<F: FnMut() -> u64>(
        self,
        f: F,
    ) -> Result<EntropyCollector<CallbackTimer<F>>, InitError> {
        self.build_with_timer(CallbackTimer::new(f))
    }
}

pub struct EntropyCollector<T: Timer = PlatformTimer> {
    osr: u32,
    flags: Flags,
    timer: T,
    #[cfg(feature = "alloc")]
    memory: Option<MemoryNoise>,
    health: HealthState,
    last_time: u64,
    work_state: u64,
    permanent_failure: HealthFailure,
}

impl EntropyCollector<PlatformTimer> {
    pub fn new(osr: u32, flags: Flags) -> Result<Self, InitError> {
        Self::with_timer(osr, flags, None, PlatformTimer)
    }
}

impl<T: Timer> EntropyCollector<T> {
    pub fn with_timer(
        osr: u32,
        flags: Flags,
        memory_size: Option<usize>,
        mut timer: T,
    ) -> Result<Self, InitError> {
        #[cfg(not(feature = "alloc"))]
        let _ = memory_size;
        if flags.contains(Flags::FORCE_INTERNAL_TIMER)
            && flags.contains(Flags::DISABLE_INTERNAL_TIMER)
        {
            return Err(InitError::ProgrammingError);
        }
        let osr = osr.max(MIN_OSR);
        if osr > MAX_OSR {
            return Err(InitError::ProgrammingError);
        }

        let first = timer.now().ok_or(InitError::TimerUnavailable)?;

        #[cfg(feature = "alloc")]
        let memory = if flags.contains(Flags::DISABLE_MEMORY_ACCESS) {
            None
        } else {
            let size = memory_size.unwrap_or_else(|| configured_memory_size(flags));
            MemoryNoise::new(size).ok_or(InitError::Memory).map(Some)?
        };

        let mut ec = Self {
            osr,
            flags,
            timer,
            #[cfg(feature = "alloc")]
            memory,
            health: HealthState::default(),
            last_time: first,
            work_state: first ^ 0x9e37_79b9_7f4a_7c15,
            permanent_failure: HealthFailure::NONE,
        };

        ec.startup_test()?;
        if flags.contains(Flags::NTG1) {
            ec.ntg1_preshape()?;
        }
        Ok(ec)
    }

    pub fn status(&self) -> Status {
        Status::new(
            self.osr,
            self.flags,
            self.memory_len(),
            self.health.last_failure().bits() | self.permanent_failure.bits(),
        )
    }

    pub fn version(&self) -> u32 {
        JENT_VERSION
    }

    pub fn fill_bytes(&mut self, out: &mut [u8]) -> Result<(), ReadError> {
        if !self.permanent_failure.is_empty() {
            return Err(Self::map_failure(self.permanent_failure));
        }
        let mut written = 0;
        while written < out.len() {
            let mut block = self.block()?;
            let n = core::cmp::min(block.len(), out.len() - written);
            out[written..written + n].copy_from_slice(&block[..n]);
            block.zeroize();
            written += n;
        }
        Ok(())
    }

    #[cfg(feature = "alloc")]
    pub fn read_entropy(&mut self, len: usize) -> Result<Vec<u8>, ReadError> {
        let mut out = vec![0u8; len];
        self.fill_bytes(&mut out)?;
        Ok(out)
    }

    #[inline(never)]
    fn block(&mut self) -> Result<[u8; OUTPUT_BLOCK_BYTES], ReadError> {
        let mut c = Shake256Conditioner::default();
        c.absorb(b"jitterentropy-rs/v1");
        c.absorb(&self.osr.to_le_bytes());
        c.absorb(&self.flags.bits().to_le_bytes());

        for _ in 0..self.osr {
            let sample = self.sample()?;
            for _ in 0..self.flags.hash_loop().count() {
                c.absorb(&sample.to_le_bytes());
                #[cfg(feature = "alloc")]
                if let Some(mem) = self.memory.as_mut() {
                    mem.disturb(sample, DEFAULT_MEMORY_ROUNDS, |b| c.absorb(b));
                }
                self.variable_work(sample);
            }
        }

        Ok(c.finalize_block())
    }

    #[inline(never)]
    fn sample(&mut self) -> Result<u64, ReadError> {
        let before = self.timer.now().ok_or(ReadError::TimerUnavailable)?;
        self.variable_work(before ^ self.last_time);
        let after = self.timer.now().ok_or(ReadError::TimerUnavailable)?;
        if after < before {
            return Err(ReadError::TimerUnavailable);
        }
        let delta = after.wrapping_sub(before);
        let delta_prev = after.wrapping_sub(self.last_time);
        self.last_time = after;

        let folded = delta ^ delta_prev.rotate_left(17) ^ after.rotate_right(9);
        if folded == 0 {
            return Err(ReadError::TimerUnavailable);
        }
        let failure = self.health.observe_delta(folded);
        if !failure.is_empty() {
            if failure.bits() & (0xffff << HealthFailure::PERMANENT_SHIFT) != 0 {
                self.permanent_failure |= failure;
            }
            if self.fips_enabled() {
                return Err(Self::map_failure(failure));
            }
        }
        Ok(folded)
    }

    #[inline(never)]
    fn variable_work(&mut self, seed: u64) {
        let mut x = seed ^ self.work_state | 1;
        let rounds = ((x as usize) & 0x7f) + 64;
        for i in 0..rounds {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            black_box(x.wrapping_add(i as u64));
        }
        self.work_state = self.work_state.rotate_left(7) ^ x.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }

    fn startup_test(&mut self) -> Result<(), InitError> {
        let mut prev = self.timer.now().ok_or(InitError::TimerUnavailable)?;
        let mut nonzero = 0usize;
        let mut variations = 0usize;
        let mut var_of_var = 0usize;
        let mut last_delta = None;
        let mut last_delta2_abs = None;
        let mut gcd = 0u64;

        for _ in 0..POWERUP_TESTS {
            self.variable_work(prev);
            let now = self.timer.now().ok_or(InitError::TimerUnavailable)?;
            if now < prev {
                return Err(InitError::TimerNotMonotonic);
            }
            let delta = now - prev;
            if delta != 0 {
                nonzero += 1;
                gcd = gcd_u64(gcd, delta);
            }
            if last_delta != Some(delta) {
                variations += 1;
            }
            if let Some(ld) = last_delta {
                let d2 = ld.abs_diff(delta);
                if last_delta2_abs != Some(d2) {
                    var_of_var += 1;
                }
                last_delta2_abs = Some(d2);
            }
            let failure = self.health.observe_delta(delta);
            if !failure.is_empty() && self.fips_enabled() {
                return Err(InitError::Health);
            }
            last_delta = Some(delta);
            prev = now;
        }

        if nonzero == 0 {
            return Err(InitError::TimerTooCoarse);
        }
        // Treat a timer that effectively advances in a single large quantum as unsuitable.
        // The GCD check is intentionally conservative; exact entropy claims require target
        // measurements, not this startup guard alone.
        if gcd > 1 && variations <= 2 {
            return Err(InitError::Gcd);
        }
        if variations < 2 {
            return Err(InitError::MinimalVariation);
        }
        if var_of_var == 0 {
            return Err(InitError::VariationOfVariationMissing);
        }
        if var_of_var < 2 {
            return Err(InitError::MinimalVariationOfVariation);
        }
        self.last_time = prev;
        Ok(())
    }

    fn ntg1_preshape(&mut self) -> Result<(), InitError> {
        let mut acc = 0u64;
        for _ in 0..NTG1_PRESEED_SAMPLES {
            acc ^= self.sample().map_err(|_| InitError::Health)?;
        }
        core::hint::black_box(acc);
        #[cfg(feature = "alloc")]
        if let Some(mem) = self.memory.as_mut() {
            fn ignore_bytes(_: &[u8]) {}
            mem.disturb(acc, NTG1_PRESEED_SAMPLES, ignore_bytes);
        }
        Ok(())
    }

    fn fips_enabled(&self) -> bool {
        self.flags.contains(Flags::FORCE_FIPS)
            || self.flags.contains(Flags::NTG1)
            || cfg!(feature = "force-fips")
    }

    fn memory_len(&self) -> usize {
        #[cfg(feature = "alloc")]
        {
            self.memory.as_ref().map(|m| m.len()).unwrap_or(0)
        }
        #[cfg(not(feature = "alloc"))]
        {
            0
        }
    }

    fn map_failure(failure: HealthFailure) -> ReadError {
        if failure.contains(HealthFailure::RCT.permanent()) {
            return ReadError::RctPermanent;
        }
        if failure.contains(HealthFailure::APT.permanent()) {
            return ReadError::AptPermanent;
        }
        if failure.contains(HealthFailure::LAG.permanent()) {
            return ReadError::LagPermanent;
        }
        if failure.contains(HealthFailure::RCT_MEMORY.permanent()) {
            return ReadError::RctMemoryPermanent;
        }
        if failure.contains(HealthFailure::RCT) {
            return ReadError::Rct;
        }
        if failure.contains(HealthFailure::APT) {
            return ReadError::Apt;
        }
        if failure.contains(HealthFailure::LAG) {
            return ReadError::Lag;
        }
        if failure.contains(HealthFailure::RCT_MEMORY) {
            return ReadError::RctMemory;
        }
        ReadError::TimerUnavailable
    }
}

#[cfg(feature = "alloc")]
fn configured_memory_size(flags: Flags) -> usize {
    match flags.memory_limit() {
        MemoryLimit::Auto => DEFAULT_MEMORY_SIZE,
        MemoryLimit::KiB(kib) => {
            let bytes = (kib as usize).saturating_mul(1024);
            bytes.clamp(4096, 64 * 1024 * 1024).next_power_of_two()
        }
    }
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct CounterTimer {
        t: u64,
        state: u64,
    }

    impl CounterTimer {
        fn new() -> Self {
            Self {
                t: 0,
                state: 0x1234_5678_9abc_def0,
            }
        }
    }

    impl Timer for CounterTimer {
        fn now(&mut self) -> Option<u64> {
            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;
            self.t = self.t.wrapping_add(37 + (self.state & 0x1f));
            Some(self.t)
        }
    }

    #[test]
    fn collector_builds_with_counter_timer() {
        let mut ec = EntropyCollector::with_timer(
            1,
            Flags::DISABLE_MEMORY_ACCESS,
            None,
            CounterTimer::new(),
        )
        .unwrap();
        let mut out = [0u8; 32];
        ec.fill_bytes(&mut out).unwrap();
        assert!(out.iter().any(|&b| b != 0));
    }
}
