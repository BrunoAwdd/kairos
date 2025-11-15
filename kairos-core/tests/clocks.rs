use kairos_core::{RateClock, Clock, VDuration, VInstant, StdClock};

#[test]
fn rate_clock_new() {
    let clock: RateClock = RateClock::new();
    assert_eq!(clock.now(), VInstant(0));
}

#[test]
fn rate_clock_with_rate() {
    let mut clock = RateClock::with_rate(2, 1); // 2x speed
    clock.tick(VDuration::from_secs(1));
    assert_eq!(clock.now(), VInstant::from(VDuration::from_secs(2)));
}

#[test]
fn rate_clock_set_rate() {
    let mut clock: RateClock = RateClock::new();
    clock.set_rate(1, 2); // 0.5x speed
    clock.tick(VDuration::from_secs(2));
    assert_eq!(clock.now(), VInstant::from(VDuration::from_secs(1)));
}

#[test]
fn rate_clock_set_rate_den_zero() {
    let mut clock: RateClock = RateClock::new();
    clock.set_rate(1, 0); // den = 0 should be treated as 1
    clock.tick(VDuration::from_secs(1));
    assert_eq!(clock.now(), VInstant::from(VDuration::from_secs(1)));
}

#[test]
fn rate_clock_advance() {
    let mut clock: RateClock = RateClock::new();
    clock.advance(VDuration::from_secs(10));
    assert_eq!(clock.now(), VInstant::from(VDuration::from_secs(10)));
}

#[test]
#[cfg(feature = "std")]
fn std_clock_now() {
    let clock = StdClock::new();
    let t0 = clock.now();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let t1 = clock.now();
    assert!(t1 > t0);
}

#[test]
#[cfg(feature = "std")]
fn std_clock_advance() {
    let mut clock = StdClock::new();
    let t0 = clock.now();
    clock.advance(VDuration::from_secs(1)); // should be a no-op
    let t1 = clock.now();
    assert!(t1 >= t0);
}

#[test]
fn kairos_ts_from() {
    let ts = kairos_core::KairosTs { phys_ns: 1, log: 2, node: 3 };
    let ts16: kairos_core::KairosTs16 = ts.into();
    assert_eq!(ts16.phys_ns, 1);
    assert_eq!(ts16.log, 2);
    assert_eq!(ts16.node, 3);

    let ts2: kairos_core::KairosTs = ts16.into();
    assert_eq!(ts2, ts);
}

#[test]
#[cfg(feature = "bench-guards")]
fn manual_clock_now_volatile() {
    let clock = kairos_core::ManualClock::new();
    let _ = clock.now_volatile();
}

#[test]
fn manual_clock_now_strict() {
    let clock = kairos_core::ManualClock::new();
    let _ = clock.now_strict();
}
