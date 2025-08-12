use kairos_core::{Clock, VInstant};
use kairos_core::autoclock::AutoClock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let clk = AutoClock::new();

    // T0s
    let VInstant(auto_ns0) = clk.now();
    let sys0 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let inst0 = Instant::now();

    sleep(Duration::from_millis(120));

    // T1s
    let VInstant(auto_ns1) = clk.now();
    let sys1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let inst1 = Instant::now();

    // Deltas
    let auto_ms = (auto_ns1 - auto_ns0) as f64 / 1e6;
    let sys_ms  = (sys1 - sys0).as_secs_f64() * 1e3;
    let inst_ms = (inst1 - inst0).as_secs_f64() * 1e3;

    println!("Δ AutoClock:  {:.3} ms", auto_ms);
    println!("Δ SystemTime: {:.3} ms", sys_ms);
    println!("Δ Instant:    {:.3} ms", inst_ms);
}
