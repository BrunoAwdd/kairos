// autoclock.rs — zero deps, zero Cargo features

use crate::{Clock, VInstant, VDuration};

// Conditional compilation for backends
#[cfg(feature = "autoclock-std")]
#[path = "autoclock/std_backend.rs"]
mod std_backend;

#[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
#[path = "autoclock/systick_backend.rs"]
mod systick_backend;

#[cfg(feature = "autoclock-soft")]
#[path = "autoclock/soft_backend.rs"]
mod soft_backend;

/// Automatic clock with no external dependencies.
/// - With `std`: uses `std::time::Instant` (zero-config)
/// - Without `std` on Cortex-M: uses SysTick in polling mode (1 line: SYSCLK_HZ)
pub struct AutoClock(Backend);

#[cfg(feature = "autoclock-soft")]
pub use soft_backend::SoftClock;

#[doc(hidden)]
pub const WHICH_BACKEND: &str = {
    #[cfg(feature = "autoclock-soft")] { "soft" }
    #[cfg(all(not(feature="autoclock-soft"), feature="autoclock-std"))] { "std" }
    #[cfg(all(not(feature="autoclock-soft"), not(feature="autoclock-std"), feature="autoclock-systick"))] { "systick" }
    #[cfg(all(not(feature="autoclock-soft"), not(feature="autoclock-std"), not(feature="autoclock-systick")))] { "none" }
};

#[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
pub fn configure_systick(sysclk_hz: u32, tick_hz: u32) {
    systick_backend::configure_systick(sysclk_hz, tick_hz);
}

enum Backend {
    #[cfg(feature = "autoclock-std")]
    Std(std_backend::StdClock),

    #[cfg(feature = "autoclock-soft")]
    Soft(soft_backend::SoftClock),

    // Cortex-M SysTick polling backend (no_std)
    #[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
    Systick(systick_backend::SystickClock),
}

impl AutoClock {
    #[inline(always)]
    pub fn new() -> Self {
        #[cfg(feature = "autoclock-soft")]
        { return Self(Backend::Soft(soft_backend::SoftClock::new())); }

        #[cfg(feature = "autoclock-std")]
        { return Self(Backend::Std(std_backend::StdClock::new())); }

        #[cfg(feature = "autoclock-systick")]
        { return Self(Backend::Systick(systick_backend::SystickClock::new())); }

        #[cfg(not(any(feature = "autoclock-soft", feature = "autoclock-std", feature = "autoclock-systick")))]
        panic!("Select a backend via features");
    }
}

impl Clock for AutoClock {
    #[inline(always)]
    fn now(&self) -> VInstant {
        match &self.0 {
            #[cfg(feature = "autoclock-std")]
            Backend::Std(c) => c.now(),

            #[cfg(feature = "autoclock-soft")]
            Backend::Soft(c) => c.now(),

            #[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
            Backend::Systick(c) => c.now(),

            #[cfg(not(any(feature = "autoclock-std", feature = "autoclock-soft", feature = "autoclock-systick")))]
            _ => unreachable!("AutoClock has no backend (check your features)"),
        }
    }
    #[inline(always)]
    fn advance(&mut self, by: VDuration) {
        match &mut self.0 {
            #[cfg(feature = "autoclock-soft")]
            Backend::Soft(c) => c.advance(by),

            #[cfg(feature = "autoclock-std")]
            Backend::Std(c) => c.advance(by), // no-op for std backend

            #[cfg(feature = "autoclock-systick")]
            Backend::Systick(c) => c.advance(by), // optional no-op for systick

            #[cfg(not(any(feature = "autoclock-std", feature = "autoclock-soft", feature = "autoclock-systick")))]
            _ => {}
        }
    }
}