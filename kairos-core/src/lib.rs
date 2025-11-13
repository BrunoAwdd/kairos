//! Kairos Core — a deterministic, portable virtual clock (`no_std` by default).
//!
//! # Quick start (soft backend)
//! ```
//! use kairos_core::{AutoClock, Clock, VDuration};
//!
//! let mut clk = AutoClock::new();
//! // advance virtual time by 2 ms
//! clk.advance(VDuration::from_millis(2));
//! assert!(clk.now().0 >= 2_000_000);
//! ```
//!
//! Enable one backend via Cargo features:
//! - `autoclock-soft` (default): portable, `no_std`, atomic counter
//! - `autoclock-std`: uses `std::time::Instant`
//! - `autoclock-systick`: Cortex-M SysTick (bare-metal)

#![cfg_attr(not(feature = "std"), no_std)]

// If you need heap types in `no_std`, enable the "alloc" feature.
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// ── Modules ──────────────────────────────────────────────────────────────────
// (Gate modules with `#[cfg(feature = "std")]` if they require std)
pub mod autoclock;
pub mod time;
pub mod clock;
pub mod calendar;

// ── High-level re-exports ────────────────────────────────────────────────────
pub use autoclock::AutoClock;
pub use time::{VInstant, VDuration};
pub use clock::{Clock, ManualClock, RateClock, StdClock, KairosHlc, KairosTs, KairosTs16};
pub use calendar::Calendar;

// Expose which backend was compiled, handy for debugging/benches.
#[doc(hidden)]
pub use autoclock::WHICH_BACKEND;

// Allow tests/doctests that need std.
#[cfg(test)]
extern crate std;
