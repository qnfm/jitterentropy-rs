#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Flags(pub u32);

impl Flags {
    pub const DISABLE_STIR: Self = Self(1 << 0);
    pub const DISABLE_UNBIAS: Self = Self(1 << 1);
    pub const DISABLE_MEMORY_ACCESS: Self = Self(1 << 2);
    pub const FORCE_INTERNAL_TIMER: Self = Self(1 << 3);
    pub const DISABLE_INTERNAL_TIMER: Self = Self(1 << 4);
    pub const FORCE_FIPS: Self = Self(1 << 5);
    pub const NTG1: Self = Self(1 << 6);
    pub const CACHE_ALL: Self = Self(1 << 7);

    pub const MEMSIZE_SHIFT: u32 = 27;
    pub const HASHLOOP_SHIFT: u32 = 24;
    pub const HASHLOOP_MASK: u32 = 0x7 << Self::HASHLOOP_SHIFT;
    pub const MEMSIZE_MASK: u32 = 0xffff_ffffu32.wrapping_shl(Self::MEMSIZE_SHIFT);

    pub const fn empty() -> Self {
        Self(0)
    }
    pub const fn bits(self) -> u32 {
        self.0
    }
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn hash_loop(self) -> HashLoop {
        match (self.0 >> Self::HASHLOOP_SHIFT) & 0x7 {
            0 => HashLoop::X1,
            1 => HashLoop::X2,
            2 => HashLoop::X4,
            3 => HashLoop::X8,
            4 => HashLoop::X16,
            5 => HashLoop::X32,
            6 => HashLoop::X64,
            _ => HashLoop::X128,
        }
    }

    pub const fn memory_limit(self) -> MemoryLimit {
        let raw = self.0 >> Self::MEMSIZE_SHIFT;
        MemoryLimit::from_raw(raw)
    }
}

impl core::ops::BitOr for Flags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for Flags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HashLoop {
    X1,
    X2,
    X4,
    X8,
    X16,
    X32,
    X64,
    X128,
}

impl HashLoop {
    pub const fn count(self) -> usize {
        match self {
            Self::X1 => 1,
            Self::X2 => 2,
            Self::X4 => 4,
            Self::X8 => 8,
            Self::X16 => 16,
            Self::X32 => 32,
            Self::X64 => 64,
            Self::X128 => 128,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryLimit {
    Auto,
    KiB(u32),
}

impl MemoryLimit {
    pub const fn from_raw(raw: u32) -> Self {
        if raw == 0 {
            Self::Auto
        } else {
            // Encode as KiB and clamp instead of allowing debug-build shift panics.
            // The high bits are controlled by the C-style flags word, so invalid
            // encodings must be handled defensively.
            let shift = raw + 9;
            if shift >= 31 {
                Self::KiB(1 << 30)
            } else {
                Self::KiB(1 << shift)
            }
        }
    }
}
