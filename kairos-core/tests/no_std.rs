#[test]
fn addassign_equiv_add() {
    use kairos_core::{VInstant, VDuration};
    let base = VInstant(123);
    let mut x = base;
    x += VDuration::from_secs(1);
    assert_eq!(x.0, (base + VDuration::from_secs(1)).0);
}

// Só roda se o backend soft estiver ativo
#[cfg(all(test, feature = "autoclock-soft"))]
mod tests {
    use super::*;
    use kairos_core::{autoclock::{AutoClock, SoftClock}, clock::Clock};

    #[test]
    fn soft_now_basico() {
        // Estado inicial do soft é 0 ns (static inicializado)
        let clk = AutoClock::new();
        assert_eq!(clk.now().0, 0, "esperava iniciar em 0 ns");

        // Avança 1.000 ns via helper do backend
        SoftClock::tick_ns(1_000);
        assert_eq!(clk.now().0, 1_000, "now() deve refletir tick_ns(1000)");

        // Avança 2 ms (2_000_000 ns)
        SoftClock::tick_ms(2);
        assert_eq!(clk.now().0, 1_000 + 2_000_000, "now() deve refletir +2ms");
    }
}