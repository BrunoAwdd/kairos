// kairos-core/src/hlc.rs

use crate::Clock;
use core::convert::From;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KairosTs {
    pub phys_ns: u64, // nanos since its UTC (or virtual) epoch
    pub log: u32,     // logical counter
    pub node: u32,    // node id (optional, for tiebreaker)
}

// Total ordering: (phys_ns, log, node)
impl Ord for KairosTs { 
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (self.phys_ns, self.log, self.node).cmp(&(other.phys_ns, other.log, other.node))
    }
}
impl PartialOrd for KairosTs { 
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct KairosHlc<C: Clock> {
    clk: C,
    last: KairosTs,
}

impl<C: Clock> KairosHlc<C> {
    pub fn new(clock: C, node: u32) -> Self {
        Self { clk: clock, last: KairosTs { phys_ns: 0, log: 0, node } }
    }

    /// Generates local timestamp (ordered and monotonic)
    pub fn now(&mut self) -> KairosTs {
        let p = self.clk.now().as_nanos();
        let last_p = self.last.phys_ns;
        if p > last_p {
            self.last.phys_ns = p;
            self.last.log = 0;
        } else {
            // doesn't let you go back in time: it gets stuck on the last physical one and goes up logically
            self.last.log = self.last.log.wrapping_add(1);
        }
        self.last
    }

    /// Observes received timestamp (from another node) and maintains monotonicity
    pub fn observe(&mut self, incoming: KairosTs) -> KairosTs {
        let p = self.clk.now().as_nanos();
        let max_p = core::cmp::max(p, incoming.phys_ns);
        let last_p = self.last.phys_ns;

        self.last.phys_ns = core::cmp::max(last_p, max_p);
        if self.last.phys_ns == last_p && self.last.phys_ns == incoming.phys_ns {
            self.last.log = core::cmp::max(self.last.log, incoming.log).wrapping_add(1);
        } else if self.last.phys_ns == last_p {
            self.last.log = self.last.log.wrapping_add(1);
        } else if self.last.phys_ns == incoming.phys_ns {
            self.last.log = incoming.log.wrapping_add(1);
        } else {
            self.last.log = 0;
        }
        self.last.clone()
    }
}
// 16 bytes, aligned:
#[repr(C)]
pub struct KairosTs16 {
    pub phys_ns: u64,
    pub log: u32,
    pub node: u32,
}

impl From<KairosTs> for KairosTs16 {
    #[inline(always)]
    fn from(t: KairosTs) -> Self { Self { phys_ns: t.phys_ns, log: t.log, node: t.node } }
}
impl From<KairosTs16> for KairosTs {
    #[inline(always)]
    fn from(t: KairosTs16) -> Self { Self { phys_ns: t.phys_ns, log: t.log, node: t.node } }
}

impl<C: Clock> KairosHlc<C> {
    pub fn with_persisted(clock: C, node: u32, persisted_phys_ns: u64) -> Self {
        let now_p = clock.now().as_nanos();
        let base = if now_p >= persisted_phys_ns { now_p } else { persisted_phys_ns };
        Self { clk: clock, last: KairosTs { phys_ns: base, log: 0, node } }
    }
    #[inline(always)]
    pub fn last(&self) -> KairosTs { self.last.clone() }
}

#[test]
fn hlc_monotonic_local() {
    use crate::clock::manual::ManualClock;
    let mut hlc = KairosHlc::new(ManualClock::default(), 7);
    let a = hlc.now();
    let b = hlc.now();
    assert!(a < b);
}

#[test]
fn hlc_observe_merges() {
    use crate::clock::manual::ManualClock;
    let mut c1 = KairosHlc::new(ManualClock::default(), 1);
    let mut c2 = KairosHlc::new(ManualClock::default(), 2);

    let t1 = c1.now();
    let _ = c2.observe(t1);
    let t2 = c2.now();
    assert!(t1 < t2);
}

#[test]
fn hlc_no_regress_on_equal_phys() {
    use crate::clock::manual::ManualClock;
    let mut hlc = KairosHlc::new(ManualClock::default(), 1);
    let a = hlc.now();
    // same phys → logical must increase
    let b = hlc.now();
    assert!(b.phys_ns == a.phys_ns && b.log > a.log);
}

#[test]
fn hlc_with_persisted_clamps() {
    use crate::clock::manual::ManualClock;
    let clk = ManualClock::default();
    let hlc = KairosHlc::with_persisted(clk, 9, 1_000);
    assert!(hlc.last().phys_ns >= 1_000);
}
