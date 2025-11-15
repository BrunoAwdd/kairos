use kairos_core::{ManualClock, VDuration};
use kairos_scheduler::Scheduler;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

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

#[derive(Hash)]
struct SimulationState {
    time: u64,
    value: u32,
}

fn run_simulation(clock: ManualClock) -> u64 {
    let mut scheduler = make_scheduler!(u32, clock);
    let mut state = SimulationState { time: 0, value: 0 };

    let _ = scheduler.schedule_in(VDuration::from_secs(1), 10).unwrap();
    let _ = scheduler.schedule_in(VDuration::from_secs(2), 20).unwrap();
    let _ = scheduler.schedule_in(VDuration::from_secs(3), 30).unwrap();

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
