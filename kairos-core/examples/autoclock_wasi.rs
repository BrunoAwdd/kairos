// kairos-core/examples/autoclock_wasi.rs
use kairos_core::{Clock, VInstant};
use kairos_core::autoclock::AutoClock;

static mut CLK: Option<AutoClock> = None;

#[no_mangle]
pub extern "C" fn autoclock_init() {
    unsafe { CLK = Some(AutoClock::new()); }
}

#[no_mangle]
#[allow(static_mut_refs)]
pub extern "C" fn autoclock_now_ns() -> u64 {
    unsafe {
        let clk = CLK.as_ref().expect("autoclock_init not called");
        let VInstant(ns) = clk.now();
        ns
    }
}

#[no_mangle]
pub extern "C" fn autoclock_elapsed_ms_since(start_ns: u64) -> u64 {
    let now = autoclock_now_ns();
    (now.saturating_sub(start_ns)) / 1_000_000
}

// ✅ evita trap no _start: há um main válido
fn main() {}