use kairos_core::{VDuration, VInstant};

#[test]
fn vduration_from_micros() {
    assert_eq!(VDuration::from_micros(1), VDuration::from_nanos(1_000));
}

#[test]
fn vduration_as_nanos() {
    assert_eq!(VDuration::from_secs(1).as_nanos(), 1_000_000_000);
}

#[test]
fn vduration_as_secs_f64() {
    assert_eq!(VDuration::from_millis(500).as_secs_f64(), 0.5);
}

#[test]
fn vinstant_sub() {
    let t1 = VInstant(1000);
    let t2 = VInstant(500);
    assert_eq!(t1 - t2, VDuration(500));
}

#[test]
fn vduration_add() {
    let d1 = VDuration(100);
    let d2 = VDuration(200);
    assert_eq!(d1 + d2, VDuration(300));
}

#[test]
fn vduration_sub() {
    let d1 = VDuration(300);
    let d2 = VDuration(100);
    assert_eq!(d1 - d2, VDuration(200));
}

#[test]
fn vduration_from_core_duration() {
    let core_dur = core::time::Duration::new(1, 500_000_000); // 1.5 seconds
    let v_dur = VDuration::from(core_dur);
    assert_eq!(v_dur, VDuration::from_millis(1500));
}

#[test]
fn vduration_add_assign() {
    let mut d1 = VDuration::from_secs(1);
    let d2 = VDuration::from_secs(2);
    d1 += d2;
    assert_eq!(d1.0, 3_000_000_000);
}

#[test]
fn vinstant_from_vduration() {
    let d = VDuration::from_secs(5);
    let t = VInstant::from(d);
    assert_eq!(t.0, 5_000_000_000);
}
