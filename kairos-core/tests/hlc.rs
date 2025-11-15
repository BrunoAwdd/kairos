use kairos_core::{ManualClock, KairosHlc, KairosTs, Clock, VDuration};

#[test]
fn hlc_observe_simple() {
    let mut clock = ManualClock::default();
    let mut hlc = KairosHlc::new(clock.clone(), 1);
    
    let incoming = KairosTs { phys_ns: VDuration::from_secs(20).0, log: 0, node: 2 };
    clock.advance_secs(20);
    let ts = hlc.observe(incoming);
    assert_eq!(ts.phys_ns, VDuration::from_secs(20).0);
}

#[test]
fn hlc_observe_same_phys_and_last() {
    let mut hlc = KairosHlc::new(ManualClock::default(), 1);
    let mut hlc2 = KairosHlc::new(ManualClock::default(), 2);

    let ts1 = hlc.now();
    let ts2 = hlc2.observe(ts1);

    assert_eq!(ts2.phys_ns, ts1.phys_ns);
    assert_eq!(ts2.log, ts1.log + 1);
}

#[test]
fn hlc_observe_last_greater() {
    let mut clock = ManualClock::default();
    let mut hlc = KairosHlc::new(clock.clone(), 1);
    
    clock.advance_secs(10);
    let mut hlc2 = KairosHlc::new(clock, 2);

    let ts1 = hlc.now();
    let ts2 = hlc2.observe(ts1);
    
    assert_eq!(ts2.phys_ns, hlc2.last().phys_ns);
    assert_eq!(ts2.log, 0);
}

#[test]
fn hlc_observe_incoming_greater() {
    let clock = ManualClock::default();
    let mut hlc = KairosHlc::new(clock.clone(), 1);
    
    let mut clock2 = ManualClock::default();
    clock2.advance_secs(10);
    let mut hlc2 = KairosHlc::new(clock2, 2);

    let ts1 = hlc2.now();
    let ts2 = hlc.observe(ts1);
    
    assert_eq!(ts2.phys_ns, ts1.phys_ns);
    assert_eq!(ts2.log, ts1.log + 1);
}

#[test]
fn hlc_observe_else_case() {
    let mut clock_for_hlc = ManualClock::default();
    clock_for_hlc.advance_secs(30); // hlc's clock is at 30s
    let mut hlc = KairosHlc::new(clock_for_hlc, 1); // hlc.last is {0,0,1}

    let incoming = KairosTs { phys_ns: VDuration::from_secs(10).0, log: 0, node: 2 }; // incoming.phys_ns is 10s

    let ts2 = hlc.observe(incoming);
    
    assert_eq!(ts2.phys_ns, VDuration::from_secs(30).0);
    assert_eq!(ts2.log, 0);
}

#[test]
fn hlc_observe_incoming_phys_eq_last_phys() {
    let mut clock_for_hlc = ManualClock::default();
    clock_for_hlc.advance_secs(10); // hlc's clock is at 10s
    let mut hlc = KairosHlc::new(clock_for_hlc, 1); // hlc.last is {0,0,1}

    // Call now() to set hlc.last.phys_ns to 10s
    let _ = hlc.now(); // hlc.last is now {10s, 0, 1}

    let incoming = KairosTs { phys_ns: VDuration::from_secs(10).0, log: 5, node: 2 }; // incoming.phys_ns is 10s, log 5

    // Now, when observe is called:
    // p = 10s
    // incoming.phys_ns = 10s
    // last_p = 10s
    // max_p = max(10s, 10s) = 10s
    // self.last.phys_ns = max(10s, 10s) = 10s

    // Check conditions:
    // self.last.phys_ns (10s) == last_p (10s) -> TRUE
    // self.last.phys_ns (10s) == incoming.phys_ns (10s) -> TRUE
    // So, the first IF branch should be taken: self.last.log = max(self.last.log, incoming.log).wrapping_add(1)
    // self.last.log = max(0, 5) + 1 = 6

    let ts2 = hlc.observe(incoming);
    
    assert_eq!(ts2.phys_ns, VDuration::from_secs(10).0);
    assert_eq!(ts2.log, 6);
}

trait ManualClockExt {
    fn advance_secs(&mut self, secs: u64);
}

impl ManualClockExt for ManualClock {
    fn advance_secs(&mut self, secs: u64) {
        self.advance(kairos_core::VDuration::from_secs(secs));
    }
}
