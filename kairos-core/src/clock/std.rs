// kairos-core/src/clock/std.rs

use crate::{Clock, VInstant, VDuration};

/// Real-time backed clock using `std::time::Instant` (host environments).
#[cfg(feature = "std")]
pub struct StdClock {
    start_instant: std::time::Instant,
    start_virtual: VInstant,
}

#[cfg(feature = "std")]
impl StdClock {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            start_instant: std::time::Instant::now(),
            start_virtual: VInstant(0),
        }
    }
}

#[cfg(feature = "std")]
impl Clock for StdClock {
    #[inline(always)]
    fn now(&self) -> VInstant {
        self.start_virtual + VDuration::from(self.start_instant.elapsed())
    }
    #[inline(always)]
    fn advance(&mut self, _by: VDuration) { /* no-op */ }
}
