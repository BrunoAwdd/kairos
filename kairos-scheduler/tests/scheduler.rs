use kairos_core::{ManualClock, VDuration, VInstant, Clock};
use kairos_scheduler::{Scheduler, Event};

#[test]
fn scheduler_runs_events_in_order() {
    let clock = ManualClock::new();
    let mut scheduler = Scheduler::new(clock);

    scheduler.schedule_in(VDuration::from_secs(10), "event_10s");
    scheduler.schedule_in(VDuration::from_secs(5), "event_5s");
    scheduler.schedule_in(VDuration::from_secs(15), "event_15s");

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
    let mut scheduler = Scheduler::new(clock);

    scheduler.schedule_in(VDuration::from_secs(5), "event_5s");
    scheduler.schedule_in(VDuration::from_secs(15), "event_15s");

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
    let mut scheduler: Scheduler<(), _> = Scheduler::new(clock);
    scheduler.clock_mut().advance(VDuration::from_secs(10));
    assert_eq!(scheduler.now().0 / 1_000_000_000, 10);
}

#[test]
fn event_ordering() {
    let event1 = Event {
        at: VInstant(100),
        payload: "event1",
    };
    let event2 = Event {
        at: VInstant(200),
        payload: "event2",
    };
    let event3 = Event {
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
    let mut scheduler = Scheduler::new(clock);
    scheduler.schedule_in(VDuration::from_secs(10), "event");
    let target = scheduler.now() + VDuration::from_secs(5);
    scheduler.run_until(target, |_, _| unreachable!());
    assert_eq!(scheduler.now(), target);
}
