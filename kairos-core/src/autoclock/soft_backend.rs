// kairos-core/src/autoclock/soft_backend.rs

use core::sync::atomic::{AtomicU32, Ordering};
use crate::{Clock, VInstant, VDuration};

// Global counter in ns (portable)
static NS: AtomicU32 = AtomicU32::new(0);

#[inline(always)]
pub(super) fn add_ns(ns: u32) {
    NS.fetch_add(ns, Ordering::Relaxed);
}

#[inline(always)]
fn load_ns() -> u32 {
    NS.load(Ordering::Relaxed)
}

#[inline(always)]
pub(super) fn reset_ns() { NS.store(0, Ordering::Relaxed); }

#[inline(always)]
pub(super) fn set_ns(ns: u32) { NS.store(ns, Ordering::Relaxed); }

/// Portable backend without `std`: read = atomic load; advance = atomic add.
#[cfg(feature = "autoclock-soft")]
pub struct SoftClock;

#[cfg(feature = "autoclock-soft")]
impl SoftClock {
    #[inline(always)]
    pub fn new() -> Self { Self }

    /// Useful helpers for tests/benchmarks
    #[inline(always)]
    pub(super) fn tick_ns(ns: u32) { add_ns(ns); }

    #[inline(always)]
    pub(super) fn tick_ms(ms: u32) { add_ns(ms.saturating_mul(1_000_000)); }
}

#[cfg(feature = "autoclock-soft")]
impl Clock for SoftClock {
    #[inline(always)]
    fn now(&self) -> VInstant {
        VInstant(load_ns() as u64)
    }

    #[inline(always)]
    fn advance(&mut self, _by: VDuration) {
        let _ = _by; // no-op by default
    }
}
