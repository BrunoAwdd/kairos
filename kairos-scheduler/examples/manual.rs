use kairos_core::{ManualClock, VDuration};
use kairos_scheduler::Scheduler;

fn main() {
    let mut sched = Scheduler::new(ManualClock::new());
    sched.schedule_in(VDuration::from_millis(500), "meio segundo");
    sched.schedule_in(VDuration::from_millis(1000), "um segundo");

    sched.run_until(kairos_core::VInstant(1_000_000_000), |msg| {
        println!("[t=1s] evento: {msg}");
    });
}
