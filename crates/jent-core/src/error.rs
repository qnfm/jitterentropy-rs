#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JentError {
    TimerTooCoarse,
    HealthTestFailed,
    OutputBufferTooSmall,
    UnsupportedConditioner,
}
