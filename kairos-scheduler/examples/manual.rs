use kairos_core::{ManualClock, VDuration};
use kairos_scheduler::Scheduler;

fn main() {
    let mut sched = Scheduler::new(ManualClock::new());
    let _ = sched.schedule_in(VDuration::from_millis(500), "meio segundo").unwrap();
    let _ = sched.schedule_in(VDuration::from_millis(1000), "um segundo").unwrap();

    sched.run_until(kairos_core::VInstant(1_000_000_000), |timestamp, msg| {
        println!("[t={}] evento: {}", timestamp.as_nanos(), msg);
    });
}
