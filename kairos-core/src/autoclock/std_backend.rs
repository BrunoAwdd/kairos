// kairos-core/src/autoclock/std_backend.rs

use crate::{Clock, VInstant, VDuration};

#[cfg(feature = "autoclock-std")]
pub(super) struct StdClock { start: std::time::Instant }

#[cfg(feature = "autoclock-std")]
impl StdClock {
    #[inline(always)]
    pub(super) fn new() -> Self { Self { start: std::time::Instant::now() } }
}

#[cfg(feature = "autoclock-std")]
impl Clock for StdClock {
    #[inline(always)]
    fn now(&self) -> VInstant {
        VInstant(0) + VDuration::from(self.start.elapsed()) 
    }
    #[inline(always)]
    fn advance(&mut self, _by: VDuration) {}
}
