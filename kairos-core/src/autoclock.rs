// autoclock.rs — zero deps, zero Cargo features

use crate::{Clock, VInstant, VDuration};

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

enum Backend {
    #[cfg(feature = "autoclock-std")]
    Std(StdClock),

    #[cfg(feature = "autoclock-soft")]
    Soft(soft_backend::SoftClock),

    // Cortex-M SysTick polling backend (no_std)
    #[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
    Systick(SystickClock),
}

impl AutoClock {
    #[inline(always)]
    pub fn new() -> Self {
        #[cfg(feature = "autoclock-soft")]
        { return Self(Backend::Soft(soft_backend::SoftClock::new())); }

        #[cfg(feature = "autoclock-std")]
        { return Self(Backend::Std(StdClock::new())); }

        #[cfg(feature = "autoclock-systick")]
        { return Self(Backend::Systick(SystickClock::new())); }

        #[allow(unreachable_code)]
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

            _ => {}
        }
    }
}

/* ------------------------- BACKEND STD (zero-config) ---------------------- */

#[cfg(feature = "autoclock-std")]
struct StdClock { start: std::time::Instant }

#[cfg(feature = "autoclock-std")]
impl StdClock {
    #[inline(always)]
    fn new() -> Self { Self { start: std::time::Instant::now() } }
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

/* --------------- BACKEND SYSTICK POLLING (no_std, no ISR) ----------------- */
/* No crates: direct register access to SysTick                              */
/* Reference: ARMv7-M Architecture Reference Manual                          */

#[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
mod systick_backend {
    use core::cell::UnsafeCell;
    use crate::{Clock, VInstant, VDuration};

    // SysTick registers (ARM)
    const SYST_CSR:  *mut u32 = 0xE000_E010 as *mut u32;
    const SYST_RVR:  *mut u32 = 0xE000_E014 as *mut u32;
    const SYST_CVR:  *mut u32 = 0xE000_E018 as *mut u32;

    // CSR bits
    const CSR_ENABLE:    u32 = 1 << 0;
    const CSR_TICKINT:   u32 = 1 << 1;
    const CSR_CLKSOURCE: u32 = 1 << 2;

    /// ***Single line for the user to edit for accuracy:***
    /// CPU frequency in Hz (e.g., 48 MHz). Default works "OK".
    pub const SYSCLK_HZ: u32 = 48_000_000;
    /// Target SysTick frequency (Hz). 1000 => 1 ms per tick.
    const TICK_HZ: u32 = 1000;

    /// Local state to extend SysTick's 24-bit counter.
    /// We don't use interrupts; polling detects wraparound.
    pub struct SystickClockState {
        reload: u32,         // reload value (N-1)
        last_cvr: u32,       // last CVR read (down-counter)
        ticks_accum: u64,    // accumulated ticks since start
    }

    impl SystickClockState {
        const fn new() -> Self {
            Self { reload: 0, last_cvr: 0, ticks_accum: 0 }
        }
    }

    #[repr(transparent)]
    struct StateCell(UnsafeCell<SystickClockState>);
    unsafe impl Sync for StateCell {}

    // Singleton without dependencies: UnsafeCell + &'static inside.
    static STATE: StateCell = StateCell(UnsafeCell::new(SystickClockState::new()));

    #[inline(always)]
    fn state_mut() -> &'static mut SystickClockState {
        unsafe { &mut *STATE.0.get() }
    }

    /// Initialize SysTick in polling mode (no TICKINT). Idempotent.
    pub fn init_if_needed() {
        let st = state_mut();
        if st.reload != 0 { return; } // already configured

        // Calculate reload for TICK_HZ (e.g., 1 ms)
        let reload = (SYSCLK_HZ / TICK_HZ).saturating_sub(1);
        unsafe {
            core::ptr::write_volatile(SYST_RVR, reload);
            core::ptr::write_volatile(SYST_CVR, 0);
            let mut csr = CSR_CLKSOURCE | CSR_ENABLE;
            core::ptr::write_volatile(SYST_CSR, csr);
        }
        st.reload   = reload;
        st.last_cvr = unsafe { core::ptr::read_volatile(SYST_CVR) };
        st.ticks_accum = 0;
    }

    /// Read number of ticks (1/TICK_HZ) since start (extends 24 bits).
    #[inline(always)]
    pub fn elapsed_ticks() -> u64 {
        let st = state_mut();
        let cur = unsafe { core::ptr::read_volatile(SYST_CVR) };
        let last = st.last_cvr;
        let reload = st.reload + 1;
        let delta = if last >= cur {
            (last - cur) as u64
        } else {
            (last as u64) + (reload as u64) - (cur as u64)
        };
        let ticks = delta / (reload as u64);
        st.ticks_accum = st.ticks_accum.wrapping_add(ticks);
        st.last_cvr = cur;
        st.ticks_accum
    }

    /// Nanoseconds per tick (e.g., 1_000_000 ns for 1 ms)
    #[inline(always)]
    pub const fn tick_ns() -> u64 {
        1_000_000_000u64 / (TICK_HZ as u64)
    }

    /// The SysTick-based clock
    pub struct SystickClock;

    impl SystickClock {
        #[inline(always)]
        pub fn new() -> Self {
            init_if_needed();
            Self
        }
    }

    impl crate::Clock for SystickClock {
        #[inline(always)]
        fn now(&self) -> VInstant {
            let t = elapsed_ticks();
            VInstant(t.saturating_mul(tick_ns()))
        }
        fn advance(&mut self, _by: VDuration) {}
    }

    // Re-export for parent module
    pub use SystickClock as ClockImpl;
}
#[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
use systick_backend::ClockImpl as SystickClock;

/* ---------------------- BACKEND SOFT (no_std, portable) ------------------- */

#[cfg(feature = "autoclock-soft")]
pub mod soft_backend {
    use core::sync::atomic::{AtomicU32, Ordering};
    use crate::{Clock, VInstant, VDuration};

    // Global counter in ns (portable)
    static NS: AtomicU32 = AtomicU32::new(0);

    #[inline(always)]
    pub fn add_ns(ns: u32) {
        NS.fetch_add(ns, Ordering::Relaxed);
    }

    #[inline(always)]
    fn load_ns() -> u32 {
        NS.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn reset_ns() { NS.store(0, Ordering::Relaxed); }

    #[inline(always)]
    pub fn set_ns(ns: u32) { NS.store(ns, Ordering::Relaxed); }

    /// Portable backend without `std`: read = atomic load; advance = atomic add.
    pub struct SoftClock;

    impl SoftClock {
        #[inline(always)]
        pub fn new() -> Self { Self }

        /// Useful helpers for tests/benchmarks
        #[inline(always)]
        pub fn tick_ns(ns: u32) { add_ns(ns); }

        #[inline(always)]
        pub fn tick_ms(ms: u32) { add_ns(ms.saturating_mul(1_000_000)); }
    }

    impl Clock for SoftClock {
        #[inline(always)]
        fn now(&self) -> VInstant {
            VInstant(load_ns() as u64)
        }

        #[inline(always)]
        fn advance(&mut self, by: VDuration) {
            let _ = by; // no-op by default
        }
    }
}
