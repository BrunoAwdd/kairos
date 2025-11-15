// kairos-core/src/clock/mod.rs

use crate::{VInstant, VDuration};

/// Source of monotonic *virtual* time.
pub trait Clock {
    /// Current virtual timestamp.
    fn now(&self) -> VInstant;
    /// Advance virtual time by `by`.
    fn advance(&mut self, by: VDuration);
}

pub mod manual;
pub mod rate;
pub mod std;

pub use manual::ManualClock;
pub use rate::RateClock;
#[cfg(feature = "std")]
pub use self::std::StdClock;