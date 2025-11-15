use kairos_core::{ManualClock, VDuration, VInstant, Clock};
use kairos_scheduler::{Scheduler, Event, EventId};

#[cfg(any(feature = "std", feature = "alloc"))]
macro_rules! make_scheduler {
    ($payload_type:ty, $clock:expr) => {
        Scheduler::<$payload_type, _>::new($clock)
    };
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
macro_rules! make_scheduler {
    ($payload_type:ty, $clock:expr) => {
        Scheduler::<$payload_type, _, 16>::new($clock)
    };
}

#[test]
fn scheduler_runs_events_in_order() {
    let clock = ManualClock::new();
    let mut scheduler = make_scheduler!(&str, clock);

    let _ = scheduler.schedule_in(VDuration::from_secs(10), "event_10s").unwrap();
    let _ = scheduler.schedule_in(VDuration::from_secs(5), "event_5s").unwrap();
    let _ = scheduler.schedule_in(VDuration::from_secs(15), "event_15s").unwrap();

    let mut processed_events = Vec::new();
    let target_time = scheduler.now() + VDuration::from_secs(20);

    scheduler.run_until(target_time, |time, payload| {
        processed_events.push((time, payload));
    });

    assert_eq!(processed_events.len(), 3);
    assert_eq!(processed_events[0].0.0 / 1_000_000_000, 5);
    assert_eq!(processed_events[0].1, "event_5s");
    assert_eq!(processed_events[1].0.0 / 1_000_000_000, 10);
    assert_eq!(processed_events[1].1, "event_10s");
    assert_eq!(processed_events[2].0.0 / 1_000_000_000, 15);
    assert_eq!(processed_events[2].1, "event_15s");

    assert_eq!(scheduler.now(), target_time);
}

#[test]
fn scheduler_run_until_stops_at_target() {
    let clock = ManualClock::new();
    let mut scheduler = make_scheduler!(&str, clock);

    let _ = scheduler.schedule_in(VDuration::from_secs(5), "event_5s").unwrap();
    let _ = scheduler.schedule_in(VDuration::from_secs(15), "event_15s").unwrap();

    let mut processed_events = Vec::new();
    let target_time = scheduler.now() + VDuration::from_secs(10);

    scheduler.run_until(target_time, |time, payload| {
        processed_events.push((time, payload));
    });

    assert_eq!(processed_events.len(), 1);
    assert_eq!(processed_events[0].0.0 / 1_000_000_000, 5);
    assert_eq!(processed_events[0].1, "event_5s");

    assert_eq!(scheduler.now(), target_time);
}

#[test]
fn scheduler_clock_mut() {
    let clock = ManualClock::new();
    let mut scheduler = make_scheduler!((), clock);
    scheduler.clock_mut().advance(VDuration::from_secs(10));
    assert_eq!(scheduler.now().0 / 1_000_000_000, 10);
}

#[test]
fn event_ordering() {
    let event1 = Event {
        id: EventId(0),
        at: VInstant(100),
        payload: "event1",
    };
    let event2 = Event {
        id: EventId(1),
        at: VInstant(200),
        payload: "event2",
    };
    let event3 = Event {
        id: EventId(2),
        at: VInstant(100),
        payload: "event3",
    };

    assert!(event1 > event2);
    assert!(event2 < event1);
    assert_eq!(event1, event3);
}

#[test]
fn run_until_break() {
    let clock = ManualClock::new();
    let mut scheduler = make_scheduler!(&str, clock);
    let _ = scheduler.schedule_in(VDuration::from_secs(10), "event").unwrap();
    let target = scheduler.now() + VDuration::from_secs(5);
    scheduler.run_until(target, |_, _| unreachable!());
    assert_eq!(scheduler.now(), target);
}

#[test]
fn event_cancellation() {
    let clock = ManualClock::new();
    let mut scheduler = make_scheduler!(&str, clock);

    let event_to_cancel = scheduler.schedule_in(VDuration::from_secs(5), "cancelled").unwrap();
    let _ = scheduler.schedule_in(VDuration::from_secs(10), "not_cancelled").unwrap();

    scheduler.cancel(event_to_cancel);

    let mut processed_events = Vec::new();
    let target_time = scheduler.now() + VDuration::from_secs(20);

    scheduler.run_until(target_time, |_, payload| {
        processed_events.push(payload);
    });

    assert_eq!(processed_events.len(), 1);
    assert_eq!(processed_events[0], "not_cancelled");
}

#[test]
#[cfg(any(feature = "std", feature = "alloc"))]
fn event_modification() {
    let clock = ManualClock::new();
    let mut scheduler = make_scheduler!(&str, clock);

    let event_id = scheduler.schedule_in(VDuration::from_secs(10), "event_10s").unwrap();
    scheduler.modify_event(event_id, Some(VInstant(VDuration::from_secs(5).0)), Some("event_5s"));

    let mut processed_events = Vec::new();
    let target_time = scheduler.now() + VDuration::from_secs(20);

    scheduler.run_until(target_time, |time, payload| {
        processed_events.push((time, payload));
    });

    assert_eq!(processed_events.len(), 1);
    assert_eq!(processed_events[0].0.0 / 1_000_000_000, 5);
    assert_eq!(processed_events[0].1, "event_5s");
}

