use jent_core::HighResTimer;

#[derive(Debug, Clone, Copy, Default)]
pub struct RdtscTimer;

#[cfg(target_arch = "x86_64")]
impl HighResTimer for RdtscTimer {
    #[inline(always)]
    fn now(&self) -> u64 {
        // SAFETY: `_rdtsc` reads the processor timestamp counter. The caller must validate
        // that this is an appropriate high-resolution timer for the target environment.
        unsafe { core::arch::x86_64::_rdtsc() }
    }
}

#[cfg(target_arch = "x86")]
impl HighResTimer for RdtscTimer {
    #[inline(always)]
    fn now(&self) -> u64 {
        // SAFETY: `_rdtsc` reads the processor timestamp counter. The caller must validate
        // that this is an appropriate high-resolution timer for the target environment.
        unsafe { core::arch::x86::_rdtsc() as u64 }
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
impl HighResTimer for RdtscTimer {
    fn now(&self) -> u64 {
        0
    }
}
