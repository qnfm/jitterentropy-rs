#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]
#![forbid(clippy::unwrap_used)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod collector;
pub mod conditioner;
pub mod error;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod flags;
pub mod health;
pub mod memory;
pub mod status;
pub mod timer;

pub use collector::{EntropyCollector, EntropyCollectorBuilder};
pub use error::{InitError, ReadError};
pub use flags::{Flags, HashLoop, MemoryLimit};
pub use status::Status;
pub use timer::{CallbackTimer, PlatformTimer, Timer};

pub const JENT_MAJVERSION: u32 = 3;
pub const JENT_MINVERSION: u32 = 7;
pub const JENT_PATCHLEVEL: u32 = 1;
pub const JENT_VERSION: u32 =
    JENT_MAJVERSION * 1_000_000 + JENT_MINVERSION * 10_000 + JENT_PATCHLEVEL * 100;

pub fn version() -> u32 {
    JENT_VERSION
}
