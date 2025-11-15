use crate::{Clock, VInstant, VDuration};

pub trait Rate {
    fn scale(&self, duration: VDuration) -> VDuration;
}

#[derive(Debug)]
pub struct Q32_32Rate {
    factor_q32_32: u64,
}

impl Default for Q32_32Rate {
    fn default() -> Self {
        Self { factor_q32_32: 1u64 << 32 } // 1.0x
    }
}

impl Rate for Q32_32Rate {
    fn scale(&self, duration: VDuration) -> VDuration {
        let scaled = ((duration.0 as u128 * self.factor_q32_32 as u128) >> 32) as u64;
        VDuration(scaled)
    }
}

/// Rate-adjusted clock: advances by `base * rate`.
#[derive(Debug)]
pub struct RateClock<R: Rate = Q32_32Rate> {
    now: VInstant,
    rate: R,
}

impl<R: Rate + Default> Default for RateClock<R> {
    fn default() -> Self {
        Self {
            now: VInstant(0),
            rate: R::default(),
        }
    }
}

impl<R: Rate + Default> RateClock<R> {
    pub fn new() -> Self { Self::default() }
}

impl RateClock<Q32_32Rate> {
    /// Practical constructor already defining the num/den ratio.
    #[inline(always)]
    pub fn with_rate(num: u32, den: u32) -> Self {
        let mut c = Self::default();
        c.set_rate(num, den);
        c
    }

    /// Sets the speed (e.g.: 2/1 = 2x). Minimum denominator = 1.
    #[inline(always)]
    pub fn set_rate(&mut self, num: u32, den: u32) {
        let den = den.max(1);
        // Q32.32 = (num/den) << 32
        self.rate.factor_q32_32 = ((num as u64) << 32) / (den as u64);
    }
}

impl<R: Rate> RateClock<R> {
    #[inline(always)]
    pub fn tick(&mut self, base: VDuration) {
        let scaled = self.rate.scale(base);
        self.now += scaled;
    }
}

impl<R: Rate> Clock for RateClock<R> {
    #[inline(always)] fn now(&self) -> VInstant { self.now }
    #[inline(always)] fn advance(&mut self, by: VDuration) { self.now += by; }
}
