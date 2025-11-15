#[cfg(any(feature = "autoclock-soft", feature = "autoclock-systick"))]
mod tests {
    use kairos_core::{AutoClock, Clock, VDuration};

    #[cfg(feature = "autoclock-soft")]
    #[test]
    fn autoclock_soft_works() {
        let mut clock = AutoClock::new();
        let t0 = clock.now();
        kairos_core::autoclock::SoftClock::tick_ms(10);
        let t1 = clock.now();
        assert!(t1 > t0);
        assert_eq!(t1.0 - t0.0, 10_000_000);

        clock.advance(VDuration::from_secs(1));
        let t2 = clock.now();
        assert_eq!(t2.0 - t1.0, 0); // advance is a no-op by default
    }

    #[cfg(feature = "autoclock-soft")]
    #[test]
    fn autoclock_soft_reset_and_set() {
        kairos_core::autoclock::soft_backend::reset_ns();
        assert_eq!(AutoClock::new().now().0, 0);

        kairos_core::autoclock::soft_backend::set_ns(123);
        assert_eq!(AutoClock::new().now().0, 123);
    }

    #[cfg(all(feature = "autoclock-systick", any(target_arch = "arm", target_arch = "aarch64")))]
    #[test]
    fn autoclock_systick_works() {
        // This test can only be run on an ARM target.
        // It requires a mock of the SysTick registers.
        // For now, we just check that the clock can be created.
        let _clock = AutoClock::new();
    }
}