use core::hint::black_box;

/// High-resolution timer abstraction used by the collector.
///
/// The timer is intentionally stateful: embedded platforms may need to keep
/// MMIO state, calibration state, or callback state behind the timer object.
pub trait Timer {
    fn now(&mut self) -> Option<u64>;
}

#[derive(Clone, Debug, Default)]
pub struct PlatformTimer;

impl Timer for PlatformTimer {
    #[inline(never)]
    fn now(&mut self) -> Option<u64> {
        platform_now()
    }
}

#[inline(never)]
pub fn platform_now() -> Option<u64> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        // SAFETY: the fence and TSC read do not dereference memory. Suitability
        // of TSC as a jitter source is still a per-platform validation matter.
        core::arch::x86_64::_mm_lfence();
        let v = core::arch::x86_64::_rdtsc();
        core::arch::x86_64::_mm_lfence();
        Some(black_box(v))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        // SAFETY: the fence and TSC read do not dereference memory. Suitability
        // of TSC as a jitter source is still a per-platform validation matter.
        core::arch::x86::_mm_lfence();
        let v = core::arch::x86::_rdtsc() as u64;
        core::arch::x86::_mm_lfence();
        return Some(black_box(v));
    }
    #[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
    {
        use std::sync::OnceLock;
        use std::time::Instant;
        static START: OnceLock<Instant> = OnceLock::new();
        return Some(black_box(
            START.get_or_init(Instant::now).elapsed().as_nanos() as u64,
        ));
    }
    #[cfg(not(any(feature = "std", target_arch = "x86", target_arch = "x86_64")))]
    {
        None
    }
}

#[derive(Clone, Copy)]
pub struct CallbackTimer<F: FnMut() -> u64> {
    f: F,
}

impl<F: FnMut() -> u64> CallbackTimer<F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: FnMut() -> u64> Timer for CallbackTimer<F> {
    #[inline(always)]
    fn now(&mut self) -> Option<u64> {
        Some((self.f)())
    }
}
