use crate::JentError;

#[derive(Debug, Clone)]
pub struct HealthState {
    last_delta: u64,
    stuck_count: u32,
    repeated_delta_limit: u32,
}

impl HealthState {
    pub const fn new() -> Self {
        Self {
            last_delta: 0,
            stuck_count: 0,
            repeated_delta_limit: 32,
        }
    }

    pub fn observe_delta(&mut self, delta: u64) -> Result<(), JentError> {
        if delta == 0 {
            return Err(JentError::TimerTooCoarse);
        }

        if delta == self.last_delta {
            self.stuck_count = self.stuck_count.saturating_add(1);
        } else {
            self.stuck_count = 0;
        }

        self.last_delta = delta;

        if self.stuck_count > self.repeated_delta_limit {
            return Err(JentError::HealthTestFailed);
        }

        Ok(())
    }
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}
