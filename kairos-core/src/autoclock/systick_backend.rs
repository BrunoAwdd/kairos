
use core::sync::atomic::{AtomicU32, Ordering};

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
static SYSCLK_HZ: AtomicU32 = AtomicU32::new(48_000_000);
/// Target SysTick frequency (Hz). 1000 => 1 ms per tick.
static TICK_HZ: AtomicU32 = AtomicU32::new(1000);

/// Configures the SysTick clock. This function should be called once at the beginning of the program.
pub fn configure_systick(sysclk_hz: u32, tick_hz: u32) {
    SYSCLK_HZ.store(sysclk_hz, Ordering::Relaxed);
    TICK_HZ.store(tick_hz, Ordering::Relaxed);
}

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
#[inline(always)]
pub(super) fn init_if_needed() {
    let st = state_mut();
    if st.reload != 0 { return; } // already configured

    let sysclk_hz = SYSCLK_HZ.load(Ordering::Relaxed);
    let tick_hz = TICK_HZ.load(Ordering::Relaxed);

    // Calculate reload for TICK_HZ (e.g., 1 ms)
    let reload = (sysclk_hz / tick_hz).saturating_sub(1);
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
pub(super) fn elapsed_ticks() -> u64 {
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
pub(super) fn tick_ns() -> u64 {
    let tick_hz = TICK_HZ.load(Ordering::Relaxed);
    1_000_000_000u64 / (tick_hz as u64)
}

/// The SysTick-based clock
#[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
pub(super) struct SystickClock;

#[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
impl SystickClock {
    #[inline(always)]
    pub(super) fn new() -> Self {
        init_if_needed();
        Self
    }
}

#[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
impl crate::Clock for SystickClock {
    #[inline(always)]
    fn now(&self) -> VInstant {
        let t = elapsed_ticks();
        VInstant(t.saturating_mul(tick_ns()))
    }
    fn advance(&mut self, _by: VDuration) {}
}
