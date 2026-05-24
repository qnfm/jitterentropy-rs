#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HealthFailure(pub u32);

impl HealthFailure {
    pub const NONE: Self = Self(0);
    pub const RCT: Self = Self(1 << 0);
    pub const APT: Self = Self(1 << 1);
    pub const LAG: Self = Self(1 << 2);
    pub const RCT_MEMORY: Self = Self(1 << 3);
    pub const PERMANENT_SHIFT: u32 = 16;

    pub const fn bits(self) -> u32 {
        self.0
    }
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
    pub const fn permanent(self) -> Self {
        Self(self.0 << Self::PERMANENT_SHIFT)
    }
}

impl core::ops::BitOr for HealthFailure {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for HealthFailure {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

const RCT_CUTOFF: u32 = 31;
const RCT_PERMANENT_CUTOFF: u32 = 2 * RCT_CUTOFF;
const APT_WINDOW: usize = 512;
const APT_CUTOFF: usize = 325;
const APT_PERMANENT_CUTOFF: usize = 450;
const LAG_CUTOFF: u32 = 32;

#[derive(Clone, Debug)]
pub struct HealthState {
    last_delta: Option<u64>,
    last_delta2_abs: Option<u64>,
    rct_count: u32,
    lag_count: u32,
    apt: [u8; APT_WINDOW],
    apt_pos: usize,
    apt_filled: bool,
    last_failure: HealthFailure,
}

impl Default for HealthState {
    fn default() -> Self {
        Self {
            last_delta: None,
            last_delta2_abs: None,
            rct_count: 0,
            lag_count: 0,
            apt: [0; APT_WINDOW],
            apt_pos: 0,
            apt_filled: false,
            last_failure: HealthFailure::NONE,
        }
    }
}

impl HealthState {
    pub fn observe_delta(&mut self, delta: u64) -> HealthFailure {
        let mut failure = HealthFailure::NONE;

        if self.last_delta == Some(delta) {
            self.rct_count = self.rct_count.saturating_add(1);
            if self.rct_count > RCT_CUTOFF {
                failure |= HealthFailure::RCT;
            }
            if self.rct_count > RCT_PERMANENT_CUTOFF {
                failure |= HealthFailure::RCT.permanent();
            }
        } else {
            self.rct_count = 0;
        }

        if let Some(prev) = self.last_delta {
            let delta2_abs = prev.abs_diff(delta);
            if self.last_delta2_abs == Some(delta2_abs) {
                self.lag_count = self.lag_count.saturating_add(1);
                if self.lag_count > LAG_CUTOFF {
                    failure |= HealthFailure::LAG;
                }
                if self.lag_count > 2 * LAG_CUTOFF {
                    failure |= HealthFailure::LAG.permanent();
                }
            } else {
                self.lag_count = 0;
            }
            self.last_delta2_abs = Some(delta2_abs);
        }

        let bit = (delta & 1) as u8;
        self.apt[self.apt_pos] = bit;
        self.apt_pos += 1;
        if self.apt_pos == APT_WINDOW {
            self.apt_pos = 0;
            self.apt_filled = true;
        }
        if self.apt_filled {
            let ones = self.apt.iter().fold(0usize, |acc, &b| acc + b as usize);
            let zeros = APT_WINDOW - ones;
            let max = core::cmp::max(ones, zeros);
            if max > APT_CUTOFF {
                failure |= HealthFailure::APT;
            }
            if max > APT_PERMANENT_CUTOFF {
                failure |= HealthFailure::APT.permanent();
            }
        }

        self.last_delta = Some(delta);
        self.last_failure = failure;
        failure
    }

    pub fn last_failure(&self) -> HealthFailure {
        self.last_failure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rct_fires_on_repetition() {
        let mut h = HealthState::default();
        let mut f = HealthFailure::NONE;
        for _ in 0..40 {
            f = h.observe_delta(7);
        }
        assert!(f.contains(HealthFailure::RCT));
    }
}
