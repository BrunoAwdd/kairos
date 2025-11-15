#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use {
    priority_queue::PriorityQueue,
    std::collections::HashMap,
    std::cmp::Reverse,
};
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
use heapless::{BinaryHeap as HeaplessBinaryHeap, FnvIndexSet as HeaplessHashSet, binary_heap::Max};

use kairos_core::{Clock, VInstant, VDuration};
use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct EventId(pub u64);

/// Represents a scheduled event.
/// `at` is the virtual time when the event should fire.
/// `payload` is the associated data to be passed to the event handler.
#[derive(Debug)]
pub struct Event<T> {
    pub id: EventId,
    pub at: VInstant,
    pub payload: T,
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
#[cfg(feature = "std")]
pub struct Scheduler<T, C: Clock> {
    clock: C,
    pq: PriorityQueue<EventId, Reverse<VInstant>>,
    events: HashMap<EventId, T>,
    next_id: u64,
}

#[cfg(feature = "std")]
impl<T, C: Clock> Scheduler<T, C> {
    /// Creates a new scheduler using the given clock.
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            pq: PriorityQueue::new(),
            events: HashMap::new(),
            next_id: 0,
        }
    }

    /// Schedule a new event to occur after a given duration.
    pub fn schedule_in(&mut self, in_dur: VDuration, payload: T) -> Result<EventId, T> {
        let at = self.clock.now() + in_dur;
        let id = EventId(self.next_id);
        self.next_id += 1;
        self.pq.push(id, Reverse(at));
        self.events.insert(id, payload);
        Ok(id)
    }

    /// Cancel a scheduled event.
    pub fn cancel(&mut self, id: EventId) {
        self.pq.remove(&id);
        self.events.remove(&id);
    }

    /// Modify a scheduled event.
    pub fn modify_event(&mut self, id: EventId, new_at: Option<VInstant>, new_payload: Option<T>) -> bool {
        if let Some(mut priority) = self.pq.get_priority(&id).cloned() {
            let mut modified = false;
            if let Some(at) = new_at {
                priority = Reverse(at);
                modified = true;
            }

            if let Some(payload) = new_payload {
                if let Some(event_payload) = self.events.get_mut(&id) {
                    *event_payload = payload;
                }
            }

            if modified {
                self.pq.change_priority(&id, priority);
            }
            true
        } else {
            false
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

    /// Runs all events up to a target time.
    /// Advances the clock as events are processed.
    pub fn run_until<F: FnMut(VInstant, T)>(&mut self, target: VInstant, mut on_event: F) {
        while let Some((_id, at)) = self.pq.peek() {
            if at.0 > target {
                break;
            }
            let (id, at) = self.pq.pop().unwrap();
            let payload = self.events.remove(&id).unwrap();

            let delta = at.0 - self.clock.now();
            self.clock.advance(delta);
            on_event(self.clock.now(), payload);
        }
        // Advance to the target time even if no events remain.
        let remaining = target - self.clock.now();
        self.clock.advance(remaining);
    }
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use {
    alloc::collections::BTreeMap,
    core::cmp::Reverse,
    priority_queue::PriorityQueue,
    ahash,
};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub struct Scheduler<T, C: Clock> {
    clock: C,
    pq: PriorityQueue<EventId, Reverse<VInstant>, ahash::RandomState>,
    events: BTreeMap<EventId, T>,
    next_id: u64,
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl<T, C: Clock> Scheduler<T, C> {
    /// Creates a new scheduler using the given clock.
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            pq: PriorityQueue::with_hasher(ahash::RandomState::new()),
            events: BTreeMap::new(),
            next_id: 0,
        }
    }

    /// Schedule a new event to occur after a given duration.
    pub fn schedule_in(&mut self, in_dur: VDuration, payload: T) -> Result<EventId, T> {
        let at = self.clock.now() + in_dur;
        let id = EventId(self.next_id);
        self.next_id += 1;
        self.pq.push(id, Reverse(at));
        self.events.insert(id, payload);
        Ok(id)
    }

    /// Cancel a scheduled event.
    pub fn cancel(&mut self, id: EventId) {
        self.pq.remove(&id);
        self.events.remove(&id);
    }

    /// Modify a scheduled event.
    pub fn modify_event(&mut self, id: EventId, new_at: Option<VInstant>, new_payload: Option<T>) -> bool {
        if let Some(mut priority) = self.pq.get_priority(&id).cloned() {
            let mut modified = false;
            if let Some(at) = new_at {
                priority = Reverse(at);
                modified = true;
            }

            if let Some(payload) = new_payload {
                if let Some(event_payload) = self.events.get_mut(&id) {
                    *event_payload = payload;
                }
            }

            if modified {
                self.pq.change_priority(&id, priority);
            }
            true
        } else {
            false
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

    /// Runs all events up to a target time.
    /// Advances the clock as events are processed.
    pub fn run_until<F: FnMut(VInstant, T)>(&mut self, target: VInstant, mut on_event: F) {
        while let Some((_id, at)) = self.pq.peek() {
            if at.0 > target {
                break;
            }
            let (id, at) = self.pq.pop().unwrap();
            let payload = self.events.remove(&id).unwrap();

            let delta = at.0 - self.clock.now();
            self.clock.advance(delta);
            on_event(self.clock.now(), payload);
        }
        // Advance to the target time even if no events remain.
        let remaining = target - self.clock.now();
        self.clock.advance(remaining);
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub struct Scheduler<T, C: Clock, const N: usize> {
    clock: C,
    pq: HeaplessBinaryHeap<Event<T>, Max, N>,
    next_id: u64,
    cancelled_ids: HeaplessHashSet<EventId, N>,
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T: core::fmt::Debug, C: Clock, const N: usize> Scheduler<T, C, N> {
    /// Creates a new scheduler using the given clock.
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            pq: HeaplessBinaryHeap::new(),
            next_id: 0,
            cancelled_ids: HeaplessHashSet::new(),
        }
    }

    /// Schedule a new event to occur after a given duration.
    pub fn schedule_in(&mut self, in_dur: VDuration, payload: T) -> Result<EventId, T> {
        let at = self.clock.now() + in_dur;
        let id = EventId(self.next_id);
        self.next_id += 1;
        self.pq.push(Event { id, at, payload }).map(|_| id).map_err(|e| e.payload)
    }

    /// Cancel a scheduled event.
    pub fn cancel(&mut self, id: EventId) {
        self.cancelled_ids.insert(id).unwrap();
    }

    /// Mutable reference to the underlying clock (for manual advancement).
    pub fn clock_mut(&mut self) -> &mut C {
        &mut self.clock
    }

    /// Returns the current virtual time.
    pub fn now(&self) -> VInstant {
        self.clock.now()
    }

    /// Runs all events up to a target time.
    /// Advances the clock as events are processed.
    pub fn run_until<F: FnMut(VInstant, T)>(&mut self, target: VInstant, mut on_event: F) {
        while let Some(ev) = self.pq.peek() {
            if ev.at > target {
                break;
            }
            let ev = self.pq.pop().unwrap();
            if self.cancelled_ids.contains(&ev.id) {
                self.cancelled_ids.remove(&ev.id);
                continue;
            }
            let delta = ev.at - self.clock.now();
            self.clock.advance(delta);
            on_event(self.clock.now(), ev.payload);
        }
        // Advance to the target time even if no events remain.
        let remaining = target - self.clock.now();
        self.clock.advance(remaining);
    }
}
