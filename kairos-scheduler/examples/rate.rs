use kairos_core::{RateClock, Clock, VDuration};

fn main() {
    let mut c = RateClock::new();
    c.set_rate(3, 1);                // 3x
    c.tick(VDuration::from_millis(200));
    println!("agora = {:?}", c.now()); // ~600ms virtuais
}
