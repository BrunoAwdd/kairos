use kairos_core::{ManualClock, VDuration};
use kairos_scheduler::Scheduler;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Hash)]
struct SimulationState {
    time: u64,
    value: u32,
}

fn run_simulation(clock: ManualClock) -> u64 {
    let mut scheduler = Scheduler::new(clock);
    let mut state = SimulationState { time: 0, value: 0 };

    scheduler.schedule_in(VDuration::from_secs(1), 10);
    scheduler.schedule_in(VDuration::from_secs(2), 20);
    scheduler.schedule_in(VDuration::from_secs(3), 30);

    let target_time = scheduler.now() + VDuration::from_secs(5);
    scheduler.run_until(target_time, |time, payload| {
        state.time = time.0;
        state.value += payload;
    });

    let mut hasher = DefaultHasher::new();
    state.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_determinism_manual_clock() {
    let first_hash = run_simulation(ManualClock::new());
    for _ in 1..1000 {
        let hash = run_simulation(ManualClock::new());
        assert_eq!(first_hash, hash);
    }
}
