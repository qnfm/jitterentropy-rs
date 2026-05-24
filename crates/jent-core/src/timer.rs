pub trait HighResTimer {
    fn now(&self) -> u64;
}

pub struct CallbackTimer<F>
where
    F: Fn() -> u64,
{
    f: F,
}

impl<F> CallbackTimer<F>
where
    F: Fn() -> u64,
{
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> HighResTimer for CallbackTimer<F>
where
    F: Fn() -> u64,
{
    #[inline(always)]
    fn now(&self) -> u64 {
        (self.f)()
    }
}
