use kairos_core::{ManualClock, Calendar, Clock, VDuration, VInstant};
 
#[test]
fn monotonicidade_manual_clock() {
    let mut c = ManualClock::new();
    let t0 = c.now();
    c.advance(VDuration::from_nanos(1));
    let t1 = c.now();
    assert!(t1 > t0);
}

#[test]
fn soma_subtracao_wrapping() {
    let mut c = ManualClock::new();
    c.advance(VDuration::from_nanos(u64::MAX));

    c.advance(VDuration::from_nanos(10));
    let _ = c.now();
}

#[test]
fn get_now() {
    let mut c = ManualClock::new();
    // Defina sua âncora (ex.: hoje em São Paulo, UTC-03):
    let cal = Calendar::new()
        .with_epoch_local(2025, 8, 11)
        .with_tz_offset_secs(-3 * 3600);

    let cal2 = Calendar::new()
        .with_epoch_local(2025, 8, 11);

    // avança 12h34m56.789s no tempo virtual
    c.advance(VDuration::from_secs(12*3600 + 34*60 + 56));
    c.advance(VDuration::from_millis(789));

    // formata
    let s = cal.format(c.now());
    let s2 = cal2.format(c.now());
    // ex.: "2025-08-11T12:34:56.789-03:00"
    println!("{s} - {s2} - {:?}", c.now().0);
}


#[test]
fn calendar_local_e_utc() {
    let mut c = ManualClock::new();
    c.advance(VDuration::from_secs(12*3600 + 34*60 + 56));
    c.advance(VDuration::from_millis(789));
    println!("now_ns = {}", c.now().0);

    let cal_local = Calendar::new()
        .with_epoch_local(2025, 8, 11)
        .with_tz_offset_secs(-3*3600);
    let s_local = cal_local.format(c.now());
    println!("local = {s_local}");
    assert_eq!(s_local, "2025-08-11T12:34:56.789-03:00");

    let cal_utc = Calendar::new()
        .with_epoch(2025, 8, 11)
        .with_tz_offset_secs(-3*3600);
    let s_utc = cal_utc.format(c.now());
    println!("utc   = {s_utc}");
    assert_eq!(s_utc, "2025-08-11T09:34:56.789-03:00");
}

#[test]
fn calendar_anchor_sem_dobro_de_fuso() {
    // Âncora LOCAL: 2025-08-11 00:00 no fuso -03
    let cal_local = Calendar::new()
        .with_epoch_local(2025, 8, 11)
        .with_tz_offset_secs(-3 * 3600);

    let mut c = ManualClock::new();
    c.advance(VDuration::from_secs(12*3600 + 34*60 + 56));
    c.advance(VDuration::from_millis(789));
    let s_local = cal_local.format(c.now());
    println!("local   = {s_local}");
    assert_eq!(s_local, "2025-08-11T12:34:56.789-03:00");

    // Âncora UTC: 2025-08-11 00:00Z e exibir em -03
    let cal_utc = Calendar::new()
        .with_epoch(2025, 8, 11)
        .with_tz_offset_secs(-3 * 3600);

    let s_utc = cal_utc.format(c.now());
    println!("utc     = {s_utc}");
    assert_eq!(s_utc, "2025-08-11T09:34:56.789-03:00");
}

#[test]
fn calendar_to_civil_utc() {
    let cal = Calendar::new().with_epoch(2025, 1, 1);
    let t = VInstant::from(VDuration::from_secs(12 * 3600)); // 12 hours
    let (y, m, d, h, ..) = cal.to_civil(t);
    assert_eq!((y, m, d, h), (2025, 1, 1, 12));
}
