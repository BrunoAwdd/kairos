use std::cmp::Ordering;
use std::collections::BinaryHeap;
use kairos_core::{Clock, VInstant, VDuration};

/// Represents a scheduled event.
/// `at` is the virtual time when the event should fire.
/// `payload` is the associated data to be passed to the event handler.
pub struct Event<T> {
    at: VInstant,
    payload: T,
}

// Priority queue ordering: earliest event = highest priority.
impl<T> PartialEq for Event<T> {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}
impl<T> Eq for Event<T> {}
impl<T> PartialOrd for Event<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse order so the BinaryHeap pops earliest event first.
        Some(other.at.cmp(&self.at))
    }
}
impl<T> Ord for Event<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.at.cmp(&self.at)
    }
}

/// Simple time-driven scheduler that runs tasks at scheduled virtual times.
/// Uses a `Clock` to track and advance time, allowing simulation or real-time modes.
pub struct Scheduler<T, C: Clock> {
    clock: C,
    pq: BinaryHeap<Event<T>>,
}

impl<T, C: Clock> Scheduler<T, C> {
    /// Creates a new scheduler using the given clock.
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            pq: BinaryHeap::new(),
        }
    }

    /// Mutable reference to the underlying clock (for manual advancement).
    pub fn clock_mut(&mut self) -> &mut C {
        &mut self.clock
    }

    /// Returns the current virtual time.
    pub fn now(&self) -> VInstant {
        self.clock.now()
    }

    /// Schedule a new event to occur after a given duration.
    pub fn schedule_in(&mut self, in_dur: VDuration, payload: T) {
        let at = self.clock.now() + in_dur;
        self.pq.push(Event { at, payload });
    }

    /// Runs all events up to a target time.
    /// Advances the clock as events are processed.
    pub fn run_until<F: FnMut(VInstant, T)>(&mut self, target: VInstant, mut on_event: F) {
        while let Some(ev) = self.pq.peek() {
            if ev.at > target {
                break;
            }
            let ev = self.pq.pop().unwrap();
            let delta = ev.at - self.clock.now();
            self.clock.advance(delta);
            on_event(self.clock.now(), ev.payload);
        }
        // Advance to the target time even if no events remain.
        let remaining = target - self.clock.now();
        self.clock.advance(remaining);
    }
}
