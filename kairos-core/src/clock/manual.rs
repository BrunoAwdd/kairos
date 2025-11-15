// kairos-core/src/clock/manual.rs

use crate::{Clock, VInstant, VDuration};

/// Manually-driven virtual clock (deterministic).
#[derive(Debug, Default, Clone)]
pub struct ManualClock { now: VInstant }
impl ManualClock {
    #[inline(always)]
    pub fn new() -> Self { Self { now: VInstant(0) } }

    #[cfg(feature = "bench-guards")]
    #[inline(never)]
    pub fn now_volatile(&self) -> VInstant {
        unsafe { core::ptr::read_volatile(&self.now) }
    }

    #[inline(never)]
    pub fn now_strict(&self) -> VInstant { self.now }
}
impl Clock for ManualClock {
    #[inline(never)]
    fn now(&self) -> VInstant { self.now }
    #[inline(never)]
    fn advance(&mut self, by: VDuration) { self.now += by; }
}
