use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum InitError {
    TimerUnavailable = 1,
    TimerTooCoarse = 2,
    TimerNotMonotonic = 3,
    MinimalVariation = 4,
    VariationOfVariationMissing = 5,
    MinimalVariationOfVariation = 6,
    ProgrammingError = 7,
    Stuck = 8,
    Health = 9,
    Rct = 10,
    Hash = 11,
    Memory = 12,
    Gcd = 13,
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InitError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ReadError {
    NullCollector = -1,
    Rct = -2,
    Apt = -3,
    TimerUnavailable = -4,
    Lag = -5,
    RctPermanent = -6,
    AptPermanent = -7,
    LagPermanent = -8,
    RctMemory = -9,
    RctMemoryPermanent = -10,
    OutputTooLarge = -11,
}

impl ReadError {
    pub fn c_code(self) -> isize {
        self as isize
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ReadError {}
