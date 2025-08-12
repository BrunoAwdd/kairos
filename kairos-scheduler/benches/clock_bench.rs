use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};
use kairos_core::{ManualClock, RateClock, Clock, VDuration};
use std::time::{Instant, SystemTime};

#[cfg(any(feature = "autoclock-std", feature = "autoclock-soft", feature = "autoclock-systick"))]
use kairos_core::autoclock::AutoClock;



fn bench_clocks(c: &mut Criterion) {
    println!("AutoClock backend: {}", kairos_core::autoclock::WHICH_BACKEND);

    let mut group = c.benchmark_group("clocks");
    group.sample_size(200);
    group.sampling_mode(SamplingMode::Linear);
    // group.measurement_time(std::time::Duration::from_secs(8));

    #[cfg(feature = "autoclock-soft")]
    group.bench_function("softclock_now (direto)", |b| {
        let soft = kairos_core::autoclock::SoftClock::new();
        b.iter(|| {
            let s = std::hint::black_box(&soft);
            let _ = s.now(); // deve ficar ~1–2 ns
        })
    });

    group.bench_function("autoclock_now (soft via AutoClock)", |b| {
        let auto = kairos_core::autoclock::AutoClock::new();
        assert!(matches!(kairos_core::autoclock::WHICH_BACKEND, "soft"));
        b.iter(|| {
            let a = std::hint::black_box(&auto);
            let _ = a.now();
        })
    });

    group.bench_function("atomic_load_u64", |b| {
        use core::sync::atomic::{AtomicU64, Ordering};
        static X: AtomicU64 = AtomicU64::new(0);
        b.iter(|| std::hint::black_box(X.load(Ordering::Relaxed)));
    });

    // 0) (vai dar ~0 ps se o compilador otimizar)
    let kairos_cold = ManualClock::new();
    group.bench_function("kairos_now (otimizável)", |b| b.iter(|| {
        let _ = std::hint::black_box(&kairos_cold);
        let _ = kairos_cold.now();
    }));

    // 1) leitura forçada (load real)
    //group.bench_function("kairos_now_volatile", |b| {
    //    let kairos = ManualClock::new();
    //    b.iter(|| {
    //        let k = std::hint::black_box(&kairos);
    //       std::hint::black_box(k.now_volatile());
    //    })
    //});

    // 2) uso “realista”: ler e avançar 1ns (evita CSE/hoist)
    group.bench_function("kairos_now+advance(1ns)", |b| {
        let mut kairos = ManualClock::new();
        b.iter(|| {
            let t = std::hint::black_box(kairos.now());
            std::hint::black_box(t);
            kairos.advance(VDuration::from_nanos(1));
        })
    });

    // 3) RateClock: custo de avançar com escala
    group.bench_function("rate_tick_1ms", |b| {
        let mut rate = RateClock::with_rate(3, 2);
        let base = VDuration::from_millis(1);
        b.iter(|| {
            let d = std::hint::black_box(base);
            rate.tick(d);
        })
    });

    // 4) Referências do SO
    group.bench_function("instant_now", |b| b.iter(|| { let _ = Instant::now(); }));
    group.bench_function("systemtime_now", |b| b.iter(|| { let _ = SystemTime::now(); }));



    group.finish();
}

criterion_group!(benches, bench_clocks);
criterion_main!(benches);
