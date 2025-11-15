#![no_std]
#![no_main]

use kairos_core::{ManualClock, Clock, VDuration};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut c = ManualClock::new();
    c.advance(VDuration::from_secs(1));
    let _t = c.now(); // só pra usar algo
    loop {} // sem runner, só precisa compilar/linkar
}

#[no_mangle]
pub extern "C" fn kairos_smoke() -> u64 {
    let mut c = ManualClock::new();
    c.advance(VDuration::from_secs(1));
    c.now().as_nanos() // 1_000_000_000 esperado
}
